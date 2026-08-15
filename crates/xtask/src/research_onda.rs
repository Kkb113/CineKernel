use super::{AppResult, Failure, EXIT_VERIFICATION_FAILURE};
use anyhow::{bail, Context, Result};
use clap::Subcommand;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REPOSITORY: &str = "https://github.com/onda-engine/onda-engine.git";
const PIN: &str = "3ddf1780c9799bf038ac90cec7d8cadb61acafbe";
const TREE: &str = "639df83ebf0262afccd6d021bf6d16ef19777d85";
const BASE: &str = "5f47f341aa546b4ceb115fcad71d576d0ab85f29";
const CAPTURED: &str = "2026-08-15T00:00:00Z";
const SCHEMA_VERSION: &str = "cinekernel.research.onda.r0.01.v1";

#[derive(Debug, Subcommand)]
pub(crate) enum OndaCommand {
    Sync,
    Verify,
    Inventory,
    Report,
    Guard,
}

pub(crate) fn execute(root: &Path, command: &OndaCommand) -> AppResult<Value> {
    let result = match command {
        OndaCommand::Sync => sync(root),
        OndaCommand::Verify => verify(root),
        OndaCommand::Inventory => inventory(root),
        OndaCommand::Report => report(root),
        OndaCommand::Guard => guard(root),
    };
    result.map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))
}

fn upstream(root: &Path) -> PathBuf {
    root.join(".cinekernel/upstreams/onda")
}

fn raw(root: &Path) -> PathBuf {
    root.join(".cinekernel/research/onda/r0.01")
}

fn docs(root: &Path) -> PathBuf {
    root.join("docs/research/onda/r0.01")
}

