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
        "all tracked manifests and lockfiles reject authoritative ONDA package identities, aliases, Git sources, and checkout paths"
            .into(),
        "absolute-path variants absent from all changed research artifacts".into(),
        "exact nontrivial upstream-file hash comparison passed".into(),
        "normalized multiline and long exact-fragment comparison passed".into(),
    ])
}

#[cfg(test)]
pub fn manifest_has_prohibited_dependency(path: &str, text: &str) -> bool {
    manifest_has_prohibited_dependency_with_names(path, text, &default_subject_names())
}

fn manifest_has_prohibited_dependency_with_names(
    path: &str,
    text: &str,
    subject_names: &BTreeSet<String>,
) -> bool {
    let lower = text.to_ascii_lowercase();
    if path.ends_with("Cargo.toml") {
        let mut dependencies = false;
        for line in lower.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                dependencies = dependency_section(trimmed);
                continue;
            }
            if dependencies
                && (contains_subject_identity(trimmed, subject_names)
                    || contains_subject_repository(trimmed)
                    || path_points_into_onda_checkout(trimmed))
            {
                return true;
            }
        }
        return false;
    }
    if path.ends_with("package.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&lower) {
            return json_dependency_has_subject(&json, subject_names);
        }
        return false;
    }
    if path.ends_with("Cargo.lock") {
        return cargo_lock_has_subject(&lower, subject_names);
    }
    if path.ends_with("pnpm-lock.yaml") {
        return contains_subject_identity(&lower, subject_names)
            || contains_subject_repository(&lower);
    }
    false
}

fn manifest_has_prohibited_dependency_at(
    root: &Path,
    path: &str,
    text: &str,
    subject_names: &BTreeSet<String>,
) -> bool {
    if manifest_has_prohibited_dependency_with_names(path, text, subject_names) {
        return true;
    }
    if !path.ends_with("Cargo.toml") {
        return false;
    }
    let checkout = root.join(".cinekernel/upstreams/onda").canonicalize().ok();
    let manifest_dir = root.join(path).parent().map(Path::to_path_buf);
    let (Some(checkout), Some(manifest_dir)) = (checkout, manifest_dir) else {
        return false;
    };
    let mut dependencies = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            dependencies = dependency_section(&trimmed.to_ascii_lowercase());
            continue;
        }
        if !dependencies || !trimmed.contains("path") {
            continue;
        }
        for quoted in trimmed.split('"').skip(1).step_by(2) {
            if manifest_dir
                .join(quoted)
                .canonicalize()
                .is_ok_and(|resolved| resolved.starts_with(&checkout))
            {
                return true;
            }
        }
    }
    false
}

fn dependency_section(header: &str) -> bool {
    header
        .trim_matches(|c| c == '[' || c == ']')
        .split('.')
        .any(|part| part.trim_matches('"').ends_with("dependencies"))
}

fn contains_subject_identity(text: &str, names: &BTreeSet<String>) -> bool {
    let normalized = text.to_ascii_lowercase();
    names.iter().any(|name| normalized.contains(name))
        || normalized.contains("@onda-engine/")
        || normalized
            .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '@' | '/')))
            .any(|token| token.starts_with("onda-") && token != "onda-r0-02-research")
}

fn contains_subject_repository(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("github.com/onda-engine/onda-engine")
        || lower.contains("onda_engine")
        || lower.contains("remotion")
        || lower.contains("hyperframes")
        || lower.contains("hyperframe")
}

fn path_points_into_onda_checkout(text: &str) -> bool {
    text.replace('\\', "/")
        .to_ascii_lowercase()
        .contains(".cinekernel/upstreams/onda")
}

fn json_dependency_has_subject(value: &serde_json::Value, names: &BTreeSet<String>) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.iter().any(|(key, value)| {
        let dependency_section = key.to_ascii_lowercase().ends_with("dependencies");
        if dependency_section {
            value.as_object().is_some_and(|dependencies| {
                dependencies.iter().any(|(name, spec)| {
                    contains_subject_identity(name, names)
                        || spec.as_str().is_some_and(|text| {
                            contains_subject_identity(text, names)
                                || contains_subject_repository(text)
                                || path_points_into_onda_checkout(text)
                        })
                })
            })
        } else {
            json_dependency_has_subject(value, names)
        }
    })
}

