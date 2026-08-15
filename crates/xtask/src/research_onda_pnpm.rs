use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Default)]
struct PackageRecord {
    integrity: Option<String>,
    source: Option<String>,
}

#[derive(Debug, Default)]
struct SnapshotRecord {
    dependencies: BTreeMap<String, (String, String)>,
}

fn unquote(value: &str) -> &str {
    value.trim().trim_matches('\'').trim_matches('"')
}

fn split_package_key(key: &str) -> Option<(String, String)> {
    let key = unquote(key);
    let base = key.split('(').next().unwrap_or(key);
    let at = base.rfind('@')?;
    if at == 0 {
        return None;
    }
    Some((base[..at].to_owned(), base[at + 1..].to_owned()))
}

fn inline_value(line: &str, field: &str) -> Option<String> {
    let marker = format!("{field}:");
    let start = line.find(&marker)? + marker.len();
    let rest = line[start..].trim_start();
    let end = rest.find([',', '}']).unwrap_or(rest.len());
    Some(unquote(rest[..end].trim()).to_owned())
}

fn parse_packages(lines: &[&str], start: usize, end: usize) -> BTreeMap<String, PackageRecord> {
    let mut output = BTreeMap::new();
    let mut key = String::new();
    for line in &lines[start..end] {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if indent == 2 && trimmed.ends_with(':') {
            key = unquote(trimmed.trim_end_matches(':')).to_owned();
            if split_package_key(&key).is_some() {
                output.insert(key.clone(), PackageRecord::default());
            }
        } else if indent == 4 && trimmed.starts_with("resolution:") {
            if let Some(record) = output.get_mut(&key) {
                record.integrity = inline_value(trimmed, "integrity");
                record.source = inline_value(trimmed, "tarball")
                    .or_else(|| Some("https://registry.npmjs.org".to_owned()));
            }
        }
    }
    output
}

fn parse_snapshots(lines: &[&str], start: usize) -> BTreeMap<String, SnapshotRecord> {
    let mut output: BTreeMap<String, SnapshotRecord> = BTreeMap::new();
    let mut key = String::new();
    let mut section = String::new();
    for line in &lines[start..] {
        let indent = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if indent == 2 && trimmed.ends_with(':') {
            key = unquote(trimmed.trim_end_matches(':')).to_owned();
            output.entry(key.clone()).or_default();
            section.clear();
        } else if indent == 4 && trimmed.ends_with(':') {
            section = trimmed.trim_end_matches(':').to_owned();
        } else if indent == 6
            && trimmed.contains(':')
            && matches!(section.as_str(), "dependencies" | "optionalDependencies")
        {
            if let Some((name, value)) = trimmed.split_once(':') {
                let name = unquote(name).to_owned();
                let value = unquote(value).to_owned();
                output
                    .entry(key.clone())
                    .or_default()
                    .dependencies
                    .insert(name, (value, section.clone()));
            }
        }
    }
    output
}

fn normalize_repository(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::to_owned)
        .or_else(|| value.get("url").and_then(Value::as_str).map(str::to_owned))
}

fn platform_license_family(name: &str) -> Option<&'static str> {
    [
        ("@biomejs/cli-", "@biomejs/cli-platform"),
        ("@esbuild/", "@esbuild/platform"),
        ("@img/sharp-libvips-", "@img/sharp-libvips-platform"),
        ("@img/sharp-", "@img/sharp-platform"),
        ("@pagefind/", "@pagefind/platform"),
        ("@remotion/compositor-", "@remotion/compositor-platform"),
        ("@rollup/rollup-", "@rollup/rollup-platform"),
        ("@rspack/binding-", "@rspack/binding-platform"),
    ]
    .into_iter()
    .find_map(|(prefix, family)| name.starts_with(prefix).then_some(family))
}