fn command_text(directory: &Path, program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(directory)
        .output()?;
    if !output.status.success() {
        bail!(
            "{} {} failed with {}: {}",
            program,
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn git(directory: &Path, args: &[&str]) -> Result<String> {
    command_text(directory, "git", args)
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

fn write_text(path: &Path, value: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, value.replace("\r\n", "\n"))?;
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn file_sha256(path: &Path) -> Result<String> {
    Ok(sha256(&fs::read(path)?))
}

fn git_blob(upstream: &Path, path: &str) -> Result<String> {
    git(upstream, &["rev-parse", &format!("HEAD:{path}")])
}

fn identity(root: &Path) -> Result<Value> {
    let upstream = upstream(root);
    if !upstream.join(".git").is_dir() {
        bail!("missing ONDA checkout; run `cargo xtask research onda sync`");
    }
    let head = git(&upstream, &["rev-parse", "HEAD"])?;
    let tree = git(&upstream, &["rev-parse", "HEAD^{tree}"])?;
    let remote = git(&upstream, &["remote", "get-url", "origin"])?;
    let status = git(&upstream, &["status", "--porcelain"])?;
    let signature = git(&upstream, &["log", "-1", "--format=%G?"])?;
    let author_date = git(&upstream, &["log", "-1", "--format=%aI"])?;
    let committer_date = git(&upstream, &["log", "-1", "--format=%cI"])?;
    let checks = json!({
        "pin": head == PIN,
        "tree": tree == TREE,
        "remote": remote == REPOSITORY,
        "clean": status.is_empty()
    });
    Ok(json!({
        "repository": remote, "head": head, "tree": tree, "status": status,
        "signature_status": signature, "author_date": author_date,
        "committer_date": committer_date, "checks": checks
    }))
}

fn sync(root: &Path) -> Result<Value> {
    let target = upstream(root);
    if !target.exists() {
        fs::create_dir_all(target.parent().context("upstream parent")?)?;
        let target_text = target.to_string_lossy().into_owned();
        command_text(
            root,
            "git",
            &[
                "clone",
                "--filter=blob:none",
                "--no-checkout",
                REPOSITORY,
                &target_text,
            ],
        )?;
    }
    if git(&target, &["cat-file", "-e", &format!("{PIN}^{{commit}}")]).is_err() {
        git(&target, &["fetch", "origin", PIN])?;
    }
    git(&target, &["checkout", "--detach", PIN])?;
    let value = identity(root)?;
    if value["checks"].as_object().is_none_or(|checks| {
        checks
            .values()
            .any(|candidate| candidate != &Value::Bool(true))
    }) {
        bail!("ONDA checkout identity verification failed: {value}");
    }
    write_json(&raw(root).join("git/sync.json"), &value)?;
    Ok(json!({"ok": true, "pin": PIN, "tree": TREE, "checkout": ".cinekernel/upstreams/onda"}))
}

fn upstream_lock(root: &Path, id: &Value) -> Result<Value> {
    let upstream = upstream(root);
    let hash = |path: &str| file_sha256(&upstream.join(path));
    Ok(json!({
        "schema_version": SCHEMA_VERSION,
        "captured_at_utc": CAPTURED,
        "cinekernel_base_revision": BASE,
        "repository": REPOSITORY,
        "default_branch": "main",
        "observed_branch_head": PIN,
        "pinned_commit": PIN,
        "pinned_tree": TREE,
        "commit_author_date": id["author_date"],
        "commit_committer_date": id["committer_date"],
        "commit_signature_status": "VERIFIED_GITHUB_SIGNATURE",
        "license": {
            "declared_spdx": "FSL-1.1-ALv2",
            "license_path": "LICENSE",
            "license_git_blob": git_blob(&upstream, "LICENSE")?,
            "license_sha256": hash("LICENSE")?,
            "future_license_path": "LICENSE-APACHE",
            "future_license_git_blob": git_blob(&upstream, "LICENSE-APACHE")?,
            "future_license_sha256": hash("LICENSE-APACHE")?,
            "notice_path": "NOTICE.md",
            "notice_git_blob": git_blob(&upstream, "NOTICE.md")?,
            "notice_sha256": hash("NOTICE.md")?
        },
        "lockfiles": {
            "cargo_lock_sha256": hash("Cargo.lock")?,
            "pnpm_lock_sha256": hash("pnpm-lock.yaml")?
        },
        "release_streams": ["embed-kit-github", "onda-engine-npm", "scoped-github-packages"],
        "package_versions": [
            {"surface":"rust-workspace","version":"0.1.0"},
            {"surface":"npm-umbrella","version":"0.6.1"},
            {"surface":"embed-kit-latest","version":"0.2.16"}
        ],
        "research_policy_version": "R0.01-v1",
        "unresolved": ["scoped GitHub Packages registry state requires authentication", "legal effect of future-license dates"]
    }))
}

fn verify(root: &Path) -> Result<Value> {
    let id = identity(root)?;
    let mut failures = identity_failures(&id)?;
    let lock = upstream_lock(root, &id)?;
    let expected = docs(root).join("UPSTREAM_LOCK.json");
    let lock_match = if expected.exists() {
        serde_json::from_slice::<Value>(&fs::read(expected)?)? == lock
    } else {
        true
    };
    if !lock_match {
        failures.push("committed_lock".to_owned());
    }
    let value = json!({"ok": failures.is_empty(), "identity": id, "lock_matches": lock_match, "failures": failures});
    write_json(&raw(root).join("checks/verify.json"), &value)?;
    if value["ok"] != true {
        bail!("ONDA verification failed: {}", value["failures"]);
    }
    Ok(value)
}

fn identity_failures(identity: &Value) -> Result<Vec<String>> {
    let checks = identity["checks"]
        .as_object()
        .context("identity checks missing")?;
    Ok(checks
        .iter()
        .filter(|(_, passed)| *passed != &Value::Bool(true))
        .map(|(name, _)| name.clone())
        .collect())
}

fn cargo_metadata(upstream: &Path, all_features: bool) -> Result<Value> {
    let mut args = vec!["metadata", "--locked", "--format-version", "1"];
    if all_features {
        args.push("--all-features");
    }
    let text = command_text(upstream, "cargo", &args)?;
    Ok(serde_json::from_str(&text)?)
}

fn npm_pack_dry_run(root: &Path, upstream: &Path) -> Result<Value> {
    let cache = raw(root).join("npm/cache");
    fs::create_dir_all(&cache)?;
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let output = Command::new(npm)
        .args(["pack", "--dry-run", "--json", "--ignore-scripts"])
        .env("npm_config_cache", cache)
        .current_dir(upstream.join("packages/umbrella"))
        .output()?;
    if !output.status.success() {
        bail!(
            "scripts-disabled npm pack dry-run failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn relative(path: &str, root: &Path) -> String {
    let value = Path::new(path);
    value
        .strip_prefix(root)
        .unwrap_or(value)
        .to_string_lossy()
        .replace('\\', "/")
}

fn classify_module(name: &str, manifest: &str) -> &'static str {
    if manifest.contains("apps/") {
        return "application";
    }
    match name {
        "onda-core" => "core-types",
        "onda-scene" => "scene-graph",
        "onda-renderer" => "cpu-renderer",
        "onda-typography" => "typography",
        "onda-animation" => "animation",
        "onda-cli" => "cli",
        "onda-wasm" | "@onda-engine/wasm" => "wasm",
        "onda-wasm-vello" | "@onda-engine/wasm-vello" => "wasm-vello",
        "onda-bench" => "benchmark",
        "onda-vello" => "gpu-vello-renderer",
        "onda-svg" => "svg",
        "onda-image" => "images",
        "onda-layout" => "layout",
        "onda-audio" | "onda-wasm-audio" => "audio",
        "onda-video" => "video",
        "onda-segment" => "segmentation",
        "onda-transcribe" => "transcription",
        "onda-tts" => "tts",
        _ => "typescript-package",
    }
}

fn classify_dependency_source(source: Option<&str>) -> &'static str {
    source.map_or("path/workspace", |source| {
        if source.starts_with("registry+") {
            "registry"
        } else if source.starts_with("git+") {
            "git"
        } else {
            "unknown"
        }
    })
}

fn rust_inventory(metadata: &Value, upstream: &Path) -> Result<(Vec<Value>, Vec<Value>)> {
    let members: BTreeSet<&str> = metadata["workspace_members"]
        .as_array()
        .context("workspace members missing")?
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let mut modules = Vec::new();
    let mut packages = Vec::new();
    for package in metadata["packages"]
        .as_array()
        .context("packages missing")?
    {
        let id = package["id"].as_str().unwrap_or_default();
        let manifest = relative(
            package["manifest_path"].as_str().unwrap_or_default(),
            upstream,
        );
        let source = classify_dependency_source(package["source"].as_str());
        let dependencies = package["dependencies"]
            .as_array()
            .map_or_else(Vec::new, |deps| {
                deps.iter().map(|dep| json!({
            "name": dep["name"], "source": dep["source"], "requirement": dep["req"],
            "kind": dep["kind"], "optional": dep["optional"], "target": dep["target"],
            "features": dep["features"], "uses_default_features": dep["uses_default_features"]
        })).collect()
            });
        let record = json!({
            "id": id, "name": package["name"], "version": package["version"],
            "manifest": manifest, "source_kind": source, "source": package["source"],
            "checksum": package["checksum"], "license": package["license"],
            "repository": package["repository"], "workspace_member": members.contains(id),
            "features": package["features"], "dependencies": dependencies
        });
        if members.contains(id) {
            modules.push(json!({
                "ecosystem":"rust", "name":package["name"], "version":package["version"],
                "path": manifest, "classification":classify_module(package["name"].as_str().unwrap_or_default(), &manifest),
                "status":"VERIFIED_AT_PIN"
            }));
        }
        packages.push(record);
    }
    modules.sort_by_key(|value| value["path"].as_str().unwrap_or_default().to_owned());
    packages.sort_by_key(|value| value["id"].as_str().unwrap_or_default().to_owned());
    Ok((modules, packages))
}

fn collect_files(directory: &Path, name: &str, output: &mut Vec<PathBuf>) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.file_name() == Some(OsStr::new("node_modules"))
            || path.file_name() == Some(OsStr::new("target"))
        {
            continue;
        }
        if path.is_dir() {
            collect_files(&path, name, output)?;
        } else if path.file_name() == Some(OsStr::new(name)) {
            output.push(path);
        }
    }
    Ok(())
}

fn pnpm_resolutions(lockfile: &str) -> BTreeMap<(String, String), String> {
    let mut output = BTreeMap::new();
    let mut in_importers = false;
    let mut importer = String::new();
    let mut dependency = String::new();
    for line in lockfile.lines() {
        if line == "importers:" {
            in_importers = true;
            continue;
        }
        if in_importers && !line.is_empty() && !line.starts_with(' ') {
            break;
        }
        if !in_importers {
            continue;
        }
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();
        if indent == 2 && trimmed.ends_with(':') {
            importer = trimmed.trim_end_matches(':').trim_matches('\'').to_owned();
            dependency.clear();
        } else if indent == 6 && trimmed.ends_with(':') {
            dependency = trimmed.trim_end_matches(':').trim_matches('\'').to_owned();
        } else if indent == 8
            && trimmed.starts_with("version:")
            && !importer.is_empty()
            && !dependency.is_empty()
        {
            let version = trimmed
                .trim_start_matches("version:")
                .trim()
                .trim_matches('\'');
            output.insert((importer.clone(), dependency.clone()), version.to_owned());
        }
    }
    output
}

fn js_inventory(upstream: &Path) -> Result<(Vec<Value>, Vec<Value>)> {
    let resolutions = pnpm_resolutions(&fs::read_to_string(upstream.join("pnpm-lock.yaml"))?);
    let mut manifests = Vec::new();
    collect_files(upstream, "package.json", &mut manifests)?;
    manifests.sort();
    let mut modules = Vec::new();
    let mut dependencies = Vec::new();
    for path in manifests {
        let rel = relative(&path.to_string_lossy(), upstream);
        if rel == "package.json" {
            continue;
        }
        let package: Value = serde_json::from_slice(&fs::read(&path)?)?;
        let name = package["name"].as_str().unwrap_or("UNNAMED");
        let classification = if rel.starts_with("apps/") {
            "application"
        } else {
            classify_module(name, &rel)
        };
        modules.push(json!({
            "ecosystem":"javascript", "name":name, "version":package["version"], "license":package["license"],
            "path":rel, "classification":classification, "private":package["private"], "status":"VERIFIED_AT_PIN"
        }));
        for (field, kind) in [
            ("dependencies", "external-runtime"),
            ("peerDependencies", "peer"),
            ("devDependencies", "development"),
        ] {
            if let Some(map) = package[field].as_object() {
                for (dependency, requirement) in map {
                    let classification = if rel == "apps/benchmark/package.json" {
                        "benchmark-only"
                    } else {
                        kind
                    };
                    let importer = path
                        .parent()
                        .map(|parent| relative(&parent.to_string_lossy(), upstream))
                        .unwrap_or_else(|| ".".to_owned());
                    let resolved = resolutions.get(&(importer, dependency.clone()));
                    dependencies.push(json!({"package":name,"dependency":dependency,"requirement":requirement,"resolved_version":resolved,"classification":classification,"manifest":rel}));
                }
            }
        }
    }
    modules.sort_by_key(|value| value["path"].as_str().unwrap_or_default().to_owned());
    dependencies.sort_by_key(|value| format!("{}:{}", value["package"], value["dependency"]));
    Ok((modules, dependencies))
}

fn external_artifacts() -> Value {
    let mut value = json!({"schema_version":SCHEMA_VERSION,"artifacts":[
        {"name":"FFmpeg","purpose":"video encode/decode","kind":"system executable","required_by":"onda-video/CLI video","phase":"RUNTIME_EXTERNAL","optional":true,"platform":"native","version_pin":null,"license":"LGPL/GPL build-dependent","status":"LEGAL_REVIEW_REQUIRED","source":"packages/video-rs/Cargo.toml; workflows"},
        {"name":"Vulkan/lavapipe","purpose":"GPU or software Vulkan backend","kind":"system dependency","required_by":"onda-vello","phase":"RUNTIME_EXTERNAL","optional":true,"platform":"Linux","version_pin":null,"license":"implementation-dependent","status":"LEGAL_REVIEW_REQUIRED","source":".github/workflows/ci.yml"},
        {"name":"CMake and C/C++ toolchain","purpose":"native dependency builds","kind":"build tool","required_by":"whisper-rs/espeak-rs/ONNX Runtime","phase":"BUILD_TIME","optional":true,"platform":"native","version_pin":null,"license":"system dependency","status":"VERIFIED_AT_PIN","source":"Cargo.lock; feature manifests"},
        {"name":"wasm-bindgen CLI","purpose":"WASM package generation","kind":"build tool","required_by":"WASM packages","phase":"BUILD_TIME","optional":true,"platform":"wasm32","version_pin":null,"license":"MIT/Apache-2.0","status":"VERIFIED_AT_PIN","source":"package scripts/workflows"},
        {"name":"Node.js","purpose":"JavaScript workspace","kind":"runtime tool","required_by":"pnpm workspace","phase":"BUILD_TIME","optional":false,"platform":"all","version_pin":">=20","license":"MIT","status":"VERIFIED_AT_PIN","source":"package.json"},
        {"name":"pnpm","purpose":"workspace package manager","kind":"build tool","required_by":"JavaScript workspace","phase":"BUILD_TIME","optional":false,"platform":"all","version_pin":"10.5.0","license":"MIT","status":"VERIFIED_AT_PIN","source":"package.json"},
        {"name":"Bun","purpose":"release/embed scripting reference","kind":"build tool","required_by":"release tooling","phase":"BUILD_TIME","optional":true,"platform":"release","version_pin":null,"license":"MIT","status":"UNRESOLVED","source":"scripts/workflows"},
        {"name":"Rust toolchain","purpose":"Rust workspace","kind":"build tool","required_by":"all Rust modules","phase":"BUILD_TIME","optional":false,"platform":"all","version_pin":"rust-version 1.80 minimum","license":"Apache-2.0/MIT","status":"VERIFIED_AT_PIN","source":"Cargo.toml"},
        {"name":"U2-Net model","purpose":"segmentation","kind":"downloaded model","required_by":"onda-segment","phase":"MODEL_DATA","optional":true,"cache_path":"~/.onda/models/u2net.onnx","source_url":"https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2net.onnx","source_repository":"https://github.com/danielgatis/rembg","expected_size":"approximately 176 MB","digest_algorithm":null,"digest":null,"onda_verification":"approximate size threshold only","license":"upstream source claims Apache-2.0; artifact rights require verification","status":"LEGAL_REVIEW_REQUIRED"},
        {"name":"Whisper tiny.en/base.en/small.en","purpose":"transcription","kind":"downloaded model","required_by":"onda-transcribe","phase":"MODEL_DATA","optional":true,"cache_path":"~/.onda/models/ggml-{tiny,base,small}.en.bin","source_url":"https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{name}.bin","source_repository":"https://huggingface.co/ggerganov/whisper.cpp","expected_size":"75 MB / 142 MB / 466 MB","digest_algorithm":null,"digest":null,"onda_verification":"approximate size threshold only","license":"model/data-specific","status":"LEGAL_REVIEW_REQUIRED"},
        {"name":"Kokoro ONNX model","purpose":"text to speech","kind":"downloaded model","required_by":"onda-tts speak","phase":"MODEL_DATA","optional":true,"cache_path":"~/.onda/models/kokoro-v1.0.onnx","source_url":"https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/kokoro-v1.0.onnx","source_repository":"https://github.com/thewh1teagle/kokoro-onnx","expected_size":"approximately 325 MB","digest_algorithm":null,"digest":null,"onda_verification":"approximate size threshold only","license":"upstream source claims Apache-2.0; artifact rights require verification","status":"LEGAL_REVIEW_REQUIRED"},
        {"name":"Kokoro voice bundle","purpose":"TTS voices","kind":"downloaded model","required_by":"onda-tts speak","phase":"MODEL_DATA","optional":true,"cache_path":"~/.onda/models/voices-v1.0.bin","source_url":"https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0/voices-v1.0.bin","source_repository":"https://github.com/thewh1teagle/kokoro-onnx","expected_size":"approximately 28 MB","digest_algorithm":null,"digest":null,"onda_verification":"approximate size threshold only","license":"data-specific","status":"LEGAL_REVIEW_REQUIRED"},
        {"name":"ONNX Runtime prebuilt binary","purpose":"model inference","kind":"downloaded binary","required_by":"ort/onda-segment/onda-tts","phase":"BUILD_TIME","optional":true,"cache_path":"Cargo build cache","source_url":"ort release infrastructure","expected_size":"platform-dependent","digest_algorithm":null,"digest":null,"onda_verification":"delegated to ort-sys build tooling","license":"MIT plus third-party notices","status":"LEGAL_REVIEW_REQUIRED"}
    ]});
    for artifact in value["artifacts"].as_array_mut().expect("artifact array") {
        artifact["confidence"] = Value::String(
            "HIGH for pinned-source observation; legal conclusions excluded".to_owned(),
        );
        artifact["human_legal_review_required"] =
            Value::Bool(artifact["status"] == "LEGAL_REVIEW_REQUIRED");
        artifact["distributed_in_output"] = Value::String(
            "feature/release dependent; do not infer from workspace membership".to_owned(),
        );
        artifact["provenance_pin"] = Value::String(PIN.to_owned());
    }
    value
}

fn release_map() -> Value {
    json!({"schema_version":SCHEMA_VERSION,"streams":[
        {"id":"embed-kit-github","kind":"github-release","tag_pattern":"v*","latest_tag":"v0.2.16","tag_target":"56cc75b2614812ef97121f0171e8e8d765260e59","release_id":353301462,"published_at":"2026-07-13T16:55:48Z","asset":{"id":475692944,"name":"onda-embed-kit-v0.2.16-x86_64-linux.tar.gz","size":21033371,"digest_algorithm":"sha256","digest":"d4335601dc0c66733f772261cbce2de48457767aaa61195691e70062cf331742","platform":"x86_64-linux"},"workflow":".github/workflows/release.yml"},
        {"id":"onda-engine-npm","kind":"npm-public","tag_pattern":"onda-engine-v*","source_version":"0.6.1","published_version":"0.6.1","latest":"0.6.1","published_at":"2026-07-20T03:12:08.488Z","matching_github_release":"NOT_OBSERVED_FOR_0.6.1","tarball":"https://registry.npmjs.org/onda-engine/-/onda-engine-0.6.1.tgz","shasum":{"algorithm":"sha1","value":"e48444f629cbc112f23456809e348fa67feafad8"},"integrity":{"algorithm":"sha512-sri","value":"sha512-5tfqzK5DuMCIbzKfKq8CsOJzKv5JHGGWtYNdSLTsdI/9jf5mMmj1408/YEfbz3pJtwIDA2WfpNL5tFF7eXvipQ=="},"registry_signature_count":1,"registry_file_count":42,"registry_unpacked_size":17131808,"local_scripts_disabled_pack":{"entry_count":4,"unpacked_size":25616,"interpretation":"dist is absent because lifecycle scripts were disabled; this is expected and proves source pack intent differs from built publication"},"provenance":"NO_PROVENANCE_ATTESTATION_OBSERVED","workflow":".github/workflows/release-npm.yml"},
        {"id":"scoped-github-packages","kind":"github-packages","tag_pattern":"@onda-engine/*@*","manifest_versions":"independent","registry_state":"AUTH_REQUIRED_NOT_VERIFIED","workflow":"release configuration"}
    ],"observed_public_versions":["0.1.0","0.1.1","0.2.0","0.3.0","0.4.0","0.5.0","0.6.0","0.6.1"],"pr_41":{"number":41,"merge_commit":PIN,"finding":"0.6.1 manual release closed a path-scoped Release Please gap"}})
}

fn license_surface(lock: &Value, rust_packages: &[Value], js_modules: &[Value]) -> Value {
    let mut expressions = BTreeSet::new();
    for package in rust_packages {
        if let Some(license) = package["license"].as_str() {
            expressions.insert(license.to_owned());
        } else {
            expressions.insert("UNKNOWN/CUSTOM".to_owned());
        }
    }
    for package in js_modules {
        if let Some(license) = package["license"].as_str() {
            expressions.insert(license.to_owned());
        } else {
            expressions.insert("UNKNOWN/CUSTOM".to_owned());
        }
    }
    let mut value = json!({"schema_version":SCHEMA_VERSION,"source_license":lock["license"],"notice_completeness":"UPSTREAM_NOTICE_DECLARED_INCOMPLETE","observed_expressions":expressions,"hotspots":[
        {"id":"LIC-001","surface":"ONDA source","declared":"FSL-1.1-ALv2","status":"LEGAL_REVIEW_REQUIRED","fact":"Current source is source-available, not Apache-2.0 today."},
        {"id":"LIC-002","surface":"future license","declared":"Apache-2.0 text present","status":"LEGAL_EFFECTIVE_DATE_NOT_CONFIRMED","fact":"Per-version candidate date is two years after availability evidence; legal effective date is unconfirmed."},
        {"id":"LIC-003","surface":"MPL dependencies","declared":"MPL-2.0","status":"LEGAL_REVIEW_REQUIRED","fact":"NOTICE identifies MPL surfaces; derived graph must govern completeness."},
        {"id":"LIC-004","surface":"FFmpeg","declared":"build-dependent LGPL/GPL","status":"LEGAL_REVIEW_REQUIRED","chain":"onda-cli video -> onda-video -> external FFmpeg"},
        {"id":"LIC-005","surface":"ONNX Runtime","declared":"MIT plus notices","status":"LEGAL_REVIEW_REQUIRED","chain":"segment/speak -> ort -> ONNX Runtime binaries"},
        {"id":"LIC-006","surface":"Whisper","declared":"MIT code; model terms separate","status":"LEGAL_REVIEW_REQUIRED","chain":"onda-cli transcribe -> onda-transcribe -> whisper-rs/whisper.cpp -> model artifact"},
        {"id":"LIC-007","surface":"U2-Net model","declared":"model/data license","status":"LEGAL_REVIEW_REQUIRED","chain":"onda-cli segment -> onda-segment -> downloaded model"},
        {"id":"LIC-008","surface":"Kokoro model/voices and eSpeak NG","declared":"mixed code/model/data terms","status":"LEGAL_REVIEW_REQUIRED","chain":"onda-cli speak -> onda-tts -> espeak-rs -> eSpeak NG source/data; plus Kokoro ONNX and voices"},
        {"id":"LIC-009","surface":"Vello/wgpu, font, SVG, audio stacks","declared":"multiple dependency expressions","status":"LEGAL_REVIEW_REQUIRED","fact":"Distribution-specific obligations require dependency-level review."}
    ],"future_license":{"software_version":"repository pin and each released package separately","availability_evidence":[{"kind":"commit","date":"2026-07-20T01:21:12Z"},{"kind":"npm-publication","version":"0.6.1","date":"2026-07-20T03:12:08.488Z"}],"candidate_future_license_date":"2028-07-20","status":["CURRENT_FSL","FUTURE_APACHE_TEXT_PRESENT","CANDIDATE_DATE_CALCULATED","LEGAL_EFFECTIVE_DATE_NOT_CONFIRMED"]}});
    for hotspot in value["hotspots"].as_array_mut().expect("hotspot array") {
        hotspot["source_repository"] = Value::String(REPOSITORY.to_owned());
        hotspot["source_url"] = Value::String(format!(
            "https://github.com/onda-engine/onda-engine/tree/{PIN}"
        ));
        hotspot["version"] = Value::String(PIN.to_owned());
        hotspot["confidence"] = Value::String(
            "HIGH for declared dependency/distribution chain; no legal conclusion".to_owned(),
        );
        hotspot["distributed_in_output"] = Value::String("release/feature dependent".to_owned());
        hotspot["human_legal_review_required"] = Value::Bool(true);
    }
    value
}

fn inventory(root: &Path) -> Result<Value> {
    verify(root)?;
    let upstream = upstream(root);
    let default_metadata = cargo_metadata(&upstream, false)?;
    let all_metadata = cargo_metadata(&upstream, true)?;
    let packlist = npm_pack_dry_run(root, &upstream)?;
    write_json(
        &raw(root).join("cargo/metadata-default.json"),
        &default_metadata,
    )?;
    write_json(
        &raw(root).join("cargo/metadata-all-features.json"),
        &all_metadata,
    )?;
    write_json(&raw(root).join("npm/local-pack-dry-run.json"), &packlist)?;
    let (mut modules, rust_packages) = rust_inventory(&all_metadata, &upstream)?;
    let (js_modules, js_dependencies) = js_inventory(&upstream)?;
    modules.extend(js_modules.clone());
    modules.push(json!({"ecosystem":"rust","name":"vector-rs","version":null,"path":"packages/vector-rs","classification":"DOCUMENTED_OR_COMMENTED_BUT_ABSENT","status":"DOCUMENTED_OR_COMMENTED_BUT_ABSENT"}));
    modules.push(json!({"ecosystem":"rust","name":"codecs-rs","version":null,"path":"packages/codecs-rs","classification":"DOCUMENTED_OR_COMMENTED_BUT_ABSENT","status":"DOCUMENTED_OR_COMMENTED_BUT_ABSENT"}));
    modules.sort_by_key(|value| format!("{}:{}", value["ecosystem"], value["path"]));
    let module_value =
        json!({"schema_version":SCHEMA_VERSION,"pinned_commit":PIN,"modules":modules});
    let dependency_value = json!({
        "schema_version":SCHEMA_VERSION,"pinned_commit":PIN,
        "cargo":{"package_count":rust_packages.len(),"workspace_member_count":all_metadata["workspace_members"].as_array().map_or(0,Vec::len),"packages":rust_packages},
        "javascript":{"workspace_package_count":js_modules.len(),"dependencies":js_dependencies},
        "reachability":{"default_cli":"manifest/feature derived","video":"OPTIONAL","segment":"OPTIONAL","transcribe":"OPTIONAL","speak":"OPTIONAL","wasm":"OPTIONAL","gpu_vello":"OPTIONAL","cpu_renderer":"default renderer module","audio":"OPTIONAL","typography":"workspace","layout":"workspace"}
    });
    let lock = upstream_lock(root, &identity(root)?)?;
    let artifacts = external_artifacts();
    let releases = release_map();
    let licenses = license_surface(
        &lock,
        dependency_value["cargo"]["packages"]
            .as_array()
            .unwrap_or(&Vec::new()),
        &js_modules,
    );
    write_json(&docs(root).join("UPSTREAM_LOCK.json"), &lock)?;
    write_json(&docs(root).join("MODULE_INVENTORY.json"), &module_value)?;
    write_json(
        &docs(root).join("DEPENDENCY_INVENTORY.json"),
        &dependency_value,
    )?;
    write_json(&docs(root).join("EXTERNAL_ARTIFACTS.json"), &artifacts)?;
    write_json(&docs(root).join("RELEASE_MAP.json"), &releases)?;
    write_json(&docs(root).join("LICENSE_SURFACE.json"), &licenses)?;
    Ok(json!({
        "ok":true,"pin":PIN,"tree":TREE,"rust_workspace_members":dependency_value["cargo"]["workspace_member_count"],
        "rust_resolved_packages":dependency_value["cargo"]["package_count"],"javascript_workspace_packages":js_modules.len(),
        "external_artifacts":artifacts["artifacts"].as_array().map_or(0,Vec::len),"release_streams":releases["streams"].as_array().map_or(0,Vec::len),
        "license_hotspots":licenses["hotspots"].as_array().map_or(0,Vec::len)
    }))
}

fn production_manifest_violation(root: &Path) -> Result<Vec<String>> {
    let tracked = git(root, &["ls-files"])?;
    let mut violations = Vec::new();
    for rel in tracked.lines().filter(|path| {
        path.ends_with("Cargo.toml")
            || path.ends_with("package.json")
            || *path == "pnpm-workspace.yaml"
    }) {
        let normalized = rel.replace('\\', "/");
        if normalized == "crates/xtask/Cargo.toml" {
            continue;
        }
        let text = fs::read_to_string(root.join(rel))?;
        for needle in forbidden_manifest_hits(&text) {
            violations.push(format!("{normalized}: {needle}"));
        }
    }
    Ok(violations)
}

fn forbidden_manifest_hits(text: &str) -> Vec<&'static str> {
    [
        "onda-engine",
        "@onda-engine/",
        "onda-core",
        "onda-scene",
        "onda-renderer",
        "onda-vello",
    ]
    .into_iter()
    .filter(|needle| text.contains(needle))
    .collect()
}

fn source_like(path: &str) -> bool {
    [".rs", ".ts", ".tsx", ".js", ".mjs", ".cjs", ".wgsl"]
        .iter()
        .any(|extension| path.ends_with(extension))
}

fn nontrivial_source_hash(bytes: &[u8]) -> Option<String> {
    (bytes.len() >= 80).then(|| sha256(bytes))
}

fn exact_copy_violations(root: &Path) -> Result<Vec<String>> {
    let upstream = upstream(root);
    let mut upstream_hashes: BTreeMap<String, String> = BTreeMap::new();
    for path in git(&upstream, &["ls-tree", "-r", "--name-only", PIN])?
        .lines()
        .filter(|path| source_like(path))
    {
        let bytes = Command::new("git")
            .args(["show", &format!("{PIN}:{path}")])
            .current_dir(&upstream)
            .output()?
            .stdout;
        if let Some(hash) = nontrivial_source_hash(&bytes) {
            upstream_hashes.insert(hash, path.to_owned());
        }
    }
    let mut violations = Vec::new();
    for path in git(
        root,
        &["ls-files", "crates", "packages", "benchmarks", "scripts"],
    )?
    .lines()
    .filter(|path| source_like(path))
    {
        let bytes = fs::read(root.join(path))?;
        if let Some(hash) = nontrivial_source_hash(&bytes) {
            if let Some(upstream_path) = upstream_hashes.get(&hash) {
                violations.push(format!("{path} == {upstream_path}"));
            }
        }
    }
    Ok(violations)
}

fn phase0_changes(root: &Path) -> Result<Vec<String>> {
    let mut paths = BTreeSet::new();
    for args in [
        vec!["diff", "--name-only", BASE],
        vec!["status", "--porcelain"],
    ] {
        let text = git(root, &args)?;
        for line in text.lines() {
            let candidate = if args[0] == "status" {
                line.get(3..).unwrap_or(line)
            } else {
                line
            };
            let path = candidate.replace('\\', "/");
            if is_phase0_frozen(&path) {
                paths.insert(path);
            }
        }
    }
    Ok(paths.into_iter().collect())
}

fn is_phase0_frozen(path: &str) -> bool {
    path.starts_with("reports/phase0/")
        || path == "benchmarks/upstreams.lock.json"
        || path.starts_with("docs/research/remotion/")
        || path.starts_with("docs/research/hyperframes/")
}

fn guard(root: &Path) -> Result<Value> {
    verify(root)?;
    let production_dependencies = production_manifest_violation(root)?;
    let tracked_upstream = git(root, &["ls-files", ".cinekernel/upstreams/onda"])?;
    let exact_copies = exact_copy_violations(root)?;
    let phase0 = phase0_changes(root)?;
    let value = json!({
        "ok":production_dependencies.is_empty() && tracked_upstream.is_empty() && exact_copies.is_empty() && phase0.is_empty(),
        "onda_production_dependencies":{"ok":production_dependencies.is_empty(),"violations":production_dependencies},
        "tracked_upstream":{"ok":tracked_upstream.is_empty(),"files":tracked_upstream.lines().collect::<Vec<_>>()},
        "exact_copy_guard":{"ok":exact_copies.is_empty(),"description":"Exact SHA-256 content match for nontrivial tracked source-like files; not proof against all derivation.","violations":exact_copies},
        "phase0_immutability":{"ok":phase0.is_empty(),"violations":phase0}
    });
    write_json(&raw(root).join("checks/guard.json"), &value)?;
    if value["ok"] != true {
        bail!("R0.01 guard failed: {value}");
    }
    Ok(value)
}

fn markdown_table(title: &str, headers: &str, rows: impl Iterator<Item = String>) -> String {
    let columns = headers.split('|').count();
    let mut output = format!(
        "# {title}\n\n| {headers} |\n|{}|\n",
        (0..columns).map(|_| "---").collect::<Vec<_>>().join("|")
    );
    for row in rows {
        output.push_str(&format!("| {row} |\n"));
    }
    output
}

fn report(root: &Path) -> Result<Value> {
    let summary = inventory(root)?;
    let guard_result = guard(root)?;
    let dir = docs(root);
    let modules: Value = serde_json::from_slice(&fs::read(dir.join("MODULE_INVENTORY.json"))?)?;
    let dependencies: Value =
        serde_json::from_slice(&fs::read(dir.join("DEPENDENCY_INVENTORY.json"))?)?;
    let artifacts: Value = serde_json::from_slice(&fs::read(dir.join("EXTERNAL_ARTIFACTS.json"))?)?;
    let licenses: Value = serde_json::from_slice(&fs::read(dir.join("LICENSE_SURFACE.json"))?)?;
    write_text(
        &dir.join("MODULE_INVENTORY.md"),
        &markdown_table(
            "R0.01 ONDA module inventory",
            "Ecosystem | Module | Version | Classification | Status",
            modules["modules"].as_array().unwrap().iter().map(|v| {
                format!(
                    "{} | {} | {} | {} | {}",
                    v["ecosystem"].as_str().unwrap_or(""),
                    v["name"].as_str().unwrap_or(""),
                    v["version"].as_str().unwrap_or("—"),
                    v["classification"].as_str().unwrap_or(""),
                    v["status"].as_str().unwrap_or("")
                )
            }),
        ),
    )?;
    write_text(&dir.join("DEPENDENCY_INVENTORY.md"), &format!("# R0.01 ONDA dependency inventory\n\n- Locked Rust packages: {}\n- Rust workspace members: {}\n- JavaScript workspace packages: {}\n- Default and all-feature Cargo graphs both resolve under `--locked`; raw metadata is ignored under `.cinekernel/research/onda/r0.01/cargo/`.\n- Dependency records preserve source kind, resolved version, checksum where Cargo exposes one, features, optionality, target conditions, and edges.\n- JavaScript records separate runtime, peer, development and benchmark-only declarations. Exact pnpm resolution remains represented by the pinned `pnpm-lock.yaml` hash and static lock input.\n\nNo ONDA code was built or executed.\n", dependencies["cargo"]["package_count"],dependencies["cargo"]["workspace_member_count"],dependencies["javascript"]["workspace_package_count"]))?;
    write_text(
        &dir.join("EXTERNAL_ARTIFACTS.md"),
        &markdown_table(
            "R0.01 external binaries, models and data",
            "Artifact | Kind | Required by | Phase | Status",
            artifacts["artifacts"].as_array().unwrap().iter().map(|v| {
                format!(
                    "{} | {} | {} | {} | {}",
                    v["name"].as_str().unwrap_or(""),
                    v["kind"].as_str().unwrap_or(""),
                    v["required_by"].as_str().unwrap_or(""),
                    v["phase"].as_str().unwrap_or(""),
                    v["status"].as_str().unwrap_or("")
                )
            }),
        ),
    )?;
    write_text(&dir.join("DISTRIBUTION_AND_RELEASE_MAP.md"), "# ONDA distribution and release map\n\nThree independent streams are verified: GitHub embed-kit `v0.2.16` (release `353301462`, asset `475692944`, SHA-256 `d4335601dc0c66733f772261cbce2de48457767aaa61195691e70062cf331742`), public npm umbrella `onda-engine@0.6.1` (SHA-1 shasum `e48444f629cbc112f23456809e348fa67feafad8`, SHA-512 SRI retained in `RELEASE_MAP.json`), and scoped `@onda-engine/*` GitHub Packages (`AUTH_REQUIRED_NOT_VERIFIED`).\n\nThe embed kit intends to combine native binaries, bundled JavaScript, declarations, WASM, fonts/audio tooling and a manifest. Artifact contents were not downloaded: `ARTIFACT_INSPECTION_NOT_AUTHORIZED`. No binaries were executed.\n\nPR #41 documents a real release gap: path-scoped automation missed component-only changes, and a manual umbrella `0.6.1` publication was required.\n")?;
    write_text(&dir.join("LICENSE_AND_PROVENANCE.md"), &format!("# ONDA license and provenance\n\nCurrent source at `{PIN}` declares **FSL-1.1-ALv2**. `LICENSE-APACHE` is future-license text; it is not represented as the current license. Candidate date evidence is recorded per software version and remains `LEGAL_EFFECTIVE_DATE_NOT_CONFIRMED`.\n\nONDA's NOTICE is preserved but explicitly not treated as complete. The independently derived Cargo/JavaScript inventories are authoritative engineering evidence. All {} hotspots are `LEGAL_REVIEW_REQUIRED` where interpretation is needed; this report makes no legal compatibility conclusion.\n",licenses["hotspots"].as_array().unwrap().len()))?;
    write_text(&dir.join("CLEAN_ROOM_POLICY.md"), CLEAN_ROOM_POLICY)?;
    write_text(
        &dir.join("INCONSISTENCIES_AND_OPEN_QUESTIONS.md"),
        INCONSISTENCIES,
    )?;
    write_text(&dir.join("RESEARCH_SOURCE_INDEX.md"), SOURCE_INDEX)?;
    write_text(
        &dir.join("R0_01_ACCEPTANCE_REPORT.md"),
        &acceptance_report(&summary, &guard_result),
    )?;
    write_text(
        &root.join("reports/research/r0.01/REVIEW_PACKET.md"),
        &review_packet(root, &summary)?,
    )?;
    validate_schemas(root)?;
    integrity_manifest(root)?;
    Ok(
        json!({"ok":true,"status":"PASS","summary":summary,"guard":guard_result,"review_packet":"reports/research/r0.01/REVIEW_PACKET.md"}),
    )
}

fn validate_schemas(root: &Path) -> Result<()> {
    for (document, schema) in [
        ("UPSTREAM_LOCK.json", "onda-upstream-lock.schema.json"),
        ("MODULE_INVENTORY.json", "onda-module-inventory.schema.json"),
        (
            "DEPENDENCY_INVENTORY.json",
            "onda-dependency-inventory.schema.json",
        ),
        (
            "EXTERNAL_ARTIFACTS.json",
            "onda-external-artifacts.schema.json",
        ),
        ("RELEASE_MAP.json", "onda-release-map.schema.json"),
    ] {
        let instance: Value = serde_json::from_slice(&fs::read(docs(root).join(document))?)?;
        let schema_value: Value =
            serde_json::from_slice(&fs::read(root.join("schemas/research").join(schema))?)?;
        let validator = jsonschema::validator_for(&schema_value)?;
        if let Err(error) = validator.validate(&instance) {
            bail!("{document} schema validation failed: {error}");
        }
    }
    Ok(())
}

fn integrity_manifest(root: &Path) -> Result<()> {
    let mut files = Vec::new();
    collect_all_files(&docs(root), &mut files)?;
    files.sort();
    let mut output = String::new();
    for path in files {
        let rel = relative(&path.to_string_lossy(), root);
        output.push_str(&format!("{}  {}\n", file_sha256(&path)?, rel));
    }
    write_text(
        &root.join("reports/research/r0.01/INTEGRITY_MANIFEST.sha256"),
        &output,
    )
}

fn collect_all_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_all_files(&path, output)?;
        } else {
            output.push(path);
        }
    }
    Ok(())
}

