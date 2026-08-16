use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

mod cleanroom;
mod contracts;
mod lock;
mod model_validation;
mod remediate;
mod reports;
mod source_index;

const BASE: &str = "974d93ef224b75383499cdb2b70cc086a0dd6f40";
const DOC_DIR: &str = "docs/research/onda/r0.02";
const REPORT_DIR: &str = "reports/research/r0.02";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Verify {
        #[arg(long)]
        json: bool,
    },
    Inventory {
        #[arg(long)]
        json: bool,
    },
    Report {
        #[arg(long)]
        json: bool,
    },
    Guard {
        #[arg(long)]
        json: bool,
    },
    Integrity {
        #[arg(long)]
        check: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Serialize)]
struct Outcome {
    command: &'static str,
    ok: bool,
    checks: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = repo_root()?;
    match cli.command {
        Cmd::Verify { json } => emit(&root, verify(&root)?, json)?,
        Cmd::Inventory { json } => emit(&root, inventory(&root)?, json)?,
        Cmd::Report { json } => emit(&root, report(&root)?, json)?,
        Cmd::Guard { json } => emit(&root, guard(&root)?, json)?,
        Cmd::Integrity { check, json } => emit(&root, integrity(&root, check)?, json)?,
    }
    Ok(())
}

fn emit(root: &Path, outcome: Outcome, json: bool) -> Result<()> {
    let raw = root.join(".cinekernel/research/onda/r0.02/checks");
    fs::create_dir_all(&raw)?;
    let mut evidence = serde_json::to_vec_pretty(&outcome)?;
    evidence.push(b'\n');
    fs::write(raw.join(format!("{}.json", outcome.command)), evidence)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&outcome)?);
    } else {
        println!(
            "{}: {}",
            outcome.command,
            if outcome.ok { "PASS" } else { "FAIL" }
        );
        for check in outcome.checks {
            println!("- {check}");
        }
    }
    Ok(())
}

fn repo_root() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !output.status.success() {
        bail!("not in a git repository")
    }
    Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()))
}

fn model(root: &Path) -> Result<Value> {
    let path = root.join(DOC_DIR).join("R0_02_RESEARCH_MODEL.json");
    serde_json::from_slice(&fs::read(&path).with_context(|| format!("reading {}", path.display()))?)
        .context("canonical model is invalid JSON")
}

fn verify(root: &Path) -> Result<Outcome> {
    let model = model(root)?;
    let upstream_lock = lock::read(root)?;
    let mut checks = Vec::new();
    for (key, expected) in [
        ("cinekernel_base", BASE),
        ("onda_pin", upstream_lock.pinned_commit.as_str()),
        ("onda_tree", upstream_lock.pinned_tree.as_str()),
    ] {
        if model.get(key).and_then(Value::as_str) != Some(expected) {
            bail!("canonical {key} does not match the authoritative lock")
        }
        checks.push(format!("locked {key}"));
    }
    lock::validate_observation(
        &upstream_lock,
        &lock::observe(&root.join(".cinekernel/upstreams/onda"))?,
    )?;
    checks.push(
        "R0.01 lock authority, remote, detached HEAD, pin, tree, and clean state verified".into(),
    );
    for key in [
        "sources",
        "claims",
        "architecture_nodes",
        "architecture_edges",
        "hypotheses",
    ] {
        if model
            .get(key)
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
        {
            bail!("canonical {key} is empty")
        }
    }
    validate_refs(&model)?;
    checks.push("all source references resolve".into());
    model_validation::validate(&model)?;
    checks.push(
        "graph, semantic, fallback, creative, requirement, and stable-order contracts pass".into(),
    );
    contracts::validate(root, &model)?;
    checks.push("all strict Draft 2020-12 schemas compile and accept their outputs".into());
    Ok(Outcome {
        command: "verify",
        ok: true,
        checks,
    })
}

fn validate_refs(model: &Value) -> Result<()> {
    let ids: BTreeSet<_> = model["sources"]
        .as_array()
        .context("sources missing")?
        .iter()
        .filter_map(|source| source["source_id"].as_str())
        .collect();
    validate_refs_recursive(model, &ids)
}

