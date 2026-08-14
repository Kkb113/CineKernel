use anyhow::{bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output};
use sysinfo::System;
use uuid::Uuid;

pub const REMOTION_COMMIT: &str = "4e459b8b3aeec12ac8346666773ea28892a30e31";
pub const HYPERFRAMES_COMMIT: &str = "532caf7aa24fef382cb103013f6414bb547a4129";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstreamLock {
    pub schema_version: String,
    pub generated_at_utc: String,
    pub remotion: Upstream,
    pub hyperframes: Upstream,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Upstream {
    pub repository: String,
    pub commit: String,
    pub source_tree_git_sha: String,
    pub release_or_package_version: String,
    pub package_registry_integrity: String,
    pub package_git_head: Option<String>,
    pub release_tag: String,
    pub release_commit: String,
    pub source_commit_ahead_of_package: Value,
    pub source_commits_ahead: u64,
    pub license_file: String,
    pub license_sha256: String,
    pub sparse_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchmarkSpec {
    pub schema_version: String,
    pub notice: String,
    pub cases: Vec<BenchmarkCase>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchmarkCase {
    pub id: String,
    pub title: String,
    pub purpose: String,
    pub duration_seconds: f64,
    pub expected_frame_count: u64,
    #[serde(default)]
    pub expected_audio_tracks: usize,
    #[serde(default)]
    pub equivalence: BTreeMap<String, String>,
    pub supported_engines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub fn repository_root() -> Result<PathBuf> {
    let mut cursor = env::current_dir().context("read current directory")?;
    loop {
        if cursor.join("Cargo.toml").is_file() && cursor.join("benchmarks").is_dir() {
            return Ok(cursor);
        }
        if !cursor.pop() {
            bail!("not inside a CineKernel checkout");
        }
    }
}

pub fn runtime_root(root: &Path) -> PathBuf {
    root.join(".cinekernel")
}

pub fn load_upstream_lock(root: &Path) -> Result<UpstreamLock> {
    let path = root.join("benchmarks/upstreams.lock.json");
    serde_json::from_slice(&fs::read(&path).with_context(|| format!("read {}", path.display()))?)
        .context("parse upstream lock")
}

pub fn load_benchmark_spec(root: &Path) -> Result<BenchmarkSpec> {
    let path = root.join("benchmarks/specs/phase0-cases.json");
    serde_json::from_slice(&fs::read(&path).with_context(|| format!("read {}", path.display()))?)
        .context("parse benchmark spec")
}

pub fn run(program: &OsStr, args: &[OsString], cwd: &Path) -> Result<CommandRecord> {
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("start {}", program.to_string_lossy()))?;
    Ok(record(program, args, &output))
}

fn record(program: &OsStr, args: &[OsString], output: &Output) -> CommandRecord {
    CommandRecord {
        program: program.to_string_lossy().into_owned(),
        args: args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect(),
        exit_code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    }
}

pub fn command_version(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(resolve_program(program))
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    String::from_utf8_lossy(text)
        .lines()
        .next()
        .map(str::to_owned)
}

pub fn pnpm_program() -> OsString {
    resolve_program("pnpm")
}

fn resolve_program(program: &str) -> OsString {
    if cfg!(windows) && matches!(program, "pnpm" | "corepack" | "npm" | "npx") {
        OsString::from(format!("{program}.cmd"))
    } else {
        OsString::from(program)
    }
}

pub fn cargo_program() -> OsString {
    if command_version("cargo", &["--version"]).is_some() {
        return OsString::from("cargo");
    }
    if cfg!(windows) {
        if let Some(profile) = env::var_os("USERPROFILE") {
            let candidate = PathBuf::from(profile).join(".cargo/bin/cargo.exe");
            if candidate.is_file() {
                return candidate.into_os_string();
            }
        }
    }
    OsString::from("cargo")
}

pub fn git_state(root: &Path) -> (String, bool) {
    let revision =
        command_text(root, "git", &["rev-parse", "HEAD"]).unwrap_or_else(|| "UNBORN".to_owned());
    let dirty = command_text(root, "git", &["status", "--porcelain"])
        .is_some_and(|value| !value.trim().is_empty());
    (revision, dirty)
}

fn command_text(root: &Path, program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub fn capture_environment(root: &Path) -> Result<Value> {
    let mut system = System::new_all();
    system.refresh_all();
    let lock = load_upstream_lock(root)?;
    let (revision, dirty) = git_state(root);
    let cpu = system
        .cpus()
        .first()
        .map_or("unknown", |item| item.brand())
        .trim()
        .to_owned();
    let physical = system.physical_core_count();
    let tools = json!({
        "git": command_version("git", &["--version"]),
        "rustc": command_version("rustc", &["--version"]).or_else(|| rustup_version("rustc")),
        "cargo": command_version("cargo", &["--version"]).or_else(|| rustup_version("cargo")),
        "node": command_version("node", &["--version"]),
        "corepack": command_version("corepack", &["--version"]),
        "pnpm": command_version("pnpm", &["--version"]),
        "ffmpeg": command_version("ffmpeg", &["-version"]),
        "ffprobe": command_version("ffprobe", &["-version"]),
        "chrome": chrome_version(),
        "docker": command_version("docker", &["--version"]),
        "blender": command_version("blender", &["--version"]),
    });
    let mut normalized = json!({
        "schema_version": "phase0.environment.v1",
        "captured_at_utc": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        "os": format!("{} {}", System::name().unwrap_or_else(|| env::consts::OS.to_owned()), System::os_version().unwrap_or_default()).trim().to_owned(),
        "architecture": env::consts::ARCH,
        "cpu": cpu,
        "logical_cores": system.cpus().len(),
        "physical_cores": physical,
        "ram_bytes": system.total_memory(),
        "gpu": detect_gpu(),
        "tools": tools,
        "upstreams": {
            "remotion": {"commit": lock.remotion.commit, "package_version": lock.remotion.release_or_package_version},
            "hyperframes": {"commit": lock.hyperframes.commit, "package_version": lock.hyperframes.release_or_package_version}
        },
        "cinekernel": {"revision": revision, "dirty": dirty},
        "render_environment": rendering_environment()
    });
    redact_value(&mut normalized);
    let mut hashable = normalized.clone();
    if let Some(object) = hashable.as_object_mut() {
        object.remove("captured_at_utc");
    }
    let encoded = serde_json::to_vec(&hashable)?;
    let environment_id = hex::encode(Sha256::digest(encoded));
    normalized
        .as_object_mut()
        .context("environment must be an object")?
        .insert("environment_id".to_owned(), Value::String(environment_id));
    Ok(normalized)
}

fn rustup_version(program: &str) -> Option<String> {
    let rustup = if cfg!(windows) {
        env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join(".cargo/bin/rustup.exe"))?
    } else {
        PathBuf::from("rustup")
    };
    let output = Command::new(rustup)
        .args(["run", "stable", program, "--version"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn chrome_version() -> Option<String> {
    if cfg!(windows) {
        for variable in ["PROGRAMFILES", "PROGRAMFILES(X86)", "LOCALAPPDATA"] {
            if let Some(base) = env::var_os(variable) {
                for suffix in [
                    "Google/Chrome/Application/chrome.exe",
                    "Microsoft/Edge/Application/msedge.exe",
                ] {
                    let candidate = PathBuf::from(&base).join(suffix);
                    if candidate.is_file() {
                        let escaped = candidate.to_string_lossy().replace('\'', "''");
                        if let Ok(output) = Command::new("powershell")
                            .args([
                                "-NoProfile",
                                "-Command",
                                &format!("(Get-Item -LiteralPath '{escaped}').VersionInfo.ProductVersion"),
                            ])
                            .output()
                        {
                            let version = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                            if !version.is_empty() {
                                return Some(format!("{} {version}", candidate.file_stem()?.to_string_lossy()));
                            }
                        }
                    }
                }
            }
        }
        return None;
    }
    for program in ["google-chrome", "chromium", "chromium-browser"] {
        if let Some(version) = command_version(program, &["--version"]) {
            return Some(version);
        }
    }
    None
}

fn detect_gpu() -> Value {
    if cfg!(windows) {
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", "Get-CimInstance Win32_VideoController | Select-Object -First 1 Name,DriverVersion | ConvertTo-Json -Compress"])
            .output();
        if let Ok(output) = output {
            if output.status.success() {
                if let Ok(value) = serde_json::from_slice(&output.stdout) {
                    return value;
                }
            }
        }
    }
    Value::Null
}

fn rendering_environment() -> BTreeMap<String, String> {
    const KEYS: [&str; 7] = [
        "CI",
        "DISPLAY",
        "WAYLAND_DISPLAY",
        "WGPU_BACKEND",
        "WGPU_ADAPTER_NAME",
        "REMOTION_BROWSER_EXECUTABLE",
        "HYPERFRAMES_BROWSER_PATH",
    ];
    KEYS.into_iter()
        .filter_map(|key| env::var(key).ok().map(|value| (key.to_owned(), value)))
        .collect()
}

pub fn redact_value(value: &mut Value) {
    match value {
        Value::String(text) => {
            if let Some(home) = env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }) {
                let home = home.to_string_lossy();
                *text = text.replace(home.as_ref(), "$HOME");
            }
            for marker in ["TOKEN=", "PASSWORD=", "SECRET=", "API_KEY="] {
                if text.to_ascii_uppercase().contains(marker) {
                    *text = "[REDACTED]".to_owned();
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact_value),
        Value::Object(values) => values.values_mut().for_each(redact_value),
        _ => {}
    }
}

pub fn new_run_id() -> String {
    format!("{}-{}", Utc::now().format("%Y%m%dT%H%M%SZ"), Uuid::new_v4())
}

pub fn ensure_safe_generated_path(root: &Path, target: &Path) -> Result<()> {
    let runtime = runtime_root(root);
    let normalized = normalize(target);
    let allowed = normalize(&runtime.join("generated"));
    if normalized != allowed && !normalized.starts_with(&allowed) {
        bail!("refusing operation outside {}", allowed.display());
    }
    Ok(())
}

fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    result
}

pub fn directory_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.metadata().ok())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_cleanup_rejects_broad_paths() {
        let root = Path::new("/repo");
        assert!(ensure_safe_generated_path(root, &root.join(".cinekernel/generated")).is_ok());
        assert!(
            ensure_safe_generated_path(root, &root.join(".cinekernel/generated/fixtures")).is_ok()
        );
        assert!(ensure_safe_generated_path(root, &root.join(".cinekernel")).is_err());
        assert!(ensure_safe_generated_path(root, root).is_err());
    }

    #[test]
    fn redaction_replaces_home_and_secret_values() {
        let mut value = json!({"path": env::var(if cfg!(windows) {"USERPROFILE"} else {"HOME"}).unwrap_or_default(), "credential": "TOKEN=abc"});
        redact_value(&mut value);
        assert_eq!(value["path"], "$HOME");
        assert_eq!(value["credential"], "[REDACTED]");
    }

    #[test]
    fn pinned_shas_have_full_length() {
        assert_eq!(REMOTION_COMMIT.len(), 40);
        assert_eq!(HYPERFRAMES_COMMIT.len(), 40);
    }

    #[test]
    fn runtime_paths_preserve_spaces_unicode_and_normalize_components() {
        let temporary = tempfile::tempdir().expect("create temporary directory");
        let root = temporary.path().join("Cine Kernel – portable");
        let runtime = runtime_root(&root);
        assert_eq!(runtime, root.join(".cinekernel"));
        assert!(ensure_safe_generated_path(
            &root,
            &runtime
                .join("generated")
                .join("fixtures")
                .join("..")
                .join("assets")
        )
        .is_ok());
        assert!(ensure_safe_generated_path(&root, &runtime.join("generated").join("..")).is_err());
    }
}