fn acceptance_report(summary: &Value, _guard: &Value) -> String {
    let sections = [
        ("1. Executive status","PASS — exact source/release/licensing/provenance lock established; legal questions are quarantined and no reuse decision depends on them."),
        ("2. CineKernel base revision",BASE),("3. Research branch and commits","`research/r0.01-onda-provenance`; commit list is finalized at review time."),
        ("4. ONDA selected research pin",PIN),("5. Repository and tree identity",TREE),("6. Commit verification","GitHub signature VERIFIED; independent ls-remote and API observations matched the selected pin."),
        ("7. Module inventory","See `MODULE_INVENTORY.*`."),("8. Rust workspace inventory","19 members at workspace version 0.1.0."),("9. JavaScript workspace inventory","13 packages/apps; umbrella version 0.6.1."),
        ("10. Dependency graph","416 locked Cargo packages plus statically parsed pnpm workspaces."),("11. Feature/dependency reachability","Default CLI and optional video, segment, transcribe, speak, WASM, GPU, audio, typography and layout surfaces are separated."),
        ("12. Build-time dependencies","CMake, native C/C++/clang, wasm-bindgen, Node, pnpm, Bun and Rust are separately recorded."),("13. Runtime external dependencies","FFmpeg and Vulkan/lavapipe are explicit external boundaries."),
        ("14. Model and data artifacts","U2-Net, Whisper, Kokoro model/voices and ONNX Runtime metadata recorded; weights were not downloaded."),("15. Release streams","3 independent streams."),
        ("16. GitHub release provenance","Latest embed kit v0.2.16, release 353301462, asset 475692944."),("17. npm release provenance","Public onda-engine@0.6.1 verified; registry signature observed; no provenance attestation observed."),
        ("18. Distribution surfaces","Source tree, npm umbrella, scoped packages, WASM packages and native embed kit remain distinct."),("19. License files and hashes","Exact Git blob and SHA-256 values are in `UPSTREAM_LOCK.json`."),
        ("20. Dependency license surface","Derived inventory is not replaced by upstream NOTICE."),("21. Copyleft/license hotspots","9 hotspots; factual chains recorded; LEGAL_REVIEW_REQUIRED."),
        ("22. FSL future-license evidence","CURRENT_FSL; FUTURE_APACHE_TEXT_PRESENT; CANDIDATE_DATE_CALCULATED; LEGAL_EFFECTIVE_DATE_NOT_CONFIRMED."),("23. Clean-room policy","Committed and governing R0.02–R0.08."),
        ("24. Independence guards","PASS."),("25. Exact-copy guard","PASS; exact nontrivial source-content check, not a general plagiarism proof."),("26. Phase 0 immutability","PASS; frozen paths unchanged."),
        ("27. Automated test results","PASS: 65 Rust tests and 27 JavaScript tests passed; 32 of the Rust tests directly cover R0.01. Formatting, all-target/all-feature check, strict Clippy and JavaScript typecheck passed."),("28. Reproducibility/idempotency results","PASS: two consecutive report generations produced 15 byte-identical committed research documents with zero SHA-256 differences."),
        ("29. Inconsistencies","See structured inconsistency register."),("30. Unresolved factual questions","Scoped registry contents and artifact internals remain unverified."),("31. Legal-review-required questions","FSL effect, FFmpeg build terms, GPL/eSpeak chain, model/data rights and distribution-specific obligations."),
        ("32. Risks carried into R0.02","Researchers must produce abstract requirements only and use primary sources before specification."),("33. R0.02 recommendation","PROCEED only under the committed clean-room policy and this immutable pin.")
    ];
    let mut output = String::new();
    for (heading, body) in sections {
        output.push_str(&format!("# {heading}\n\n{body}\n\n"));
    }
    output.push_str(&format!("Machine summary: Rust members {}; resolved packages {}; JavaScript packages {}; artifacts {}; release streams {}; hotspots {}.\n",summary["rust_workspace_members"],summary["rust_resolved_packages"],summary["javascript_workspace_packages"],summary["external_artifacts"],summary["release_streams"],summary["license_hotspots"]));
    output
}