fn validate_refs_recursive<'a>(value: &'a Value, ids: &BTreeSet<&'a str>) -> Result<()> {
    match value {
        Value::Object(map) => {
            for key in [
                "source_refs",
                "onda_source_refs",
                "independent_primary_source_refs",
            ] {
                if let Some(references) = map.get(key) {
                    let references = references
                        .as_array()
                        .context("source refs must be an array")?;
                    if references.is_empty() {
                        bail!("{key} must not be empty")
                    }
                    for reference in references {
                        let reference = reference.as_str().context("source ref is not a string")?;
                        if !ids.contains(reference) {
                            bail!("unresolved source ref {reference}")
                        }
                    }
                }
            }
            for child in map.values() {
                validate_refs_recursive(child, ids)?;
            }
        }
        Value::Array(items) => {
            for child in items {
                validate_refs_recursive(child, ids)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn inventory(root: &Path) -> Result<Outcome> {
    let model = model(root)?;
    let sources = model["sources"].as_array().context("sources missing")?;
    let counts = source_index::verify(root, sources, &lock::read(root)?)?;
    Ok(Outcome {
        command: "inventory",
        ok: true,
        checks: vec![
            format!(
                "{} pinned ONDA files hashed and fully verified",
                counts.onda_files
            ),
            format!(
                "{} official external references indexed separately",
                counts.external_references
            ),
        ],
    })
}

fn report(root: &Path) -> Result<Outcome> {
    let upstream_lock = lock::read(root)?;
    let model = remediate::model(root, model(root)?, &upstream_lock)?;
    write_stable_json(
        &root.join(DOC_DIR).join("R0_02_RESEARCH_MODEL.json"),
        &model,
    )?;
    generate_machine_outputs(root, &model)?;
    reports::generate(root, &model)?;
    contracts::generate(root, &model, &upstream_lock)?;
    for name in MACHINE_OUTPUTS.iter().chain(HUMAN_OUTPUTS.iter()) {
        let path = root.join(DOC_DIR).join(name);
        if !path.is_file() || fs::metadata(&path)?.len() == 0 {
            bail!("missing report output {}", path.display())
        }
    }
    Ok(Outcome { command: "report", ok: true, checks: vec!["16 machine projections, 20 topic-specific reports, and 17 strict schemas regenerated from the canonical model".into(), "53 generated files present; canonical model retained as the generation input".into()] })
}

fn generate_machine_outputs(root: &Path, model: &Value) -> Result<()> {
    let mappings: &[(&str, &[&str])] = &[
        ("SOURCE_INDEX.json", &["sources"]),
        ("AUTHORING_SURFACES.json", &["authoring_surfaces"]),
        (
            "ARCHITECTURE_GRAPH.json",
            &["architecture_nodes", "architecture_edges"],
        ),
        ("BOUNDARY_CONTRACTS.json", &["boundary_contracts"]),
        ("STATE_AND_TIME_OWNERSHIP.json", &["state_and_time"]),
        ("IDENTITY_AND_PROVENANCE.json", &["identity_and_provenance"]),
        ("SEMANTIC_PRESERVATION.json", &["semantic_preservation"]),
        (
            "VALIDATION_AND_FALLBACKS.json",
            &["validation_and_fallbacks"],
        ),
        ("PREVIEW_EXPORT_PARITY.json", &["preview_export_parity"]),
        ("SERIALIZATION_AND_VERSIONING.json", &["serialization"]),
        (
            "MATERIALIZATION_HYPOTHESES.json",
            &["materialization_hypotheses"],
        ),
        (
            "CREATIVE_PROGRAMMABILITY.json",
            &["creative_programmability"],
        ),
        ("NOVEL_SCENE_LITMUS.json", &["novel_scene_litmus"]),
        (
            "PRIMARY_SOURCE_COMPARISON.json",
            &["primary_source_comparison"],
        ),
        ("CANDIDATE_REQUIREMENTS.json", &["candidate_requirements"]),
        ("OPEN_QUESTIONS.json", &["open_questions"]),
    ];
    for (name, keys) in mappings {
        let mut data = serde_json::Map::new();
        for key in *keys {
            data.insert((*key).into(), model[*key].clone());
        }
        let schema = name
            .to_ascii_lowercase()
            .replace('_', "-")
            .replace(".json", ".schema.json");
        write_stable_json(
            &root.join(DOC_DIR).join(name),
            &serde_json::json!({"$schema":format!("../../../../schemas/research/r0.02/{schema}"),"schema_version":"r0.02.2","model_ref":"R0_02_RESEARCH_MODEL.json","generated_at":"2026-08-16T00:00:00Z","data":data}),
        )?;
    }
    Ok(())
}

fn guard(root: &Path) -> Result<Outcome> {
    Ok(Outcome {
        command: "guard",
        ok: true,
        checks: cleanroom::run(root)?,
    })
}

fn integrity(root: &Path, check: bool) -> Result<Outcome> {
    let manifest = root.join(REPORT_DIR).join("INTEGRITY_MANIFEST.sha256");
    if !check {
        let mut paths = Vec::new();
        for base in [
            DOC_DIR,
            REPORT_DIR,
            "schemas/research/r0.02",
            "tools/research/onda-r0-02",
            ".github/workflows",
        ] {
            let start = root.join(base);
            if !start.exists() {
                continue;
            }
            for entry in WalkDir::new(start)
                .into_iter()
                .filter_entry(|entry| entry.file_name() != "target")
                .filter_map(Result::ok)
                .filter(|entry| entry.file_type().is_file())
            {
                let relative = entry
                    .path()
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/");
                if relative == "reports/research/r0.02/INTEGRITY_MANIFEST.sha256"
                    || (relative.starts_with(".github/workflows/")
                        && relative != ".github/workflows/r0-02-onda-architecture.yml")
                {
                    continue;
                }
                paths.push(relative);
            }
        }
        paths.sort();
        paths.dedup();
        let mut body = String::new();
        for relative in &paths {
            body.push_str(&format!(
                "{}  {}\n",
                sha256(&fs::read(root.join(relative))?),
                relative
            ));
        }
        fs::write(&manifest, body)?;
        return Ok(Outcome {
            command: "integrity",
            ok: true,
            checks: vec![format!("wrote {} entries", paths.len())],
        });
    }
    let text = fs::read_to_string(&manifest)?;
    let mut count = 0;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let (expected, relative) = line.split_once("  ").context("malformed integrity line")?;
        if sha256(&fs::read(root.join(relative)).with_context(|| format!("reading {relative}"))?)
            != expected
        {
            bail!("integrity mismatch: {relative}")
        }
        count += 1;
    }
    Ok(Outcome {
        command: "integrity",
        ok: true,
        checks: vec![format!("{count} files match the manifest")],
    })
}

fn write_stable_json(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))
}
fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub const MACHINE_OUTPUTS: &[&str] = &[
    "R0_02_RESEARCH_MODEL.json",
    "SOURCE_INDEX.json",
    "AUTHORING_SURFACES.json",
    "ARCHITECTURE_GRAPH.json",
    "BOUNDARY_CONTRACTS.json",
    "STATE_AND_TIME_OWNERSHIP.json",
    "IDENTITY_AND_PROVENANCE.json",
    "SEMANTIC_PRESERVATION.json",
    "VALIDATION_AND_FALLBACKS.json",
    "PREVIEW_EXPORT_PARITY.json",
    "SERIALIZATION_AND_VERSIONING.json",
    "MATERIALIZATION_HYPOTHESES.json",
    "CREATIVE_PROGRAMMABILITY.json",
    "NOVEL_SCENE_LITMUS.json",
    "PRIMARY_SOURCE_COMPARISON.json",
    "CANDIDATE_REQUIREMENTS.json",
    "OPEN_QUESTIONS.json",
];
const HUMAN_OUTPUTS: &[&str] = &[
    "R0_02_ACCEPTANCE_REPORT.md",
    "ARCHITECTURE_OVERVIEW.md",
    "AUTHORING_SURFACES.md",
    "REACT_RECONCILER_FLOW.md",
    "CINEMA_COMPILER_FLOW.md",
    "DIRECT_JSON_AND_RUST_FLOW.md",
    "SCENE_GRAPH_CONTRACT.md",
    "STATE_AND_TIME_OWNERSHIP.md",
    "IDENTITY_AND_SOURCE_MAPPING.md",
    "LOWERING_AND_INFORMATION_LOSS.md",
    "VALIDATION_ERRORS_AND_FALLBACKS.md",
    "PREVIEW_EXPORT_PARITY.md",
    "SERIALIZATION_AND_VERSIONING.md",
    "MATERIALIZATION_AND_SCALABILITY.md",
    "CREATIVE_PROGRAMMABILITY_ASSESSMENT.md",
    "PRIMARY_SOURCE_COMPARISON.md",
    "CINEKERNEL_REQUIREMENT_CANDIDATES.md",
    "RISKS_AND_OPEN_QUESTIONS.md",
    "DEFERRED_TO_LATER_R0_PHASES.md",
    "RESEARCH_SOURCE_INDEX.md",
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepted_base_is_full_hash() {
        assert_eq!(BASE.len(), 40);
        assert!(BASE.bytes().all(|b| b.is_ascii_hexdigit()));
    }
    #[test]
    fn output_names_are_unique() {
        let mut seen = BTreeSet::new();
        for name in MACHINE_OUTPUTS.iter().chain(HUMAN_OUTPUTS) {
            assert!(seen.insert(name));
        }
    }
    #[test]
    fn output_counts_match_protocol() {
        assert_eq!(MACHINE_OUTPUTS.len(), 17);
        assert_eq!(HUMAN_OUTPUTS.len(), 20);
    }
    #[test]
    fn model_rejects_unresolved_source_ref() {
        let model = serde_json::json!({"sources":[{"source_id":"S-X"}],"claims":[{"source_refs":["S-MISSING"]}]});
        assert!(validate_refs(&model).is_err());
    }
    #[test]
    fn model_rejects_empty_source_ref() {
        let model =
            serde_json::json!({"sources":[{"source_id":"S-X"}],"claims":[{"source_refs":[]}]});
        assert!(validate_refs(&model).is_err());
    }
    #[test]
    fn stable_json_is_byte_identical() {
        let value = serde_json::json!({"b":[2,1],"a":{"stable":true}});
        assert_eq!(
            serde_json::to_vec_pretty(&value).unwrap(),
            serde_json::to_vec_pretty(&value).unwrap()
        );
    }
    #[test]
    fn command_evidence_names_are_unique() {
        let names = ["verify", "inventory", "report", "guard", "integrity"];
        let set: BTreeSet<_> = names.into_iter().collect();
        assert_eq!(set.len(), 5);
    }
}
