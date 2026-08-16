use crate::lock::{normalize_repository, UpstreamLock};
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, path::Path, process::Command};

pub const MANDATORY_PATHS: &[&str] = &[
    "README.md",
    "Cargo.toml",
    "package.json",
    "pnpm-workspace.yaml",
    "packages/react/src/index.ts",
    "packages/react/src/host-config.ts",
    "packages/react/src/reconciler.ts",
    "packages/react/src/scene.ts",
    "packages/react/src/frame.ts",
    "packages/react/src/sequence.ts",
    "packages/react/src/interpolate.ts",
    "packages/react/src/spring.ts",
    "packages/react/src/random.ts",
    "packages/react/src/transitions.ts",
    "packages/react/src/components.ts",
    "packages/react/src/fonts.ts",
    "packages/react/src/warmers.ts",
    "packages/react/src/reconciler.test.tsx",
    "packages/react/src/animation.test.tsx",
    "packages/react/src/sequence.test.tsx",
    "packages/react/src/random.test.ts",
    "packages/react/src/transitions.test.tsx",
    "packages/react/src/video.test.tsx",
    "packages/cinema/src/types.ts",
    "packages/cinema/src/timing.ts",
    "packages/cinema/src/props.ts",
    "packages/cinema/src/index.tsx",
    "packages/cinema/src/studio-payload.test.tsx",
    "packages/cinema/src/validate.test.tsx",
    "packages/cinema/src/inspect/index.ts",
    "packages/cinema/src/inspect/resolve.ts",
    "packages/scene-rs/src/lib.rs",
    "packages/animation-rs/src/lib.rs",
    "packages/layout-rs/src/lib.rs",
    "packages/image-rs/src/lib.rs",
    "packages/svg-rs/src/lib.rs",
    "packages/render/src/index.ts",
    "packages/cli-rs/src/main.rs",
    "packages/player/src/player.tsx",
    "packages/player/src/engine-drawer.ts",
    "packages/player/src/canvas-renderer.ts",
    "packages/player/src/images.ts",
    "packages/player/src/video.ts",
    "packages/player/src/audio.ts",
    "packages/player/src/audio-engine.ts",
    "packages/wasm/src/lib.rs",
    "packages/wasm-vello/src/lib.rs",
    "packages/components/package.json",
    "packages/components/src/index.ts",
    "packages/components/src/manifest.ts",
];

#[derive(Debug, PartialEq, Eq)]
pub struct InventoryCounts {
    pub onda_files: usize,
    pub external_references: usize,
}

pub fn remediate(root: &Path, model: &mut Value, lock: &UpstreamLock) -> Result<()> {
    let checkout = root.join(".cinekernel/upstreams/onda");
    let old = model["sources"]
        .as_array()
        .context("canonical sources missing")?
        .clone();
    let mut normalized = Vec::new();
    let mut paths = BTreeSet::new();
    for source in old {
        if let Some(path) = source.get("path").and_then(Value::as_str) {
            paths.insert(path.to_owned());
            normalized.push(local_record(&checkout, lock, path, Some(&source))?);
        } else {
            normalized.push(external_record(&source)?);
        }
    }
    for path in MANDATORY_PATHS {
        if paths.insert((*path).to_owned()) {
            normalized.push(local_record(&checkout, lock, path, None)?);
        }
    }
    normalized.sort_by(|a, b| {
        a["source_id"]
            .as_str()
            .unwrap_or_default()
            .cmp(b["source_id"].as_str().unwrap_or_default())
    });
    model["sources"] = Value::Array(normalized);
    model["onda_repository"] = Value::String(lock.repository.clone());
    model["onda_pin"] = Value::String(lock.pinned_commit.clone());
    model["onda_tree"] = Value::String(lock.pinned_tree.clone());
    Ok(())
}