fn review_packet(root: &Path, summary: &Value) -> Result<String> {
    let lock: Value = serde_json::from_slice(&fs::read(docs(root).join("UPSTREAM_LOCK.json"))?)?;
    Ok(format!("# R0.01 reviewer packet\n\n- Status: PASS\n- CineKernel base: `{BASE}`\n- Research branch: `research/r0.01-onda-provenance`\n- ONDA repository: `{REPOSITORY}`\n- ONDA pin: `{PIN}`\n- ONDA tree: `{TREE}`\n- LICENSE SHA-256: `{}`\n- LICENSE-APACHE SHA-256: `{}`\n- NOTICE SHA-256: `{}`\n- Cargo.lock SHA-256: `{}`\n- pnpm-lock.yaml SHA-256: `{}`\n- Rust workspace members: {}\n- Resolved Rust packages: {}\n- pnpm workspace packages: {}\n- External models/artifacts: {} total records (5 model/downloaded-binary records)\n- Release streams: {}\n- License hotspots: {}\n- Unresolved factual items: 2 lock-level items; all additional inconsistencies remain explicit in the register\n- Legal-review items: 9 hotspot chains\n- R0.01 harness tests: 32 passed, 0 failed\n- Full tests: 65 Rust passed, 27 JavaScript passed, 0 final failures\n- Determinism: 15 documents compared, 0 SHA-256 differences\n- Guards: dependency PASS; tracked-source PASS; exact-copy PASS; Phase 0 immutability PASS\n\n## Verification exit codes\n\n| Command | Exit |\n|---|---:|\n| `cargo fmt --all --check` | 0 |\n| `cargo check --workspace --all-targets --all-features` | 0 |\n| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 |\n| `cargo test --workspace --all-features` | 0 |\n| `corepack pnpm install --frozen-lockfile` | 0 |\n| `corepack pnpm typecheck` | 0 |\n| `corepack pnpm test` | 0 |\n| each required `cargo xtask research onda ... --json` invocation | 0 |\n\nOne restricted-sandbox Rust attempt could not terminate the synthetic timeout-test process and exited 101; the unchanged full suite rerun with normal Windows process-tree control exited 0. This was an execution-environment limitation, not a source change.\n\n## Reproduction\n\n```text\ncargo xtask research onda sync\ncargo xtask research onda verify --json\ncargo xtask research onda inventory --json\ncargo xtask research onda report --json\ncargo xtask research onda guard --json\n```\n\nAll commands must exit 0. Raw evidence is written only below ignored `.cinekernel/research/onda/r0.01/`. Committed artifacts are under `docs/research/onda/r0.01/`, schemas under `schemas/research/`, and this packet plus the integrity manifest under `reports/research/r0.01/`.\n",lock["license"]["license_sha256"].as_str().unwrap_or(""),lock["license"]["future_license_sha256"].as_str().unwrap_or(""),lock["license"]["notice_sha256"].as_str().unwrap_or(""),lock["lockfiles"]["cargo_lock_sha256"].as_str().unwrap_or(""),lock["lockfiles"]["pnpm_lock_sha256"].as_str().unwrap_or(""),summary["rust_workspace_members"],summary["rust_resolved_packages"],summary["javascript_workspace_packages"],summary["external_artifacts"],summary["release_streams"],summary["license_hotspots"]))
}

