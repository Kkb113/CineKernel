use anyhow::{anyhow, bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use phase0_common::{
    capture_environment, cargo_program, directory_size, ensure_safe_generated_path, git_state,
    load_benchmark_spec, load_upstream_lock, new_run_id, pnpm_program, repository_root,
    runtime_root, Upstream,
};
use serde_json::{json, Value};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Instant;

const EXIT_MISSING_DEPENDENCY: u8 = 1;
const EXIT_INVALID_CONFIGURATION: u8 = 2;
const EXIT_BENCHMARK_FAILURE: u8 = 3;
const EXIT_VERIFICATION_FAILURE: u8 = 4;
const EXIT_UNSUPPORTED_CAPABILITY: u8 = 5;

#[derive(Parser)]
#[command(
    name = "cargo xtask",
    version,
    about = "CineKernel Phase 0 cross-platform harness"
)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand)]
enum TopCommand {
    Doctor,
    Environment {
        #[command(subcommand)]
        command: EnvironmentCommand,
    },
    Upstream {
        #[command(subcommand)]
        command: UpstreamCommand,
    },
    Phase0 {
        #[command(subcommand)]
        command: Phase0Command,
    },
}

#[derive(Subcommand)]
enum EnvironmentCommand {
    Capture,
}

#[derive(Subcommand)]
enum UpstreamCommand {
    Sync {
        #[arg(long)]
        update: bool,
    },
    Verify,
}

#[derive(Subcommand)]
enum Phase0Command {
    Prepare,
    Run(RunArgs),
    Verify,
    Report,
    Clean {
        #[arg(long, value_enum)]
        scope: CleanScope,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CleanScope {
    Generated,
}

#[derive(Debug, Args)]
struct RunArgs {
    #[arg(long, value_enum, default_value = "smoke")]
    profile: Profile,
    #[arg(long, value_enum)]
    engine: Option<Engine>,
    #[arg(long = "case")]
    case_id: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum Profile {
    Smoke,
    Full,
}

impl Profile {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Smoke => "smoke",
            Self::Full => "full",
        }
    }
    const fn dimensions(self) -> (u32, u32) {
        match self {
            Self::Smoke => (640, 360),
            Self::Full => (1920, 1080),
        }
    }
    const fn duration_scale(self) -> f64 {
        match self {
            Self::Smoke => 0.2,
            Self::Full => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum Engine {
    Remotion,
    Hyperframes,
    #[value(name = "native-2d")]
    Native2d,
    NativeWgpu,
}

impl Engine {
    const ALL: [Self; 4] = [
        Self::Remotion,
        Self::Hyperframes,
        Self::Native2d,
        Self::NativeWgpu,
    ];
    const fn as_str(self) -> &'static str {
        match self {
            Self::Remotion => "remotion",
            Self::Hyperframes => "hyperframes",
            Self::Native2d => "native-2d",
            Self::NativeWgpu => "native-wgpu",
        }
    }
}

struct Failure {
    code: u8,
    error: anyhow::Error,
}

impl Failure {
    fn new(code: u8, error: impl Into<anyhow::Error>) -> Self {
        Self {
            code,
            error: error.into(),
        }
    }
}

type AppResult<T> = std::result::Result<T, Failure>;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match execute(&cli) {
        Ok(value) => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_owned())
                );
            }
            ExitCode::SUCCESS
        }
        Err(failure) => {
            if cli.json {
                println!(
                    "{}",
                    json!({"ok": false, "exit_code": failure.code, "error": format!("{:#}", failure.error)})
                );
            } else {
                eprintln!("error: {:#}", failure.error);
            }
            ExitCode::from(failure.code)
        }
    }
}

fn execute(cli: &Cli) -> AppResult<Value> {
    let root =
        repository_root().map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    match &cli.command {
        TopCommand::Doctor => doctor(&root, cli.json),
        TopCommand::Environment {
            command: EnvironmentCommand::Capture,
        } => environment_capture(&root),
        TopCommand::Upstream {
            command: UpstreamCommand::Sync { update },
        } => upstream_sync(&root, *update),
        TopCommand::Upstream {
            command: UpstreamCommand::Verify,
        } => upstream_verify(&root),
        TopCommand::Phase0 {
            command: Phase0Command::Prepare,
        } => prepare(&root),
        TopCommand::Phase0 {
            command: Phase0Command::Run(args),
        } => phase0_run(&root, args, cli.json),
        TopCommand::Phase0 {
            command: Phase0Command::Verify,
        } => phase0_verify(&root),
        TopCommand::Phase0 {
            command: Phase0Command::Report,
        } => phase0_report(&root),
        TopCommand::Phase0 {
            command:
                Phase0Command::Clean {
                    scope: CleanScope::Generated,
                },
        } => clean_generated(&root),
    }
}

