use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};
use walkdir::WalkDir;

const BASE: &str = "974d93ef224b75383499cdb2b70cc086a0dd6f40";

pub fn run(root: &Path) -> Result<Vec<String>> {
    let changed = changed_paths(root)?;
    validate_scope(&changed)?;
    validate_no_tracked_private_paths(root)?;
    validate_dependencies(root)?;
    validate_absolute_paths(root, &changed)?;
    validate_exact_files(root, &changed)?;
    validate_fragments(root, &changed)?;
    Ok(vec![
        "Phase 0 and R0.01 frozen paths unchanged".into(),
        "no tracked private/generated upstream paths".into(),
        "all tracked manifests reject direct, aliased, and Git dependencies on research subjects"
            .into(),
        "absolute-path variants absent from all changed research artifacts".into(),
        "exact nontrivial upstream-file hash comparison passed".into(),
        "normalized multiline and long exact-fragment comparison passed".into(),
    ])
}

pub fn manifest_has_prohibited_dependency(path: &str, text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    let banned = [
        "onda-engine",
        "onda_engine",
        "remotion",
        "hyperframes",
        "hyperframe",
    ];
    if path.ends_with("Cargo.toml") {
        let mut dependencies = false;
        for line in lower.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                dependencies = trimmed.contains("dependencies");
                continue;
            }
            if dependencies && banned.iter().any(|name| trimmed.contains(name)) {
                return true;
            }
        }
        return false;
    }
    if path.ends_with("package.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&lower) {
            for section in [
                "dependencies",
                "devdependencies",
                "peerdependencies",
                "optionaldependencies",
            ] {
                if let Some(map) = json.get(section).and_then(serde_json::Value::as_object) {
                    if map.iter().any(|(name, value)| {
                        banned.iter().any(|b| {
                            name.contains(b) || value.as_str().is_some_and(|v| v.contains(b))
                        })
                    }) {
                        return true;
                    }
                }
            }
        }
        return false;
    }
    if path.ends_with("pnpm-lock.yaml") || path.ends_with("Cargo.lock") {
        return banned.iter().any(|name| lower.contains(name));
    }
    false
}

pub fn contains_absolute_path(text: &str) -> bool {
    let normalized = text.replace('\\', "/");
    let users = ["", "Users", ""].join("/");
    let home = ["", "home", ""].join("/");
    let temporary = ["", "tmp", ""].join("/");
    normalized.contains(&users)
        || normalized.contains(&home)
        || normalized.contains(&temporary)
        || normalized.contains(&format!(":{users}"))
        || normalized.contains(&format!(":/{}", ["Documents", ""].join("/")))
        || normalized.contains(&format!(":/{}", ["Repos", ""].join("/")))
}

pub fn normalized_fragments(text: &str) -> BTreeSet<String> {
    let lines: Vec<String> = text
        .lines()
        .map(normalize_line)
        .filter(|line| line.len() >= 12)
        .collect();
    let mut fragments = BTreeSet::new();
    for window in lines.windows(4) {
        let joined = window.join("\n");
        if joined.len() >= 240 {
            fragments.insert(joined);
        }
    }
    fragments
}

#[cfg(test)]
pub fn exact_nontrivial_copy(upstream: &[u8], candidate: &[u8]) -> bool {
    upstream.len() >= 256 && upstream == candidate
}

#[cfg(test)]
pub fn has_multiline_copy(upstream: &str, candidate: &str) -> bool {
    let source = normalized_fragments(upstream);
    normalized_fragments(candidate)
        .iter()
        .any(|fragment| source.contains(fragment))
}

fn validate_scope(changed: &[String]) -> Result<()> {
    let frozen = [
        "reports/phase0/",
        "benchmarks/upstreams.lock.json",
        "docs/research/remotion/",
        "docs/research/hyperframes/",
        "docs/research/onda/r0.01/",
        "reports/research/r0.01/",
        "schemas/research/onda-",
        ".github/workflows/r0-01-onda-provenance.yml",
        "crates/xtask/src/research_onda.rs",
        "crates/xtask/src/research_onda_pnpm.rs",
    ];
    let allowed = [
        "docs/research/onda/r0.02/",
        "reports/research/r0.02/",
        "schemas/research/r0.02/",
        "tools/research/onda-r0-02/",
        ".github/workflows/r0-02-onda-architecture.yml",
    ];
    for path in changed {
        if frozen
            .iter()
            .any(|item| path == item || path.starts_with(item))
        {
            bail!("frozen path changed: {path}")
        }
        if !allowed
            .iter()
            .any(|item| path == item || path.starts_with(item))
        {
            bail!("R0.02 changed an out-of-scope path: {path}")
        }
    }
    Ok(())
}