fn cargo_lock_has_subject(text: &str, names: &BTreeSet<String>) -> bool {
    text.split("[[package]]").skip(1).any(|package| {
        package.lines().any(|line| {
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_prefix("name = ") {
                let name = name.trim_matches('"');
                name != "onda-r0-02-research" && (names.contains(name) || name.starts_with("onda-"))
            } else if trimmed.starts_with("source = ") {
                contains_subject_repository(trimmed)
            } else {
                false
            }
        })
    })
}

fn default_subject_names() -> BTreeSet<String> {
    [
        "onda-engine",
        "@onda-engine/",
        "onda-scene",
        "onda-vello",
        "onda-audio",
        "remotion",
        "hyperframes",
        "hyperframe",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn authoritative_subject_names(root: &Path) -> Result<BTreeSet<String>> {
    let mut names = default_subject_names();
    for relative in [
        "docs/research/onda/r0.01/MODULE_INVENTORY.json",
        "docs/research/onda/r0.01/DEPENDENCY_INVENTORY.json",
    ] {
        let value: serde_json::Value = serde_json::from_slice(&fs::read(root.join(relative))?)?;
        collect_onda_names(&value, &mut names);
    }
    Ok(names)
}

fn collect_onda_names(value: &serde_json::Value, names: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                if matches!(key.as_str(), "name" | "package_name") {
                    if let Some(name) = child.as_str() {
                        let name = name.to_ascii_lowercase();
                        if name == "onda-engine"
                            || name.starts_with("onda-")
                            || name.starts_with("@onda-engine/")
                        {
                            names.insert(name);
                        }
                    }
                }
                collect_onda_names(child, names);
            }
        }
        serde_json::Value::Array(items) => {
            for child in items {
                collect_onda_names(child, names);
            }
        }
        _ => {}
    }
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
    let subject_names = authoritative_subject_names(root)?;
    for path in git(root, &["ls-files"])?.lines().filter(|p| {
        p.ends_with("Cargo.toml")
            || p.ends_with("Cargo.lock")
            || p.ends_with("package.json")
            || p.ends_with("pnpm-lock.yaml")
    }) {
        let text = fs::read_to_string(root.join(path))?;
        if manifest_has_prohibited_dependency_at(root, path, &text, &subject_names) {
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
            if !manifest_has_prohibited_dependency_at(root, path, &baseline_text, &subject_names)
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
    fn catches_authoritative_onda_rust_crates() {
        for name in ["onda-scene", "onda-vello", "onda-audio"] {
            assert!(manifest_has_prohibited_dependency(
                "Cargo.toml",
                &format!("[dependencies]\nsubject = {{ package = \"{name}\", version = \"1\" }}")
            ));
        }
    }
    #[test]
    fn catches_renamed_onda_core_dependency() {
        assert!(manifest_has_prohibited_dependency(
            "Cargo.toml",
            "[target.'cfg(unix)'.dependencies]\nscene_backend = { package = \"onda-core\", optional = true }"
        ));
    }
    #[test]
    fn catches_path_dependency_into_onda_checkout() {
        assert!(manifest_has_prohibited_dependency(
            "Cargo.toml",
            "[build-dependencies]\nbackend = { path = \"../../../.cinekernel/upstreams/onda/packages/scene-rs\" }"
        ));
    }
    #[test]
    fn catches_workspace_and_dev_dependencies() {
        assert!(manifest_has_prohibited_dependency(
            "Cargo.toml",
            "[workspace.dependencies]\nonda-vello = \"1\""
        ));
        assert!(manifest_has_prohibited_dependency(
            "Cargo.toml",
            "[dev-dependencies]\naudio = { package = \"onda-audio\", version = \"1\" }"
        ));
    }
    #[test]
    fn catches_resolved_lockfile_identity() {
        assert!(manifest_has_prohibited_dependency(
            "Cargo.lock",
            "[[package]]\nname = \"onda-scene\"\nversion = \"0.1.0\""
        ));
    }
    #[test]
    fn catches_npm_alias_and_scoped_dependency() {
        assert!(manifest_has_prohibited_dependency(
            "package.json",
            r#"{"devDependencies":{"safe":"npm:@onda-engine/react@1"}}"#
        ));
    }
    #[test]
    fn clean_negative_dependency_is_allowed() {
        assert!(!manifest_has_prohibited_dependency(
            "Cargo.toml",
            "[package]\nname = \"onda-r0-02-research\"\n[dependencies]\nserde = \"1\""
        ));
        assert!(!manifest_has_prohibited_dependency(
            "Cargo.lock",
            "[[package]]\nname = \"onda-r0-02-research\"\nversion = \"0.1.0\"\n[[package]]\nname = \"serde\"\nversion = \"1.0.0\""
        ));
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