fn doctor(root: &Path, json_output: bool) -> AppResult<Value> {
    let environment = capture_environment(root)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let required = [
        "git", "rustc", "cargo", "node", "corepack", "pnpm", "ffmpeg", "ffprobe", "chrome",
    ];
    let tools = environment["tools"].as_object().ok_or_else(|| {
        Failure::new(
            EXIT_INVALID_CONFIGURATION,
            anyhow!("invalid environment tool map"),
        )
    })?;
    let missing: Vec<&str> = required
        .into_iter()
        .filter(|key| tools.get(*key).is_none_or(Value::is_null))
        .collect();
    let disk = available_disk_space(root);
    let report = json!({"ok": missing.is_empty(), "required_missing": missing, "optional": {"docker": tools.get("docker"), "blender": tools.get("blender")}, "available_disk_bytes": disk, "environment": environment});
    if !json_output {
        println!("CineKernel Phase 0 doctor");
        for (name, value) in tools {
            println!(
                "  {name:10} {}",
                if value.is_null() {
                    "MISSING".to_owned()
                } else {
                    value.as_str().unwrap_or("detected").to_owned()
                }
            );
        }
        println!(
            "  disk       {} bytes available",
            disk.map_or_else(|| "unknown".to_owned(), |value| value.to_string())
        );
    }
    if report["ok"] == false {
        return Err(Failure::new(EXIT_MISSING_DEPENDENCY, anyhow!("missing required dependencies: {}. Install them or expose them on PATH; Rust in the standard user .cargo/bin directory is detected automatically.", missing.join(", "))));
    }
    Ok(report)
}

fn available_disk_space(path: &Path) -> Option<u64> {
    #[cfg(windows)]
    {
        let drive = path
            .components()
            .next()?
            .as_os_str()
            .to_string_lossy()
            .into_owned();
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!(
                    "(Get-PSDrive -Name '{}').Free",
                    drive.trim_end_matches([':', '\\'])
                ),
            ])
            .output()
            .ok()?;
        String::from_utf8_lossy(&output.stdout).trim().parse().ok()
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        None
    }
}

fn environment_capture(root: &Path) -> AppResult<Value> {
    let manifest = capture_environment(root)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let id = manifest["environment_id"].as_str().unwrap_or("unknown");
    let directory = runtime_root(root).join("environments");
    fs::create_dir_all(&directory)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let content = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    fs::write(directory.join(format!("{id}.json")), &content)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    fs::write(directory.join("latest.json"), &content)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    Ok(
        json!({"ok": true, "environment_id": id, "path": format!(".cinekernel/environments/{id}.json")}),
    )
}

fn upstream_sync(root: &Path, update: bool) -> AppResult<Value> {
    if update {
        return Err(Failure::new(EXIT_UNSUPPORTED_CAPABILITY, anyhow!("automatic lock advancement is intentionally unsupported in Phase 0; resolve candidate SHAs explicitly and review the manifest diff")));
    }
    let lock = load_upstream_lock(root)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let base = runtime_root(root).join("upstreams");
    fs::create_dir_all(&base).map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    sync_one(root, "remotion", &lock.remotion, &base.join("remotion"))?;
    sync_one(
        root,
        "hyperframes",
        &lock.hyperframes,
        &base.join("hyperframes"),
    )?;
    upstream_verify(root)
}