fn validate_no_tracked_private_paths(root: &Path) -> Result<()> {
    for path in git(root, &["ls-files"])?.lines() {
        if path.starts_with(".cinekernel/")
            || path.contains("/node_modules/")
            || path.starts_with("node_modules/")
            || path.contains("/target/")
            || path.starts_with("target/")
        {
            bail!("tracked private or generated path: {path}")
        }
    }
    Ok(())
}

fn validate_dependencies(root: &Path) -> Result<()> {
    for path in git(root, &["ls-files"])?.lines().filter(|p| {
        p.ends_with("Cargo.toml")
            || p.ends_with("Cargo.lock")
            || p.ends_with("package.json")
            || p.ends_with("pnpm-lock.yaml")
    }) {
        let text = fs::read_to_string(root.join(path))?;
        if manifest_has_prohibited_dependency(path, &text) {
            let spec = format!("{BASE}:{path}");
            let baseline = Command::new("git")
                .args(["show", &spec])
                .current_dir(root)
                .output()?;
            let baseline_text = if baseline.status.success() {
                String::from_utf8_lossy(&baseline.stdout).into_owned()
            } else {
                String::new()
            };
            if !manifest_has_prohibited_dependency(path, &baseline_text)
                || text.replace("\r\n", "\n") != baseline_text.replace("\r\n", "\n")
            {
                bail!("new or modified prohibited permanent dependency in {path}")
            }
        }
    }
    Ok(())
}

fn validate_absolute_paths(root: &Path, changed: &[String]) -> Result<()> {
    for path in changed {
        let full = root.join(path);
        if full.is_file() {
            let text = fs::read_to_string(&full).unwrap_or_default();
            if contains_absolute_path(&text) {
                bail!("absolute path leaked into {path}")
            }
        }
    }
    Ok(())
}

fn validate_exact_files(root: &Path, changed: &[String]) -> Result<()> {
    let upstream = root.join(".cinekernel/upstreams/onda");
    let mut upstream_hashes = BTreeMap::new();
    for entry in files(&upstream) {
        let bytes = fs::read(&entry)?;
        if bytes.len() >= 256 {
            upstream_hashes.insert(sha256(&bytes), entry);
        }
    }
    for path in changed {
        let full = root.join(path);
        if full.is_file() {
            let bytes = fs::read(&full)?;
            if bytes.len() >= 256 {
                if let Some(upstream_path) = upstream_hashes.get(&sha256(&bytes)) {
                    bail!(
                        "exact upstream file copied into {path} from {}",
                        upstream_path.display()
                    )
                }
            }
        }
    }
    Ok(())
}

fn validate_fragments(root: &Path, changed: &[String]) -> Result<()> {
    let upstream = root.join(".cinekernel/upstreams/onda");
    let mut exact_lines = BTreeSet::new();
    let mut chunks = BTreeSet::new();
    for path in files(&upstream) {
        if fs::metadata(&path)
            .map(|m| m.len() > 1_000_000)
            .unwrap_or(true)
        {
            continue;
        }
        if let Ok(text) = fs::read_to_string(path) {
            exact_lines.extend(
                text.lines()
                    .map(str::trim)
                    .filter(|line| line.len() >= 120)
                    .map(str::to_owned),
            );
            chunks.extend(normalized_fragments(&text));
        }
    }
    for path in changed {
        if !(path.ends_with(".rs")
            || path.ends_with(".md")
            || path.ends_with(".json")
            || path.ends_with(".yml")
            || path.ends_with(".toml"))
        {
            continue;
        }
        let text = fs::read_to_string(root.join(path)).unwrap_or_default();
        if text
            .lines()
            .map(str::trim)
            .any(|line| exact_lines.contains(line))
        {
            bail!("long exact upstream line copied into {path}")
        }
        if normalized_fragments(&text)
            .iter()
            .any(|fragment| chunks.contains(fragment))
        {
            bail!("normalized multiline upstream fragment copied into {path}")
        }
    }
    Ok(())
}

fn changed_paths(root: &Path) -> Result<Vec<String>> {
    let output = git(root, &["diff", "--name-only", BASE])?;
    Ok(output
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect())
}