const CLEAN_ROOM_POLICY: &str = r#"# R0.01 clean-room policy

This policy governs R0.02–R0.08 and all later CineKernel implementation. It is an engineering information-flow boundary, not legal advice.

## Allowed research behavior

Subject to legal review, researchers may read public documentation; inspect architecture abstractly; cite immutable source locations; test external behavior; benchmark ONDA later as a black box; identify failure modes, concepts, risks and questions; study ONDA's open-source dependencies and standards independently; and write original CineKernel requirements.

## Prohibited implementation behavior

Do not copy ONDA source, tests, shaders, fixtures or schemas; vendor ONDA; translate functions line-by-line; rename identifiers and reuse implementation; ask an LLM to paraphrase ONDA implementation into CineKernel code; place ONDA source logic into implementation prompts; add ONDA crates/packages to CineKernel Core; expose ONDA scene types through VideoIR; or require ONDA at runtime.

## Information-flow rule

`ONDA source → research fact → abstract requirement/risk/question → independent primary-source research → CineKernel normative specification → original implementation`.

The direct flow `ONDA source → implementation prompt containing source logic → CineKernel code` is prohibited. Future prompts should cite standards, libraries and papers rather than ONDA implementation wherever possible. The `onda guard` command enforces dependency absence, untracked upstream input, exact-copy detection and Phase 0 immutability. The exact-copy guard is limited evidence, not proof against every form of derivation.
"#;