fn sync_one(root: &Path, name: &str, upstream: &Upstream, target: &Path) -> AppResult<()> {
    if !target.join(".git").exists() {
        checked(
            root,
            "git",
            &[
                OsString::from("clone"),
                OsString::from("--filter=blob:none"),
                OsString::from("--no-checkout"),
                OsString::from(&upstream.repository),
                target.as_os_str().to_owned(),
            ],
            EXIT_MISSING_DEPENDENCY,
        )?;
    } else {
        checked(
            root,
            "git",
            &[
                OsString::from("-C"),
                target.as_os_str().to_owned(),
                OsString::from("fetch"),
                OsString::from("--filter=blob:none"),
                OsString::from("origin"),
                OsString::from(&upstream.commit),
            ],
            EXIT_MISSING_DEPENDENCY,
        )?;
    }
    checked(
        root,
        "git",
        &[
            OsString::from("-C"),
            target.as_os_str().to_owned(),
            OsString::from("sparse-checkout"),
            OsString::from("init"),
            OsString::from("--cone"),
        ],
        EXIT_INVALID_CONFIGURATION,
    )?;
    let mut sparse = vec![
        OsString::from("-C"),
        target.as_os_str().to_owned(),
        OsString::from("sparse-checkout"),
        OsString::from("set"),
    ];
    sparse.extend(upstream.sparse_paths.iter().map(OsString::from));
    checked(root, "git", &sparse, EXIT_INVALID_CONFIGURATION)?;
    checked(
        root,
        "git",
        &[
            OsString::from("-C"),
            target.as_os_str().to_owned(),
            OsString::from("checkout"),
            OsString::from("--detach"),
            OsString::from(&upstream.commit),
        ],
        EXIT_INVALID_CONFIGURATION,
    )?;
    println!("synced {name} at {}", upstream.commit);
    Ok(())
}

fn upstream_verify(root: &Path) -> AppResult<Value> {
    let lock = load_upstream_lock(root)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let base = runtime_root(root).join("upstreams");
    let mut values = serde_json::Map::new();
    for (name, upstream) in [
        ("remotion", &lock.remotion),
        ("hyperframes", &lock.hyperframes),
    ] {
        let target = base.join(name);
        if !target.join(".git").exists() {
            return Err(Failure::new(
                EXIT_VERIFICATION_FAILURE,
                anyhow!("{name} checkout is missing; run cargo xtask upstream sync"),
            ));
        }
        let actual = output_text(Command::new("git").args([
            "-C",
            &target.to_string_lossy(),
            "rev-parse",
            "HEAD",
        ]))
        .map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?;
        let license = target.join(&upstream.license_file);
        let ok = actual == upstream.commit && license.is_file();
        values.insert(name.to_owned(), json!({"ok": ok, "expected": upstream.commit, "actual": actual, "license": upstream.license_file, "license_present": license.is_file()}));
        if !ok {
            return Err(Failure::new(
                EXIT_VERIFICATION_FAILURE,
                anyhow!("{name} upstream verification failed"),
            ));
        }
    }
    Ok(json!({"ok": true, "upstreams": values}))
}

fn prepare(root: &Path) -> AppResult<Value> {
    for path in ["generated", "runs", "logs", "environments", "projects"] {
        fs::create_dir_all(runtime_root(root).join(path))
            .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    }
    checked_program(
        root,
        &pnpm_program(),
        &[OsString::from("fixtures")],
        EXIT_MISSING_DEPENDENCY,
    )?;
    let fixture_manifest = runtime_root(root).join("generated/fixtures/manifest.json");
    if !fixture_manifest.is_file() {
        return Err(Failure::new(
            EXIT_VERIFICATION_FAILURE,
            anyhow!(
                "fixture generator did not create {}",
                fixture_manifest.display()
            ),
        ));
    }
    Ok(
        json!({"ok": true, "fixtures": ".cinekernel/generated/fixtures/manifest.json", "bytes": directory_size(&runtime_root(root).join("generated/fixtures"))}),
    )
}