fn files(root: &Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            !matches!(
                e.file_name().to_string_lossy().as_ref(),
                ".git" | "node_modules" | "target" | "dist" | "coverage" | "pkg"
            )
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect()
}

fn normalize_line(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )
    }
    String::from_utf8(output.stdout)
        .context("git output was not UTF-8")
        .map(|v| v.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catches_cargo_dependency_alias() {
        assert!(manifest_has_prohibited_dependency(
            "Cargo.toml",
            "[dependencies]\nsome_name = { package = \"onda-engine\", version = \"1\" }"
        ));
    }
    #[test]
    fn catches_git_dependency_alias() {
        assert!(manifest_has_prohibited_dependency("Cargo.toml", "[dependencies]\nsafe = { package = \"other\", git = \"https://github.com/onda-engine/onda-engine\" }"));
    }
    #[test]
    fn package_name_outside_dependencies_is_not_a_dependency() {
        assert!(!manifest_has_prohibited_dependency(
            "Cargo.toml",
            "[package]\nname = \"onda-r0-02-research\"\n[dependencies]\nserde = \"1\""
        ));
    }
    #[test]
    fn catches_windows_forward_slash_path() {
        assert!(contains_absolute_path(
            &["C:", "Users", "person", "repo"].join("/")
        ));
    }
    #[test]
    fn catches_windows_backslash_path() {
        assert!(contains_absolute_path(
            &["C:", "Users", "person", "repo"].join("\\")
        ));
    }
    #[test]
    fn catches_macos_path() {
        assert!(contains_absolute_path(
            &["", "Users", "person", "repo"].join("/")
        ));
    }
    #[test]
    fn catches_linux_home_path() {
        assert!(contains_absolute_path(
            &["", "home", "person", "repo"].join("/")
        ));
    }
    #[test]
    fn catches_temp_path() {
        assert!(contains_absolute_path(&["", "tmp", "evidence"].join("/")));
    }
    #[test]
    fn multiline_fragment_is_normalized() {
        let a="alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron\nsecond long line with enough repeated architectural words to qualify as meaningful evidence fragment data\nthird long line with identifiers and behavior descriptions that should survive whitespace normalization\nfourth long line closes a copied source fragment and ensures the combined threshold is exceeded";
        let b = a.replace(' ', "   ");
        assert_eq!(normalized_fragments(a), normalized_fragments(&b));
    }
    #[test]
    fn short_text_is_not_a_fragment() {
        assert!(normalized_fragments("one\ntwo\nthree\nfour").is_empty());
    }
    #[test]
    fn exact_copy_positive_fixture() {
        let bytes = vec![b'x'; 300];
        assert!(exact_nontrivial_copy(&bytes, &bytes));
    }
    #[test]
    fn exact_copy_negative_fixture() {
        let bytes = vec![b'x'; 300];
        let other = vec![b'y'; 300];
        assert!(!exact_nontrivial_copy(&bytes, &other));
    }
    #[test]
    fn multiline_copy_positive_fixture() {
        let source = "first sufficiently long source line describes a compiler boundary and its deterministic ownership rules in abstract terms\nsecond sufficiently long source line describes identity preservation and the point at which semantic data becomes unavailable\nthird sufficiently long source line describes temporal conversion including rounding clamping and fractional behavior\nfourth sufficiently long source line completes the nontrivial normalized fragment used by the positive fixture";
        assert!(has_multiline_copy(source, &source.replace(' ', "  ")));
    }
    #[test]
    fn multiline_copy_negative_fixture() {
        let source = "first sufficiently long source line describes a compiler boundary and its deterministic ownership rules in abstract terms\nsecond sufficiently long source line describes identity preservation and the point at which semantic data becomes unavailable\nthird sufficiently long source line describes temporal conversion including rounding clamping and fractional behavior\nfourth sufficiently long source line completes the nontrivial normalized fragment used by the positive fixture";
        assert!(!has_multiline_copy(source, "independent summary"));
    }
    #[test]
    fn frozen_r001_path_is_rejected() {
        assert!(validate_scope(&["docs/research/onda/r0.01/UPSTREAM_LOCK.json".into()]).is_err());
    }
    #[test]
    fn frozen_phase0_path_is_rejected() {
        assert!(validate_scope(&["reports/phase0/result.json".into()]).is_err());
    }
}