pub fn verify(root: &Path, sources: &[Value], lock: &UpstreamLock) -> Result<InventoryCounts> {
    let checkout = root.join(".cinekernel/upstreams/onda");
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut onda_files = 0;
    let mut external_references = 0;
    for source in sources {
        let id = source["source_id"].as_str().context("source_id missing")?;
        if !ids.insert(id) {
            bail!("duplicate source ID {id}")
        }
        match source["classification"].as_str() {
            Some("PRIMARY_STANDARD" | "PRIMARY_IMPLEMENTATION_DOC") => {
                external_references += 1;
                let url = source["document_url"]
                    .as_str()
                    .context("external document_url missing")?;
                if !url.starts_with("https://") {
                    bail!("external source {id} does not use an immutable HTTPS identity")
                }
            }
            Some(
                "UPSTREAM_SOURCE"
                | "UPSTREAM_TEST"
                | "UPSTREAM_MANIFEST"
                | "UPSTREAM_DOCUMENTATION",
            ) => {
                onda_files += 1;
                verify_local(&checkout, source, lock)?;
                paths.insert(
                    source["path"]
                        .as_str()
                        .context("local source path missing")?
                        .to_owned(),
                );
            }
            Some(other) => bail!("invalid source classification {other}"),
            None => bail!("source classification missing"),
        }
    }
    for path in MANDATORY_PATHS {
        if !paths.contains(*path) {
            bail!("mandatory source coverage missing: {path}")
        }
    }
    Ok(InventoryCounts {
        onda_files,
        external_references,
    })
}

fn local_record(
    checkout: &Path,
    lock: &UpstreamLock,
    path: &str,
    previous: Option<&Value>,
) -> Result<Value> {
    let bytes = git_bytes(
        checkout,
        &[
            "cat-file",
            "blob",
            &format!("{}:{path}", lock.pinned_commit),
        ],
    )?;
    let text = std::str::from_utf8(&bytes).with_context(|| format!("{path} is not UTF-8"))?;
    let line_count = text.lines().count().max(1);
    let prior_symbol = previous
        .and_then(|p| p.get("symbol").or_else(|| p.get("symbol_or_section")))
        .and_then(Value::as_str)
        .unwrap_or("WHOLE_FILE");
    let symbol = if prior_symbol != "WHOLE_FILE" && text.contains(prior_symbol) {
        prior_symbol
    } else {
        "WHOLE_FILE"
    };
    let (start_line, end_line) = if symbol == "WHOLE_FILE" {
        (1, line_count)
    } else {
        let ranges = previous
            .and_then(|p| p.get("line_ranges"))
            .and_then(Value::as_array);
        let start = ranges
            .and_then(|r| r.first())
            .and_then(Value::as_array)
            .and_then(|r| r.first())
            .and_then(Value::as_u64)
            .unwrap_or(1) as usize;
        let end = ranges
            .and_then(|r| r.first())
            .and_then(Value::as_array)
            .and_then(|r| r.get(1))
            .and_then(Value::as_u64)
            .unwrap_or(line_count as u64) as usize;
        (start.min(line_count).max(1), end.min(line_count).max(start))
    };
    let source_id = previous
        .and_then(|p| p.get("id").or_else(|| p.get("source_id")))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| source_id(path));
    let facts = previous
        .and_then(|p| p.get("facts").or_else(|| p.get("facts_supported")))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_else(|| vec![Value::String(mandatory_fact(path).into())]);
    Ok(json!({
        "source_id": source_id,
        "repository": lock.repository,
        "pinned_commit": lock.pinned_commit,
        "pinned_tree": lock.pinned_tree,
        "path": path,
        "git_blob": git_text(checkout, &["rev-parse", &format!("{}:{path}", lock.pinned_commit)])?,
        "symbol_or_section": symbol,
        "start_line": start_line,
        "end_line": end_line,
        "file_sha256": sha256(&bytes),
        "classification": classification(path),
        "facts_supported": facts
    }))
}

fn external_record(previous: &Value) -> Result<Value> {
    let id = previous
        .get("id")
        .or_else(|| previous.get("source_id"))
        .and_then(Value::as_str)
        .context("external id missing")?;
    Ok(json!({
        "source_id": id,
        "classification": match id {
            "E-REACT" => "PRIMARY_IMPLEMENTATION_DOC",
            _ => "PRIMARY_STANDARD"
        },
        "publisher": previous.get("publisher").and_then(Value::as_str).context("publisher missing")?,
        "document_title": previous.get("title").or_else(||previous.get("document_title")).and_then(Value::as_str).context("external title missing")?,
        "document_url": previous.get("url").or_else(||previous.get("document_url")).and_then(Value::as_str).context("external URL missing")?,
        "accessed_at_utc": previous.get("accessed_at").or_else(||previous.get("accessed_at_utc")).and_then(Value::as_str).context("access time missing")?,
        "section": previous.get("section").and_then(Value::as_str).context("external section missing")?,
        "facts_supported": previous.get("facts").or_else(||previous.get("facts_supported")).and_then(Value::as_array).context("external facts missing")?
    }))
}