fn phase0_run(root: &Path, args: &RunArgs, json_output: bool) -> AppResult<Value> {
    let spec = load_benchmark_spec(root)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let environment = capture_environment(root)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let environment_id = environment["environment_id"]
        .as_str()
        .context("environment id missing")
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?
        .to_owned();
    environment_capture(root)?;
    let run_id = new_run_id();
    let run_dir = runtime_root(root).join("runs").join(&run_id);
    fs::create_dir_all(&run_dir)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let engines: Vec<Engine> = args
        .engine
        .map_or_else(|| Engine::ALL.to_vec(), |engine| vec![engine]);
    let mut results = Vec::new();
    let mut failures = 0_u64;
    for engine in engines {
        for case in &spec.cases {
            if args.case_id.as_ref().is_some_and(|id| id != &case.id)
                || !case
                    .supported_engines
                    .iter()
                    .any(|candidate| candidate == engine.as_str())
            {
                continue;
            }
            let repetitions = match args.profile {
                Profile::Smoke => 1,
                Profile::Full if case.id == "mixed-2d-3d" => 3,
                Profile::Full => 5,
            };
            if args.profile == Profile::Full {
                let _ = execute_run(
                    root,
                    &run_id,
                    engine,
                    args.profile,
                    case,
                    0,
                    &environment_id,
                    true,
                    json_output,
                );
            }
            for repetition in 1..=repetitions {
                let result = execute_run(
                    root,
                    &run_id,
                    engine,
                    args.profile,
                    case,
                    repetition,
                    &environment_id,
                    false,
                    json_output,
                );
                match result {
                    Ok(value) => {
                        if value["exit_code"].as_i64().unwrap_or(1) != 0
                            || value["verification"]["passed"] != true
                        {
                            failures += 1;
                        }
                        results.push(value);
                    }
                    Err(error) => {
                        failures += 1;
                        results.push(json!({"engine": engine.as_str(), "case_id": case.id, "profile": args.profile.as_str(), "repetition": repetition, "harness_error": format!("{:#}", error)}));
                    }
                }
            }
        }
    }
    fs::write(
        run_dir.join("run-summary.json"),
        serde_json::to_vec_pretty(
            &json!({"run_id": run_id, "failures": failures, "results": results}),
        )
        .unwrap(),
    )
    .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    if failures > 0 {
        return Err(Failure::new(EXIT_BENCHMARK_FAILURE, anyhow!("{failures} benchmark repetition(s) failed; raw evidence preserved under .cinekernel/runs/{run_id}")));
    }
    Ok(
        json!({"ok": true, "run_id": run_id, "result_count": results.len(), "path": format!(".cinekernel/runs/{run_id}")}),
    )
}

#[allow(clippy::too_many_arguments)]
fn execute_run(
    root: &Path,
    run_id: &str,
    engine: Engine,
    profile: Profile,
    case: &phase0_common::BenchmarkCase,
    repetition: u64,
    environment_id: &str,
    warmup: bool,
    json_output: bool,
) -> Result<Value> {
    let label = if warmup {
        "warmup".to_owned()
    } else {
        format!("rep-{repetition}")
    };
    let directory = runtime_root(root)
        .join("runs")
        .join(run_id)
        .join(engine.as_str())
        .join(&case.id)
        .join(label);
    fs::create_dir_all(&directory)?;
    let output = directory.join("output.mp4");
    let log_path = directory.join("command.log");
    let (width, height) = profile.dimensions();
    let duration = (case.duration_seconds * profile.duration_scale()).max(1.0 / 30.0);
    let mut command = engine_command(root, engine, profile, &case.id, &output)?;
    command
        .env("CINEKERNEL_RUN_ID", run_id)
        .env("CINEKERNEL_WIDTH", width.to_string())
        .env("CINEKERNEL_HEIGHT", height.to_string())
        .env("CINEKERNEL_FPS", "30")
        .env("CINEKERNEL_DURATION_SECONDS", duration.to_string())
        .env(
            "CINEKERNEL_FIXTURES",
            runtime_root(root).join("generated/fixtures"),
        );
    let started = Instant::now();
    let command_output = command.output().context("start benchmark engine")?;
    let total_ms = started.elapsed().as_secs_f64() * 1000.0;
    fs::write(
        &log_path,
        format!(
            "status: {}\nstdout:\n{}\nstderr:\n{}\n",
            command_output.status,
            String::from_utf8_lossy(&command_output.stdout),
            String::from_utf8_lossy(&command_output.stderr)
        ),
    )?;
    if warmup {
        return Ok(json!({"warmup": true, "exit_code": command_output.status.code()}));
    }
    let verify_started = Instant::now();
    let verification = verify_output(&output, duration, 30);
    let verify_ms = verify_started.elapsed().as_secs_f64() * 1000.0;
    let lock = load_upstream_lock(root)?;
    let (revision, dirty) = git_state(root);
    let (engine_version, upstream_commit) = match engine {
        Engine::Remotion => (
            lock.remotion.release_or_package_version,
            Some(lock.remotion.commit),
        ),
        Engine::Hyperframes => (
            lock.hyperframes.release_or_package_version,
            Some(lock.hyperframes.commit),
        ),
        Engine::Native2d => (env!("CARGO_PKG_VERSION").to_owned(), None),
        Engine::NativeWgpu => (env!("CARGO_PKG_VERSION").to_owned(), None),
    };
    let result = json!({
        "schema_version": "phase0.result.v1", "run_id": run_id, "timestamp_utc": Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true),
        "cinekernel_revision": revision, "cinekernel_dirty": dirty, "environment_id": environment_id,
        "engine": engine.as_str(), "engine_version": engine_version, "upstream_commit": upstream_commit,
        "case_id": case.id, "profile": profile.as_str(), "repetition": repetition,
        "configuration": {"width": width, "height": height, "fps": "30/1", "duration_seconds": duration, "codec": "h264", "pixel_format": "yuv420p", "worker_mode": std::env::var("CINEKERNEL_CONCURRENCY").or_else(|_| std::env::var("CINEKERNEL_WORKERS")).unwrap_or_else(|_| "default".to_owned())},
        "timings_ms": {"prepare": null, "compile": null, "initialize": null, "frame_production": null, "encode": null, "verify": verify_ms, "total": total_ms + verify_ms},
        "resources": {"peak_rss_bytes": null, "temporary_disk_bytes": directory_size(&directory), "output_bytes": fs::metadata(&output).ok().map(|m| m.len())},
        "capabilities": {"gpu_active": if engine == Engine::NativeWgpu {Some(true)} else {None}, "gpu_backend": null, "software_fallback": null},
        "verification": verification,
        "exit_code": command_output.status.code().unwrap_or(-1),
        "warnings": if command_output.stderr.is_empty() {Vec::<String>::new()} else {vec!["engine emitted stderr; inspect command.log".to_owned()]}
    });
    fs::write(
        directory.join("result.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    if !json_output {
        println!(
            "{} {} {} rep {}: {} ms, verified={}",
            engine.as_str(),
            case.id,
            profile.as_str(),
            repetition,
            total_ms.round(),
            result["verification"]["passed"]
        );
    }
    Ok(result)
}