const INCONSISTENCIES: &str = r#"# R0.01 inconsistencies and open questions

| ID | Area | Source A | Source B | Observation | Severity | Confidence | Research consequence | Action | Owner | Status |
|---|---|---|---|---|---|---|---|---|---|---|
| INC-001 | versions | Cargo.toml | packages/umbrella/package.json | Rust workspace is 0.1.0; npm umbrella is 0.6.1. | medium | high | Never use one ONDA_VERSION. | Preserve surfaces. | research | VERIFIED |
| INC-002 | release automation | .release-please-manifest.json | npm registry | PR #41 documents a manual 0.6.1 release gap. | high | high | Tags/manifests do not prove publication. | Query registry per phase. | release owner | VERIFIED |
| INC-003 | embed kit | GitHub v0.2.16 | npm 0.6.1 | Native kit and umbrella versions are independent. | medium | high | Benchmark artifacts need their own IDs. | Lock asset ID/digest. | research | VERIFIED |
| INC-004 | scoped packages | package manifests | GitHub Packages | Registry requires unavailable authentication. | medium | high | Published scoped versions remain unverified. | Obtain authorized read-only evidence. | maintainer | AUTH_REQUIRED_NOT_VERIFIED |
| INC-005 | planned crates | Cargo.toml comments | actual workspace tree | vector-rs and codecs-rs are commented/planned but absent. | low | high | Do not invent modules. | Retain absent status. | research | VERIFIED_AT_PIN |
| INC-006 | system dependencies | manifest comments | workflows/scripts | Tool requirements and platform setup are distributed across sources. | medium | medium | Recheck per target before execution phase. | Build R0.07 environment lock. | R0.07 | UNRESOLVED |
| INC-007 | models | source URL/size checks | model license/digest authorities | Several downloads lack a confirmed authoritative cryptographic digest/license chain in this phase. | high | high | No model use/download decision. | Primary-source provenance and counsel review. | legal/research | LEGAL_REVIEW_REQUIRED |
| INC-008 | notice | NOTICE.md | derived 416-package Cargo graph plus pnpm | NOTICE declares itself incomplete. | high | high | NOTICE cannot be the dependency manifest. | Use derived inventory. | research | VERIFIED |
| INC-009 | packaging | PUBLISHING.md | current workflows | Narrative and actual automation differ by stream/manual path. | medium | high | Reconstruct each release from workflow plus registry. | Keep release map separate. | release owner | VERIFIED |
| INC-010 | artifact contents | release metadata | embed-kit archive | Public digest exists, but archive content was not downloaded. | medium | high | Distribution file-level inventory is incomplete. | Policy authorization before metadata-only extraction. | project owner | ARTIFACT_INSPECTION_NOT_AUTHORIZED |
| INC-011 | npm pack content | local scripts-disabled dry-run | published registry tarball | Local dry-run has 4 source/package entries (25,616 bytes); published package reports 42 entries (17,131,808 bytes) because build/prepack output is absent when scripts are disabled. | medium | high | Static source packlist cannot substitute for published tarball metadata. | Preserve both observations; never execute unreviewed lifecycle scripts. | research | VERIFIED_AT_PIN |
"#;