fn run_license_inventory(
    upstream: &Path,
    raw: &Path,
    packages: &BTreeMap<String, PackageRecord>,
) -> Result<BTreeMap<(String, String), Value>> {
    let corepack = if cfg!(windows) {
        "corepack.cmd"
    } else {
        "corepack"
    };
    let install = Command::new(corepack)
        .args(["pnpm", "install", "--frozen-lockfile", "--ignore-scripts"])
        .env("CI", "true")
        .current_dir(upstream)
        .output()?;
    if !install.status.success() {
        bail!(
            "scripts-disabled pnpm install failed with {}: {}",
            install.status,
            String::from_utf8_lossy(&install.stderr)
        );
    }
    let output = Command::new(corepack)
        .args(["pnpm", "licenses", "list", "--json"])
        .current_dir(upstream)
        .output()?;
    if !output.status.success() {
        bail!(
            "pnpm license inventory failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let raw_value: Value = serde_json::from_slice(&output.stdout)?;
    fs::create_dir_all(raw)?;
    let mut bytes = serde_json::to_vec_pretty(&raw_value)?;
    bytes.push(b'\n');
    fs::write(raw.join("licenses-raw.json"), bytes)?;

    let mut licenses = BTreeMap::new();
    for (expression, entries) in raw_value.as_object().context("license object")? {
        for entry in entries.as_array().context("license entries")? {
            let name = entry["name"].as_str().context("license package name")?;
            let versions = entry["versions"].as_array().context("license versions")?;
            let paths = entry["paths"].as_array().context("license paths")?;
            for (index, version) in versions.iter().enumerate() {
                let version = version.as_str().context("license version")?;
                let package_path = paths
                    .get(index)
                    .or_else(|| paths.first())
                    .and_then(Value::as_str)
                    .map(PathBuf::from);
                let manifest = package_path
                    .as_ref()
                    .and_then(|path| fs::read(path.join("package.json")).ok())
                    .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
                let repository = manifest
                    .as_ref()
                    .and_then(|value| normalize_repository(&value["repository"]));
                let homepage = manifest
                    .as_ref()
                    .and_then(|value| value["homepage"].as_str().map(str::to_owned))
                    .or_else(|| entry["homepage"].as_str().map(str::to_owned));
                licenses.insert(
                    (name.to_owned(), version.to_owned()),
                    json!({
                        "expression": expression,
                        "repository": repository,
                        "homepage": homepage,
                        "source": "pnpm licenses list --json plus installed package.json",
                        "status": if expression.trim().is_empty() { "UNRESOLVED" } else { "VERIFIED_AT_PIN" }
                    }),
                );
            }
        }
    }
    let mut canonical_platform_packages = BTreeMap::new();
    for key in packages.keys() {
        let Some((name, version)) = split_package_key(key) else {
            continue;
        };
        if let Some(family) = platform_license_family(&name) {
            canonical_platform_packages
                .entry((family.to_owned(), version))
                .and_modify(|canonical: &mut String| {
                    if name < *canonical {
                        *canonical = name.clone();
                    }
                })
                .or_insert(name);
        }
    }
    let mut normalized_families = BTreeMap::new();
    for ((family, version), package_name) in canonical_platform_packages {
        let spec = format!("{package_name}@{version}");
        let output = Command::new(if cfg!(windows) { "npm.cmd" } else { "npm" })
            .args(["view", &spec, "license", "repository", "homepage", "--json"])
            .current_dir(upstream)
            .output()?;
        if !output.status.success() {
            bail!(
                "npm registry metadata lookup failed for {spec}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let metadata: Value = serde_json::from_slice(&output.stdout)?;
        let expression = metadata["license"].as_str().unwrap_or("UNKNOWN/CUSTOM");
        normalized_families.insert(
            (family, version),
            json!({
                "expression":expression,
                "repository":normalize_repository(&metadata["repository"]),
                "homepage":metadata["homepage"].as_str(),
                "source":"npm registry exact package metadata; platform-family normalized",
                "status":if expression == "UNKNOWN/CUSTOM" { "UNRESOLVED" } else { "VERIFIED_AT_PIN" },
                "canonical_package":package_name
            }),
        );
    }
    let family_evidence = normalized_families
        .iter()
        .map(|((family, version), record)| {
            json!({"family":family,"version":version,"record":record})
        })
        .collect::<Vec<_>>();
    write_json_value(raw.join("platform-family-licenses.json"), &family_evidence)?;
    for (key, mut record) in normalized_families {
        record
            .as_object_mut()
            .expect("license record")
            .remove("canonical_package");
        licenses.insert(key, record);
    }
    Ok(licenses)
}

fn write_json_value(path: PathBuf, value: &impl serde::Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn snapshot_target(name: &str, resolution: &str) -> Option<String> {
    if resolution.starts_with("link:") || resolution.starts_with("workspace:") {
        None
    } else {
        Some(format!("{name}@{resolution}"))
    }
}

type PropagatedGraphState = (
    BTreeSet<String>,
    BTreeMap<String, BTreeSet<String>>,
    BTreeMap<String, BTreeSet<String>>,
);

fn propagate_graph_state(
    snapshots: &BTreeMap<String, SnapshotRecord>,
    declarations: &[Value],
) -> PropagatedGraphState {
    let mut direct = BTreeSet::new();
    let mut classifications: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut reachability: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut queue = VecDeque::new();
    for declaration in declarations {
        let name = declaration["dependency"].as_str().unwrap_or_default();
        let resolved = declaration["resolved_version"].as_str().unwrap_or_default();
        if let Some(target) = snapshot_target(name, resolved) {
            let class = declaration["classification"].as_str().unwrap_or("unknown");
            let importer = declaration["manifest"].as_str().unwrap_or("unknown");
            direct.insert(target.clone());
            classifications
                .entry(target.clone())
                .or_default()
                .insert(class.to_owned());
            reachability
                .entry(target.clone())
                .or_default()
                .insert(importer.to_owned());
            queue.push_back(target);
        }
    }
    while let Some(parent) = queue.pop_front() {
        let parent_classes = classifications.get(&parent).cloned().unwrap_or_default();
        let parent_importers = reachability.get(&parent).cloned().unwrap_or_default();
        let Some(snapshot) = snapshots.get(&parent) else {
            continue;
        };
        for (name, (resolution, edge_kind)) in &snapshot.dependencies {
            let Some(target) = snapshot_target(name, resolution) else {
                continue;
            };
            let classes = classifications.entry(target.clone()).or_default();
            let previous_class_count = classes.len();
            classes.extend(parent_classes.iter().cloned());
            if edge_kind == "optionalDependencies" {
                classes.insert("optional".to_owned());
            }
            let importers = reachability.entry(target.clone()).or_default();
            let previous_importer_count = importers.len();
            importers.extend(parent_importers.iter().cloned());
            if classes.len() != previous_class_count || importers.len() != previous_importer_count {
                queue.push_back(target);
            }
        }
    }
    (direct, classifications, reachability)
}

pub(crate) fn build_graph(upstream: &Path, raw: &Path, declarations: &[Value]) -> Result<Value> {
    let lock = fs::read_to_string(upstream.join("pnpm-lock.yaml"))?;
    let lines = lock.lines().collect::<Vec<_>>();
    let packages_start = lines
        .iter()
        .position(|line| *line == "packages:")
        .context("pnpm packages section")?
        + 1;
    let snapshots_start = lines
        .iter()
        .position(|line| *line == "snapshots:")
        .context("pnpm snapshots section")?;
    let packages = parse_packages(&lines, packages_start, snapshots_start);
    let snapshots = parse_snapshots(&lines, snapshots_start + 1);
    let licenses = run_license_inventory(upstream, raw, &packages)?;

    let (direct, classifications, reachability) = propagate_graph_state(&snapshots, declarations);

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for (key, snapshot) in &snapshots {
        let (name, version) =
            split_package_key(key).unwrap_or_else(|| (key.clone(), "UNRESOLVED".to_owned()));
        let package_key = format!("{name}@{version}");
        let package = packages.get(&package_key);
        let exact_license_key = (name.clone(), version.clone());
        let family_license_key =
            platform_license_family(&name).map(|family| (family.to_owned(), version.clone()));
        let license = family_license_key
            .as_ref()
            .and_then(|key| licenses.get(key))
            .or_else(|| licenses.get(&exact_license_key))
            .cloned()
            .unwrap_or_else(|| {
                json!({
                    "expression":"UNKNOWN/CUSTOM","repository":null,"homepage":null,
                    "source":"not returned by pnpm licenses list","status":"UNRESOLVED"
                })
            });
        let node_edges = snapshot.dependencies.iter().filter_map(|(dependency,(resolution,kind))| {
            snapshot_target(dependency,resolution).map(|target| {
                edges.push(json!({"from":format!("npm:{key}"),"to":format!("npm:{target}"),"kind":kind}));
                json!({"target":format!("npm:{target}"),"kind":kind})
            })
        }).collect::<Vec<_>>();
        nodes.push(json!({
            "id":format!("npm:{key}"),"package_name":name,"version":version,
            "source_kind":"registry","registry":package.and_then(|p|p.source.clone()).unwrap_or_else(||"https://registry.npmjs.org".to_owned()),
            "integrity":package.and_then(|p|p.integrity.clone()),"direct":direct.contains(key),
            "classifications":classifications.get(key).cloned().unwrap_or_default(),
            "reachable_from":reachability.get(key).cloned().unwrap_or_default(),
            "reachability_status":if reachability.contains_key(key) { "REACHABLE_FROM_WORKSPACE_IMPORTER" } else { "NOT_REACHABLE_FROM_WORKSPACE_IMPORTER" },
            "license":license,"dependencies":node_edges
        }));
    }
    nodes.sort_by_key(|node| node["id"].as_str().unwrap_or_default().to_owned());
    edges.sort_by_key(|edge| format!("{}:{}", edge["from"], edge["to"]));
    let license_records = nodes.iter().map(|node| json!({
        "package_id":node["id"],"package_name":node["package_name"],"version":node["version"],
        "expression":node["license"]["expression"],"repository":node["license"]["repository"],
        "homepage":node["license"]["homepage"],"status":node["license"]["status"]
    })).collect::<Vec<_>>();
    Ok(json!({
        "lockfile_version":"9.0","resolved_package_count":nodes.len(),"dependency_edge_count":edges.len(),
        "resolved_packages":nodes,"dependency_edges":edges,"license_records":license_records
    }))
}

pub(crate) fn forbidden_lock_nodes(lockfile: &str) -> Vec<String> {
    let lines = lockfile.lines().collect::<Vec<_>>();
    let Some(packages_start) = lines.iter().position(|line| *line == "packages:") else {
        return vec!["missing packages section".to_owned()];
    };
    let Some(snapshots_start) = lines.iter().position(|line| *line == "snapshots:") else {
        return vec!["missing snapshots section".to_owned()];
    };
    parse_packages(&lines, packages_start + 1, snapshots_start)
        .into_iter()
        .filter_map(|(key, package)| {
            let forbidden_name = split_package_key(&key).is_some_and(|(name, _)| {
                name == "onda-engine"
                    || name.starts_with("onda-")
                    || name.starts_with("@onda-engine/")
            });
            let forbidden_source = package
                .source
                .as_deref()
                .is_some_and(|source| source.contains("onda-engine/onda-engine"));
            (forbidden_name || forbidden_source).then_some(key)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_keys_cover_scoped_and_peer_qualified_nodes() {
        assert_eq!(
            split_package_key("'@scope/name@1.2.3(peer@4)'"),
            Some(("@scope/name".to_owned(), "1.2.3".to_owned()))
        );
        assert_eq!(
            split_package_key("name@2.0.0"),
            Some(("name".to_owned(), "2.0.0".to_owned()))
        );
    }

    #[test]
    fn package_parser_preserves_integrity() {
        let lines = ["  'name@1.0.0':", "    resolution: {integrity: sha512-abc}"];
        let parsed = parse_packages(&lines, 0, lines.len());
        assert_eq!(
            parsed["name@1.0.0"].integrity.as_deref(),
            Some("sha512-abc")
        );
    }

    #[test]
    fn snapshot_parser_preserves_dependency_kinds() {
        let lines = [
            "  parent@1.0.0:",
            "    dependencies:",
            "      child: 2.0.0",
            "    optionalDependencies:",
            "      optional: 3.0.0",
        ];
        let parsed = parse_snapshots(&lines, 0);
        assert_eq!(
            parsed["parent@1.0.0"].dependencies["child"].1,
            "dependencies"
        );
        assert_eq!(
            parsed["parent@1.0.0"].dependencies["optional"].1,
            "optionalDependencies"
        );
    }

    #[test]
    fn resolved_lock_guard_detects_forbidden_and_accepts_clean_nodes() {
        let forbidden = "packages:\n  'onda-engine@0.6.1':\n    resolution: {integrity: sha512-x}\nsnapshots:\n  'onda-engine@0.6.1': {}\n";
        assert_eq!(forbidden_lock_nodes(forbidden), ["onda-engine@0.6.1"]);
        let clean = "packages:\n  'react@19.0.0':\n    resolution: {integrity: sha512-x}\nsnapshots:\n  'react@19.0.0': {}\n";
        assert!(forbidden_lock_nodes(clean).is_empty());
    }

    #[test]
    fn platform_license_families_are_host_neutral() {
        assert_eq!(
            platform_license_family("@esbuild/linux-x64"),
            platform_license_family("@esbuild/win32-x64")
        );
        assert_eq!(
            platform_license_family("@rollup/rollup-linux-x64-gnu"),
            platform_license_family("@rollup/rollup-win32-x64-msvc")
        );
        assert_ne!(
            platform_license_family("@img/sharp-linux-x64"),
            platform_license_family("@img/sharp-libvips-linux-x64")
        );
        assert_eq!(platform_license_family("react"), None);
    }

    #[test]
    fn importer_reachability_converges_across_unequal_paths() {
        let snapshots = BTreeMap::from([
            (
                "shared@1.0.0".to_owned(),
                SnapshotRecord {
                    dependencies: BTreeMap::from([(
                        "leaf".to_owned(),
                        ("1.0.0".to_owned(), "dependencies".to_owned()),
                    )]),
                },
            ),
            ("leaf@1.0.0".to_owned(), SnapshotRecord::default()),
            (
                "bridge@1.0.0".to_owned(),
                SnapshotRecord {
                    dependencies: BTreeMap::from([(
                        "shared".to_owned(),
                        ("1.0.0".to_owned(), "dependencies".to_owned()),
                    )]),
                },
            ),
        ]);
        let declarations = [
            json!({"dependency":"shared","resolved_version":"1.0.0","classification":"external-runtime","manifest":"root-a/package.json"}),
            json!({"dependency":"bridge","resolved_version":"1.0.0","classification":"external-runtime","manifest":"root-b/package.json"}),
        ];
        let first = propagate_graph_state(&snapshots, &declarations);
        let expected = BTreeSet::from([
            "root-a/package.json".to_owned(),
            "root-b/package.json".to_owned(),
        ]);
        assert_eq!(first.2["shared@1.0.0"], expected);
        assert_eq!(first.2["leaf@1.0.0"], expected);

        let second = propagate_graph_state(&snapshots, &declarations);
        assert_eq!(first, second);
        assert_eq!(
            second.2["leaf@1.0.0"].iter().cloned().collect::<Vec<_>>(),
            ["root-a/package.json", "root-b/package.json"]
        );
    }
}