fn engine_command(
    root: &Path,
    engine: Engine,
    profile: Profile,
    case_id: &str,
    output: &Path,
) -> Result<Command> {
    let mut command;
    match engine {
        Engine::Remotion | Engine::Hyperframes => {
            command = Command::new(pnpm_program());
            command
                .current_dir(root)
                .args([
                    "--filter",
                    if engine == Engine::Remotion {
                        "@cinekernel/phase0-remotion"
                    } else {
                        "@cinekernel/phase0-hyperframes"
                    },
                    "render",
                    "--",
                    "--case",
                    case_id,
                    "--profile",
                    profile.as_str(),
                    "--output",
                ])
                .arg(output);
        }
        Engine::Native2d | Engine::NativeWgpu => {
            command = Command::new(cargo_program());
            command
                .current_dir(root)
                .args([
                    "run",
                    "--release",
                    "--package",
                    if engine == Engine::Native2d {
                        "phase0-native-2d"
                    } else {
                        "phase0-native-wgpu"
                    },
                    "--",
                    "--case",
                    case_id,
                    "--profile",
                    profile.as_str(),
                    "--output",
                ])
                .arg(output);
        }
    }
    Ok(command)
}

fn verify_output(path: &Path, expected_duration: f64, fps: u64) -> Value {
    if !path.is_file() {
        return json!({"passed": false, "frame_count": null, "audio_tracks": null, "video_tracks": null, "duration_seconds": null, "sampled_frame_hashes": {}, "issues": ["output file missing"]});
    }
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-count_frames",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
        ])
        .arg(path)
        .output();
    let Ok(output) = output else {
        return json!({"passed": false, "issues": ["ffprobe could not start"]});
    };
    let parsed: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    let streams = parsed["streams"].as_array().cloned().unwrap_or_default();
    let video_tracks = streams
        .iter()
        .filter(|s| s["codec_type"] == "video")
        .count();
    let audio_tracks = streams
        .iter()
        .filter(|s| s["codec_type"] == "audio")
        .count();
    let duration = parsed["format"]["duration"]
        .as_str()
        .and_then(|value| value.parse::<f64>().ok());
    let frame_count = streams
        .iter()
        .find(|s| s["codec_type"] == "video")
        .and_then(|s| s["nb_read_frames"].as_str())
        .and_then(|value| value.parse::<u64>().ok());
    let expected_frames = (expected_duration * fps as f64).round() as u64;
    let mut issues = Vec::new();
    if video_tracks != 1 {
        issues.push(format!("expected one video track, found {video_tracks}"));
    }
    if duration.is_none_or(|actual| (actual - expected_duration).abs() > 0.12) {
        issues.push(format!(
            "duration mismatch: expected {expected_duration:.3}, found {duration:?}"
        ));
    }
    if frame_count.is_none_or(|actual| actual.abs_diff(expected_frames) > 1) {
        issues.push(format!(
            "frame count mismatch: expected {expected_frames}, found {frame_count:?}"
        ));
    }
    json!({"passed": issues.is_empty(), "frame_count": frame_count, "audio_tracks": audio_tracks, "video_tracks": video_tracks, "duration_seconds": duration, "sampled_frame_hashes": {}, "issues": issues, "ffprobe": parsed})
}