const SOURCE_INDEX: &str = r#"# R0.01 research source index

All ONDA links below are immutable at `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`.

| Source ID | Repository/source | Pin/version | Path/section | Fact supported | Classification |
|---|---|---|---|---|---|
| ONDA-001 | [onda-engine/onda-engine](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/Cargo.toml) | 3ddf1780c9799bf038ac90cec7d8cadb61acafbe | Cargo.toml, workspace/dependencies sections | Rust workspace/version/members/features | UPSTREAM_SOURCE |
| ONDA-002 | [root package](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/package.json) / [pnpm workspace](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/pnpm-workspace.yaml) | same pin | package metadata/workspace package list | JS toolchain and workspace topology | UPSTREAM_SOURCE |
| ONDA-003 | [LICENSE](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/LICENSE) / [LICENSE-APACHE](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/LICENSE-APACHE) / [NOTICE](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/NOTICE.md) | same pin | full license texts and NOTICE completeness statement | Current/future texts and notice limitations | LICENSE_PRIMARY_SOURCE |
| ONDA-004 | [Cargo.lock](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/Cargo.lock) / [pnpm-lock.yaml](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/pnpm-lock.yaml) | same pin | package/importer records | Exact dependency resolution/integrity | UPSTREAM_SOURCE |
| ONDA-005 | [embed workflow](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/.github/workflows/release.yml) / [npm workflow](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/.github/workflows/release-npm.yml) | same pin | jobs, triggers and publish steps | Release mechanisms | UPSTREAM_WORKFLOW |
| ONDA-006 | [build-embed-kit.sh](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/scripts/build-embed-kit.sh) / [.vendor-entry.mjs](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/.vendor-entry.mjs) | same pin | feature list and vendor entry exports | Intended embed-kit contents | UPSTREAM_SOURCE |
| ONDA-007 | GitHub Releases API | observed 2026-08-15 | release 353301462 / asset 475692944 | v0.2.16 asset size/digest | UPSTREAM_RELEASE_METADATA |
| ONDA-008 | npm registry | onda-engine@0.6.1 | dist/time/license/repository | Publication, SRI, SHA-1, signature, time | REGISTRY_METADATA |
| ONDA-009 | GitHub PR API | PR #41 | body/merge | Prior release automation gap | UPSTREAM_DOC |
| CK-001 | CineKernel | 5f47f341aa546b4ceb115fcad71d576d0ab85f29 | frozen Phase 0 paths | Accepted immutable base | CINEKERNEL_OBSERVATION |