fn verify_local(checkout: &Path, source: &Value, lock: &UpstreamLock) -> Result<()> {
    let id = source["source_id"].as_str().context("source ID missing")?;
    if normalize_repository(source["repository"].as_str().unwrap_or_default())
        != normalize_repository(&lock.repository)
    {
        bail!("source {id} repository identity mismatch")
    }
    if source["pinned_commit"].as_str() != Some(&lock.pinned_commit)
        || source["pinned_tree"].as_str() != Some(&lock.pinned_tree)
    {
        bail!("source {id} pin or tree mismatch")
    }
    let path = source["path"].as_str().context("source path missing")?;
    if path.contains("/main/") || path.contains("/master/") || path.starts_with("refs/heads/") {
        bail!("source {id} contains a floating reference")
    }
    let spec = format!("{}:{path}", lock.pinned_commit);
    let bytes = git_bytes(checkout, &["cat-file", "blob", &spec])?;
    let actual_blob = git_text(checkout, &["rev-parse", &spec])?;
    if source["git_blob"].as_str() != Some(&actual_blob) {
        bail!("source {id} Git blob mismatch")
    }
    if source["file_sha256"].as_str() != Some(&sha256(&bytes)) {
        bail!("source {id} SHA-256 mismatch")
    }
    let text = std::str::from_utf8(&bytes).context("indexed source is not UTF-8")?;
    let line_count = text.lines().count().max(1) as u64;
    let start = source["start_line"]
        .as_u64()
        .context("start_line missing")?;
    let end = source["end_line"].as_u64().context("end_line missing")?;
    if start == 0 || end < start || end > line_count {
        bail!("source {id} has invalid line range {start}-{end} for {line_count} lines")
    }
    let symbol = source["symbol_or_section"]
        .as_str()
        .context("symbol_or_section missing")?;
    if symbol != "WHOLE_FILE" && !text.contains(symbol) {
        bail!("source {id} symbol or section is absent: {symbol}")
    }
    Ok(())
}

fn source_id(path: &str) -> String {
    let slug = path
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!("S-MANDATORY-{}", slug.trim_matches('-'))
}

fn classification(path: &str) -> &'static str {
    if path.ends_with(".test.ts") || path.ends_with(".test.tsx") || path.ends_with(".test.rs") {
        "UPSTREAM_TEST"
    } else if path.ends_with("Cargo.toml")
        || path.ends_with("package.json")
        || path.ends_with("pnpm-workspace.yaml")
    {
        "UPSTREAM_MANIFEST"
    } else if path.ends_with(".md") {
        "UPSTREAM_DOCUMENTATION"
    } else {
        "UPSTREAM_SOURCE"
    }
}

fn mandatory_fact(path: &str) -> &'static str {
    if path.contains("react/src") {
        "Required evidence for React authoring, timing, primitive, transition, or lowering behavior."
    } else if path.contains("cinema/src") {
        "Required evidence for Cinema validation, property adaptation, inspection, or lowering behavior."
    } else if path.contains("player/src") {
        "Required evidence for preview scheduling, media, fallback, or engine-boundary behavior."
    } else if path.contains("components") {
        "Required evidence for public component discovery and registry boundaries."
    } else {
        "Required R0.02 repository or compiler-boundary evidence."
    }
}

fn git_text(root: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git_bytes(root, args)?)?.trim().to_owned())
}

fn git_bytes(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        )
    }
    Ok(output.stdout)
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mandatory_inventory_has_no_duplicate_paths() {
        let set: BTreeSet<_> = MANDATORY_PATHS.iter().collect();
        assert_eq!(set.len(), MANDATORY_PATHS.len());
    }

    #[test]
    fn source_ids_are_stable() {
        assert_eq!(
            source_id("packages/react/src/index.ts"),
            "S-MANDATORY-PACKAGES-REACT-SRC-INDEX-TS"
        );
    }

    #[test]
    fn classifications_distinguish_tests_and_manifests() {
        assert_eq!(classification("x.test.tsx"), "UPSTREAM_TEST");
        assert_eq!(classification("package.json"), "UPSTREAM_MANIFEST");
        assert_eq!(classification("x.ts"), "UPSTREAM_SOURCE");
    }
}