fn phase0_verify(root: &Path) -> AppResult<Value> {
    let runs = runtime_root(root).join("runs");
    let mut result_files = Vec::new();
    collect_named(&runs, "result.json", &mut result_files)
        .map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?;
    if result_files.is_empty() {
        return Err(Failure::new(
            EXIT_VERIFICATION_FAILURE,
            anyhow!("no benchmark results found under .cinekernel/runs"),
        ));
    }
    result_files.sort();
    let mut latest = std::collections::BTreeMap::<String, (PathBuf, Value)>::new();
    let mut retained_failures = 0_usize;
    for path in &result_files {
        let value: Value = serde_json::from_slice(
            &fs::read(path).map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?,
        )
        .map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?;
        if value["verification"]["passed"] != true || value["exit_code"].as_i64().unwrap_or(-1) != 0
        {
            retained_failures += 1;
        }
        let key = format!(
            "{}/{}/{}",
            value["engine"].as_str().unwrap_or("unknown"),
            value["case_id"].as_str().unwrap_or("unknown"),
            value["profile"].as_str().unwrap_or("unknown")
        );
        latest.insert(key, (path.clone(), value));
    }
    let failed: Vec<String> = latest
        .values()
        .filter(|(_, value)| {
            value["verification"]["passed"] != true
                || value["exit_code"].as_i64().unwrap_or(-1) != 0
        })
        .map(|(path, _)| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect();
    if !failed.is_empty() {
        return Err(Failure::new(
            EXIT_VERIFICATION_FAILURE,
            anyhow!(
                "{} result(s) failed verification: {}",
                failed.len(),
                failed.join(", ")
            ),
        ));
    }
    Ok(
        json!({"ok": true, "latest_groups_verified": latest.len(), "retained_result_count": result_files.len(), "retained_failed_attempts": retained_failures}),
    )
}

fn phase0_report(root: &Path) -> AppResult<Value> {
    let runs = runtime_root(root).join("runs");
    let mut paths = Vec::new();
    collect_named(&runs, "result.json", &mut paths)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let mut results = Vec::new();
    for path in paths {
        if let Ok(value) = serde_json::from_slice::<Value>(&fs::read(path).unwrap_or_default()) {
            results.push(value);
        }
    }
    let mut groups = std::collections::BTreeMap::<String, Vec<f64>>::new();
    for result in &results {
        if result["verification"]["passed"] != true
            || result["exit_code"].as_i64().unwrap_or(-1) != 0
        {
            continue;
        }
        if let (Some(engine), Some(case_id), Some(profile), Some(total)) = (
            result["engine"].as_str(),
            result["case_id"].as_str(),
            result["profile"].as_str(),
            result["timings_ms"]["total"].as_f64(),
        ) {
            groups
                .entry(format!("{engine}/{case_id}/{profile}"))
                .or_default()
                .push(total);
        }
    }
    let successful_result_count = results
        .iter()
        .filter(|result| {
            result["verification"]["passed"] == true
                && result["exit_code"].as_i64().unwrap_or(-1) == 0
        })
        .count();
    let failed_result_count = results.len() - successful_result_count;
    let summaries: Vec<Value> = groups.into_iter().map(|(key, mut samples)| { samples.sort_by(f64::total_cmp); let count = samples.len(); let mean = samples.iter().sum::<f64>() / count as f64; let median = if count % 2 == 0 {(samples[count/2-1]+samples[count/2])/2.0} else {samples[count/2]}; let variance = samples.iter().map(|value| (value-mean).powi(2)).sum::<f64>() / count as f64; json!({"key":key,"sample_count":count,"minimum_ms":samples[0],"median_ms":median,"maximum_ms":samples[count-1],"mean_ms":mean,"standard_deviation_ms":variance.sqrt(),"raw_ms":samples}) }).collect();
    let report_dir = root.join("reports/phase0");
    fs::create_dir_all(&report_dir)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let payload = json!({"schema_version":"phase0.aggregate.v1","generated_at_utc":Utc::now().to_rfc3339_opts(SecondsFormat::Millis,true),"result_count":results.len(),"successful_result_count":successful_result_count,"failed_result_count":failed_result_count,"summaries":summaries,"raw_results":results});
    fs::write(
        report_dir.join("BASELINE_RESULTS.json"),
        serde_json::to_vec_pretty(&payload).unwrap(),
    )
    .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let mut markdown = format!("# Phase 0 baseline results\n\nGenerated from all retained `result.json` files under `.cinekernel/runs/`. Timing summaries include only verified successful attempts; all {failed_result_count} failed attempts remain in `BASELINE_RESULTS.json` under `raw_results`. Successful attempts: {successful_result_count}. Total retained attempts: {}.\n\n| Engine / case / profile | n | min ms | median ms | mean ms | max ms | stddev ms |\n|---|---:|---:|---:|---:|---:|---:|\n", results.len());
    for summary in payload["summaries"].as_array().unwrap_or(&Vec::new()) {
        markdown.push_str(&format!(
            "| {} | {} | {:.1} | {:.1} | {:.1} | {:.1} | {:.1} |\n",
            summary["key"].as_str().unwrap_or(""),
            summary["sample_count"],
            summary["minimum_ms"].as_f64().unwrap_or(0.0),
            summary["median_ms"].as_f64().unwrap_or(0.0),
            summary["mean_ms"].as_f64().unwrap_or(0.0),
            summary["maximum_ms"].as_f64().unwrap_or(0.0),
            summary["standard_deviation_ms"].as_f64().unwrap_or(0.0)
        ));
    }
    fs::write(report_dir.join("BASELINE_RESULTS.md"), markdown)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    Ok(
        json!({"ok":true,"result_count":payload["result_count"],"json":"reports/phase0/BASELINE_RESULTS.json","markdown":"reports/phase0/BASELINE_RESULTS.md"}),
    )
}

fn clean_generated(root: &Path) -> AppResult<Value> {
    let target = runtime_root(root).join("generated");
    ensure_safe_generated_path(root, &target)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    if target.exists() {
        fs::remove_dir_all(&target)
            .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    }
    Ok(json!({"ok":true,"removed":".cinekernel/generated","recoverable":false}))
}

fn checked(root: &Path, program: &str, args: &[OsString], failure_code: u8) -> AppResult<()> {
    checked_program(root, OsStr::new(program), args, failure_code)
}

fn checked_program(
    root: &Path,
    program: &OsStr,
    args: &[OsString],
    failure_code: u8,
) -> AppResult<()> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| Failure::new(failure_code, error))?;
    if !output.status.success() {
        return Err(Failure::new(
            failure_code,
            anyhow!(
                "{} {} failed with {}\nstdout:\n{}\nstderr:\n{}",
                program.to_string_lossy(),
                args.iter()
                    .map(|a| a.to_string_lossy())
                    .collect::<Vec<_>>()
                    .join(" "),
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    Ok(())
}

fn output_text(command: &mut Command) -> Result<String> {
    let output = command.output()?;
    if !output.status.success() {
        bail!(
            "command failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn collect_named(directory: &Path, name: &str, output: &mut Vec<PathBuf>) -> Result<()> {
    if !directory.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_named(&path, name, output)?;
        } else if path.file_name() == Some(OsStr::new(name)) {
            output.push(path);
        }
    }
    Ok(())
}