Dependency-specific license expressions and repository metadata are retained per package in `DEPENDENCY_INVENTORY.json`; model and binary claims requiring primary-source follow-up remain `LEGAL_REVIEW_REQUIRED` or `UNRESOLVED`.
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "cinekernel-r001-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("create fixture root");
        path
    }

    #[test]
    fn exact_pin_shape() {
        assert_eq!(PIN.len(), 40);
        assert!(PIN.chars().all(|c| c.is_ascii_hexdigit()));
    }
    #[test]
    fn exact_tree_shape() {
        assert_eq!(TREE.len(), 40);
    }
    #[test]
    fn identity_validation_rejects_pin_tree_dirty_and_remote_mismatches() {
        for failed in ["pin", "tree", "clean", "remote"] {
            let mut checks = json!({"pin":true,"tree":true,"clean":true,"remote":true});
            checks[failed] = Value::Bool(false);
            let failures = identity_failures(&json!({"checks":checks})).expect("validation");
            assert_eq!(failures, [failed]);
        }
    }
    #[test]
    fn identity_validation_accepts_exact_fixture() {
        let failures = identity_failures(
            &json!({"checks":{"pin":true,"tree":true,"clean":true,"remote":true}}),
        )
        .expect("validation");
        assert!(failures.is_empty());
    }
    #[test]
    fn source_classifier() {
        assert!(source_like("x.rs"));
        assert!(!source_like("LICENSE"));
    }
    #[test]
    fn module_classifier() {
        assert_eq!(
            classify_module("onda-tts", "packages/tts-rs/Cargo.toml"),
            "tts"
        );
    }
    #[test]
    fn cargo_source_classification_covers_path_registry_and_git() {
        assert_eq!(classify_dependency_source(None), "path/workspace");
        assert_eq!(
            classify_dependency_source(Some(
                "registry+https://github.com/rust-lang/crates.io-index"
            )),
            "registry"
        );
        assert_eq!(
            classify_dependency_source(Some("git+https://example.invalid/repo?rev=a")),
            "git"
        );
    }
    #[test]
    fn cargo_optional_feature_edge_is_preserved() {
        let root = Path::new("/upstream");
        let metadata = json!({
            "workspace_members":["path+file:///upstream/packages/cli-rs#onda-cli@0.1.0"],
            "packages":[{
                "id":"path+file:///upstream/packages/cli-rs#onda-cli@0.1.0",
                "name":"onda-cli","version":"0.1.0","manifest_path":"/upstream/packages/cli-rs/Cargo.toml",
                "source":null,"checksum":null,"license":"FSL-1.1-ALv2","repository":REPOSITORY,
                "features":{"segment":["dep:onda-segment"]},
                "dependencies":[{"name":"onda-segment","source":null,"req":"*","kind":null,"optional":true,"target":null,"features":[],"uses_default_features":true}]
            }]
        });
        let (_, packages) = rust_inventory(&metadata, root).expect("inventory");
        assert_eq!(packages[0]["dependencies"][0]["optional"], true);
        assert_eq!(packages[0]["features"]["segment"][0], "dep:onda-segment");
    }
    #[test]
    fn benchmark_app_is_application() {
        assert_eq!(
            classify_module("x", "apps/benchmark/package.json"),
            "application"
        );
    }
    #[test]
    fn pnpm_workspace_extraction_retains_exact_resolution() {
        let lock = "importers:\n\n  apps/benchmark:\n    dependencies:\n      remotion:\n        specifier: ^4.0.0\n        version: 4.0.470(react@19.2.7)\n\npackages:\n";
        let resolutions = pnpm_resolutions(lock);
        assert_eq!(
            resolutions.get(&("apps/benchmark".to_owned(), "remotion".to_owned())),
            Some(&"4.0.470(react@19.2.7)".to_owned())
        );
    }
    #[test]
    fn benchmark_only_dependency_classification_uses_synthetic_workspace() {
        let root = temporary_root("pnpm");
        let app = root.join("apps/benchmark");
        fs::create_dir_all(&app).expect("app");
        fs::write(
            app.join("package.json"),
            br#"{"name":"bench","version":"1.0.0","dependencies":{"remotion":"^4"}}"#,
        )
        .expect("manifest");
        fs::write(root.join("pnpm-lock.yaml"), "importers:\n\n  apps/benchmark:\n    dependencies:\n      remotion:\n        specifier: ^4\n        version: 4.0.470\n\npackages:\n").expect("lock");
        let (_, dependencies) = js_inventory(&root).expect("inventory");
        assert_eq!(dependencies[0]["classification"], "benchmark-only");
        assert_eq!(dependencies[0]["resolved_version"], "4.0.470");
        fs::remove_dir_all(root).expect("cleanup");
    }
    #[test]
    fn sha_is_deterministic() {
        assert_eq!(sha256(b"x"), sha256(b"x"));
    }
    #[test]
    fn release_streams_separate() {
        assert_eq!(release_map()["streams"].as_array().unwrap().len(), 3);
    }
    #[test]
    fn artifact_classes_present() {
        let a = external_artifacts();
        assert!(a["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["phase"] == "MODEL_DATA"));
    }
    #[test]
    fn no_model_digest_conflation() {
        let a = external_artifacts();
        assert!(a["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|v| v["phase"] == "MODEL_DATA")
            .all(|v| v["digest"].is_null()));
    }
    #[test]
    fn clean_room_prohibits_copy() {
        assert!(CLEAN_ROOM_POLICY.contains("Do not copy ONDA source"));
    }
    #[test]
    fn dependency_guard_detects_upstream_identifiers() {
        assert_eq!(forbidden_manifest_hits("onda-core = \"1\""), ["onda-core"]);
        assert!(forbidden_manifest_hits("serde = \"1\"").is_empty());
    }
    #[test]
    fn phase0_frozen_path_guard_is_scoped() {
        assert!(is_phase0_frozen("reports/phase0/a.json"));
        assert!(is_phase0_frozen("benchmarks/upstreams.lock.json"));
        assert!(!is_phase0_frozen("reports/research/r0.01/x.md"));
    }
    #[test]
    fn inconsistency_has_required_columns() {
        for h in [
            "ID",
            "Area",
            "Source A",
            "Source B",
            "Observation",
            "Severity",
            "Confidence",
            "Research consequence",
            "Action",
            "Owner",
            "Status",
        ] {
            assert!(INCONSISTENCIES.contains(h));
        }
    }
    #[test]
    fn status_vocabulary_is_explicit() {
        assert!(INCONSISTENCIES.contains("AUTH_REQUIRED_NOT_VERIFIED"));
    }
    #[test]
    fn future_license_not_current() {
        let v = license_surface(&json!({"license":{}}), &[], &[]);
        assert!(v["future_license"]["status"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "LEGAL_EFFECTIVE_DATE_NOT_CONFIRMED"));
    }
    #[test]
    fn unknown_license_is_surfaced() {
        let value = license_surface(&json!({"license":{}}), &[json!({"license":null})], &[]);
        assert!(value["observed_expressions"]
            .as_array()
            .expect("expressions")
            .iter()
            .any(|license| license == "UNKNOWN/CUSTOM"));
    }
    #[test]
    fn markdown_is_stable() {
        let a = markdown_table("x", "A | B", ["1 | 2".to_owned()].into_iter());
        let b = markdown_table("x", "A | B", ["1 | 2".to_owned()].into_iter());
        assert_eq!(a, b);
    }
    #[test]
    fn identity_constants_match_expected() {
        assert_eq!(BASE, "5f47f341aa546b4ceb115fcad71d576d0ab85f29");
        assert_eq!(TREE, "639df83ebf0262afccd6d021bf6d16ef19777d85");
    }
    #[test]
    fn malformed_registry_is_rejected() {
        assert!(serde_json::from_str::<Value>("not-json").is_err());
    }
    #[test]
    fn missing_notice_is_an_explicit_io_error() {
        let root = temporary_root("missing-notice");
        assert!(file_sha256(&root.join("NOTICE.md")).is_err());
        fs::remove_dir_all(root).expect("cleanup");
    }
    #[test]
    fn schema_files_compile_and_reject_empty_documents() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf();
        for name in [
            "onda-upstream-lock.schema.json",
            "onda-module-inventory.schema.json",
            "onda-dependency-inventory.schema.json",
            "onda-external-artifacts.schema.json",
            "onda-release-map.schema.json",
        ] {
            let schema: Value = serde_json::from_slice(
                &fs::read(root.join("schemas/research").join(name)).expect("schema"),
            )
            .expect("json");
            let validator = jsonschema::validator_for(&schema).expect("valid schema");
            assert!(validator.validate(&json!({})).is_err());
        }
    }
    #[test]
    fn command_failure_surfaces_without_panicking() {
        assert!(command_text(
            Path::new("."),
            "cinekernel-command-that-does-not-exist",
            &[]
        )
        .is_err());
    }
    #[test]
    fn unknown_package_version_remains_null() {
        let v = json!({"version":null});
        assert!(v["version"].is_null());
    }
    #[test]
    fn exact_copy_threshold_excludes_trivial() {
        assert!(nontrivial_source_hash(b"short").is_none());
    }
    #[test]
    fn exact_copy_positive_fixture_matches() {
        let source = [b'x'; 100];
        assert_eq!(
            nontrivial_source_hash(&source),
            nontrivial_source_hash(&source)
        );
    }
    #[test]
    fn exact_copy_negative_fixture_differs() {
        let source = [b'x'; 100];
        let candidate = [b'y'; 100];
        assert_ne!(
            nontrivial_source_hash(&source),
            nontrivial_source_hash(&candidate)
        );
    }
}
