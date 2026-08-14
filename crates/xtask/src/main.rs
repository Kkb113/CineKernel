use anyhow::{anyhow, bail, Context, Result};
use chrono::{SecondsFormat, Utc};
use clap::{Args, Parser, Subcommand, ValueEnum};
use phase0_common::{
    capture_environment, cargo_program, directory_size, ensure_safe_generated_path, git_state,
    load_benchmark_spec, load_upstream_lock, new_run_id, pnpm_program, repository_root,
    runtime_root, Upstream,
};
use phase0_verifier::{artifact_manifest_path, hash_file, verify, VerifyRequest};
use serde_json::{json, Value};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant};

mod process;
use process::{run_supervised, write_log};

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
    CanonicalRun(RunArgs),
    Verify(CanonicalArgs),
    VerifyArtifact(VerifyArtifactArgs),
    Report(CanonicalArgs),
    Probes(CanonicalArgs),
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
    #[arg(long, default_value_t = 3600)]
    timeout_seconds: u64,
    #[arg(long, default_value_t = 900)]
    stall_seconds: u64,
    #[arg(long)]
    worker_mode: Option<String>,
}

#[derive(Debug, Args)]
struct CanonicalArgs {
    #[arg(long)]
    canonical: bool,
}

#[derive(Debug, Args)]
struct VerifyArtifactArgs {
    #[arg(long)]
    output: PathBuf,
    #[arg(long = "case")]
    case_id: String,
    #[arg(long, value_enum)]
    profile: Profile,
    #[arg(long, value_enum)]
    engine: Engine,
    #[arg(long)]
    expect_invalid: bool,
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
        } => phase0_run(&root, args, cli.json, false),
        TopCommand::Phase0 {
            command: Phase0Command::CanonicalRun(args),
        } => phase0_run(&root, args, cli.json, true),
        TopCommand::Phase0 {
            command: Phase0Command::Verify(args),
        } => phase0_verify(&root, args.canonical),
        TopCommand::Phase0 {
            command: Phase0Command::VerifyArtifact(args),
        } => phase0_verify_artifact(&root, args),
        TopCommand::Phase0 {
            command: Phase0Command::Report(args),
        } => phase0_report(&root, args.canonical),
        TopCommand::Phase0 {
            command: Phase0Command::Probes(args),
        } => phase0_probes(&root, args.canonical),
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
        let tree = output_text(Command::new("git").args([
            "-C",
            &target.to_string_lossy(),
            "rev-parse",
            "HEAD^{tree}",
        ]))
        .map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?;
        let license_hash = hash_file(&license).ok();
        let ok = actual == upstream.commit
            && tree == upstream.source_tree_git_sha
            && license_hash.as_deref() == Some(upstream.license_sha256.as_str());
        values.insert(name.to_owned(), json!({"ok": ok, "expected": upstream.commit, "actual": actual, "source_tree_expected":upstream.source_tree_git_sha,"source_tree_actual":tree,"release_tag":upstream.release_tag,"release_commit":upstream.release_commit,"source_commits_ahead":upstream.source_commits_ahead,"package_integrity":upstream.package_registry_integrity,"license": upstream.license_file, "license_present": license.is_file(),"license_sha256_expected":upstream.license_sha256,"license_sha256_actual":license_hash}));
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

fn phase0_run(root: &Path, args: &RunArgs, json_output: bool, canonical: bool) -> AppResult<Value> {
    let spec = load_benchmark_spec(root)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let (implementation_revision, dirty) = git_state(root);
    if canonical && !canonical_revision_is_eligible(&implementation_revision, dirty) {
        return Err(Failure::new(
            EXIT_INVALID_CONFIGURATION,
            anyhow!("canonical runs require a clean committed worktree; revision={implementation_revision}, dirty={dirty}"),
        ));
    }
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
    let spec_hash = hash_file(&root.join("benchmarks/specs/phase0-cases.json"))
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let lock_hash = hash_file(&root.join("benchmarks/upstreams.lock.json"))
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let engines: Vec<Engine> = args
        .engine
        .map_or_else(|| Engine::ALL.to_vec(), |engine| vec![engine]);
    let mut groups = Vec::new();
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
            let repetitions = repetition_count(args.profile, &case.id);
            let modes = worker_modes(engine, &case.id, args.profile, args.worker_mode.as_deref())
                .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
            for mode in modes {
                groups.push((engine, case, mode, repetitions));
            }
        }
    }
    if groups.is_empty() {
        return Err(Failure::new(
            EXIT_INVALID_CONFIGURATION,
            anyhow!("benchmark selection matched no supported engine/case groups"),
        ));
    }
    let expected_result_count: u64 = groups
        .iter()
        .map(|(_, _, _, repetitions)| *repetitions)
        .sum();
    let expected_groups: Vec<Value> = groups.iter().map(|(engine, case, mode, repetitions)| json!({
        "engine":engine.as_str(), "case_id":case.id, "worker_mode":mode, "repetitions":repetitions,
        "warmups":usize::from(args.profile==Profile::Full),
        "equivalence_level":case.equivalence.get(engine.as_str()).cloned().unwrap_or_else(||"unsupported".to_owned())
    })).collect();
    let manifest_path = run_dir.join("canonical-run-manifest.json");
    let started_at = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
    if canonical {
        write_json(&manifest_path, &json!({
            "schema_version":"phase0.canonical-run.v1", "canonical_run_id":run_id, "status":"running",
            "implementation_revision":implementation_revision, "worktree_clean":true, "environment_id":environment_id,
            "benchmark_spec_sha256":spec_hash, "upstream_lock_sha256":lock_hash, "profile":args.profile.as_str(),
            "engine_selection":args.engine.map(Engine::as_str).unwrap_or("all"), "case_selection":args.case_id,
            "timeout_seconds":args.timeout_seconds, "stall_seconds":args.stall_seconds, "heartbeat_seconds":15,
            "started_at_utc":started_at, "expected_result_count":expected_result_count, "expected_groups":expected_groups,
        })).map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION,error))?;
    }
    let mut results = Vec::new();
    let mut failures = 0_u64;
    for (engine, case, mode, repetitions) in groups {
        if args.profile == Profile::Full {
            match execute_run(
                root,
                &run_id,
                engine,
                args.profile,
                case,
                0,
                &mode,
                &environment_id,
                &implementation_revision,
                &spec_hash,
                &lock_hash,
                true,
                canonical,
                json_output,
                args.timeout_seconds,
                args.stall_seconds,
            ) {
                Ok(value) if successful_run(&value) => {}
                Ok(value) => {
                    failures += 1;
                    results.push(json!({"warmup_failure":value}));
                    continue;
                }
                Err(error) => {
                    failures += 1;
                    results.push(json!({"engine":engine.as_str(),"case_id":case.id,"worker_mode":mode,"warmup":true,"harness_error":format!("{error:#}")}));
                    continue;
                }
            }
        }
        for repetition in 1..=repetitions {
            match execute_run(
                root,
                &run_id,
                engine,
                args.profile,
                case,
                repetition,
                &mode,
                &environment_id,
                &implementation_revision,
                &spec_hash,
                &lock_hash,
                false,
                canonical,
                json_output,
                args.timeout_seconds,
                args.stall_seconds,
            ) {
                Ok(value) => {
                    if !successful_run(&value) {
                        failures += 1;
                    }
                    results.push(value);
                }
                Err(error) => {
                    failures += 1;
                    results.push(json!({"engine":engine.as_str(),"case_id":case.id,"profile":args.profile.as_str(),"worker_mode":mode,"repetition":repetition,"harness_error":format!("{error:#}")}));
                }
            }
        }
    }
    let summary = json!({"schema_version":"phase0.run-summary.v2","canonical_run_id":run_id,"canonical":canonical,"failures":failures,"result_count":results.iter().filter(|result|result.get("schema_version").is_some()).count(),"expected_result_count":expected_result_count,"results":results});
    write_json(&run_dir.join("run-summary.json"), &summary)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let actual_result_count = summary["result_count"].as_u64().unwrap_or_default();
    let canonical_complete =
        canonical_run_is_complete(failures, actual_result_count, expected_result_count);
    if canonical {
        let completed_manifest = json!({
            "schema_version":"phase0.canonical-run.v1", "canonical_run_id":run_id,
            "status":if canonical_complete {"complete"} else {"failed"}, "implementation_revision":implementation_revision,
            "worktree_clean":true, "environment_id":environment_id, "benchmark_spec_sha256":spec_hash,
            "upstream_lock_sha256":lock_hash, "profile":args.profile.as_str(),
            "engine_selection":args.engine.map(Engine::as_str).unwrap_or("all"), "case_selection":args.case_id,
            "timeout_seconds":args.timeout_seconds, "stall_seconds":args.stall_seconds, "heartbeat_seconds":15,
            "started_at_utc":started_at, "completed_at_utc":Utc::now().to_rfc3339_opts(SecondsFormat::Millis,true),
            "expected_result_count":expected_result_count, "actual_result_count":actual_result_count, "failure_count":failures,
            "expected_groups":expected_groups,
        });
        write_json(&manifest_path, &completed_manifest)
            .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
        if canonical_complete {
            let pointer_dir = runtime_root(root).join("canonical");
            fs::create_dir_all(&pointer_dir)
                .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
            write_json(&pointer_dir.join("latest.json"),&json!({"canonical_run_id":run_id,"manifest":format!(".cinekernel/runs/{run_id}/canonical-run-manifest.json")})).map_err(|error|Failure::new(EXIT_INVALID_CONFIGURATION,error))?;
        }
    }
    if failures > 0 || actual_result_count != expected_result_count {
        return Err(Failure::new(EXIT_BENCHMARK_FAILURE, anyhow!("{failures} benchmark group/repetition failure(s), {actual_result_count}/{expected_result_count} measured results; evidence preserved under .cinekernel/runs/{run_id}")));
    }
    Ok(
        json!({"ok":true,"canonical":canonical,"canonical_run_id":run_id,"result_count":summary["result_count"],"expected_result_count":expected_result_count,"path":format!(".cinekernel/runs/{run_id}")}),
    )
}

fn canonical_run_is_complete(failures: u64, actual: u64, expected: u64) -> bool {
    failures == 0 && actual == expected
}

fn canonical_revision_is_eligible(revision: &str, dirty: bool) -> bool {
    !dirty
        && revision.len() == 40
        && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
        && revision != "UNBORN"
}

fn repetition_count(profile: Profile, case_id: &str) -> u64 {
    match profile {
        Profile::Smoke => 1,
        Profile::Full if case_id == "mixed-2d-3d" => 3,
        Profile::Full => 5,
    }
}

fn successful_run(value: &Value) -> bool {
    value["exit_code"].as_i64() == Some(0)
        && value["timed_out"] != true
        && value["verification"]["passed"] == true
}

fn worker_modes(
    engine: Engine,
    case_id: &str,
    profile: Profile,
    requested: Option<&str>,
) -> Result<Vec<String>> {
    if let Some(mode) = requested {
        if !["default", "auto", "1", "4"].contains(&mode) {
            bail!("unsupported worker mode {mode}");
        }
        return Ok(vec![mode.to_owned()]);
    }
    if profile == Profile::Full && case_id == "media-frame-sampling" {
        return Ok(match engine {
            Engine::Remotion => ["default", "1", "4"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            Engine::Hyperframes => ["auto", "1", "4"].into_iter().map(str::to_owned).collect(),
            _ => vec!["default".to_owned()],
        });
    }
    Ok(vec![if engine == Engine::Hyperframes {
        "auto"
    } else {
        "default"
    }
    .to_owned()])
}

#[allow(clippy::too_many_arguments)]
fn execute_run(
    root: &Path,
    run_id: &str,
    engine: Engine,
    profile: Profile,
    case: &phase0_common::BenchmarkCase,
    repetition: u64,
    worker_mode: &str,
    environment_id: &str,
    implementation_revision: &str,
    benchmark_spec_sha256: &str,
    upstream_lock_sha256: &str,
    warmup: bool,
    canonical: bool,
    json_output: bool,
    timeout_seconds: u64,
    stall_seconds: u64,
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
        .join(format!(
            "worker-{}",
            worker_mode.replace(|character: char| !character.is_ascii_alphanumeric(), "-")
        ))
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
    match engine {
        Engine::Remotion if worker_mode != "default" => {
            command.env("CINEKERNEL_CONCURRENCY", worker_mode);
        }
        Engine::Hyperframes if worker_mode != "auto" => {
            command.env("CINEKERNEL_WORKERS", worker_mode);
        }
        _ => {}
    }
    let outcome = run_supervised(
        &mut command,
        Duration::from_secs(timeout_seconds),
        Duration::from_secs(stall_seconds),
        Duration::from_secs(15),
        root,
        &directory,
    )?;
    write_log(log_path, &outcome)?;
    if outcome.timed_out || outcome.stalled {
        write_json(
            &directory.join("failure.json"),
            &json!({"canonical_run_id":run_id,"engine":engine.as_str(),"case_id":case.id,"worker_mode":worker_mode,"repetition":repetition,"warmup":warmup,"timed_out":outcome.timed_out,"stalled":outcome.stalled,"termination":outcome.termination,"partial_output_present":output.exists(),"partial_output_bytes":fs::metadata(&output).ok().map(|metadata|metadata.len()),"valid":false}),
        )?;
        bail!(
            "benchmark child {} for {}/{}",
            if outcome.timed_out {
                "timed out"
            } else {
                "stalled"
            },
            engine.as_str(),
            case.id
        );
    }
    let child_json = outcome
        .child_json
        .clone()
        .context("benchmark child did not emit a parseable final JSON object")?;
    let verify_started = Instant::now();
    let verification = verify(&VerifyRequest {
        output: &output,
        fixtures: &runtime_root(root).join("generated/fixtures"),
        case_id: &case.id,
        engine: engine.as_str(),
        width,
        height,
        fps: 30,
        duration_seconds: duration,
        expected_audio_tracks: case.expected_audio_tracks,
    });
    let verify_ms = verify_started.elapsed().as_secs_f64() * 1000.0;
    write_json(
        &artifact_manifest_path(&directory.join("result.json")),
        &json!({"schema_version":"phase0.verification-manifest.v1","canonical_run_id":run_id,"engine":engine.as_str(),"case_id":case.id,"worker_mode":worker_mode,"output":"output.mp4","output_sha256":if output.is_file(){hash_file(&output).ok()}else{None},"verification":verification}),
    )?;
    let lock = load_upstream_lock(root)?;
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
    let child_timings = &child_json["timings_ms"];
    let frame_production = child_timings["frame_production"]
        .as_f64()
        .or_else(|| child_json["frame_production_ms"].as_f64());
    let encode = child_timings["encode"]
        .as_f64()
        .or_else(|| child_json["encode_ms"].as_f64());
    let render_command = child_timings["render_command"]
        .as_f64()
        .or(Some(outcome.elapsed_ms));
    let encoder = if child_json["encoder"].is_object() {
        child_json["encoder"].clone()
    } else {
        json!({"container":"mp4","video_codec":"h264","encoder":"libx264","pixel_format":"yuv420p","crf":18,"preset":"medium","audio_codec":if case.expected_audio_tracks>0 {Value::String("aac".to_owned())}else{Value::Null},"audio_bitrate":if case.expected_audio_tracks>0 {Value::String("192k".to_owned())}else{Value::Null},"sample_rate":if case.expected_audio_tracks>0 {Value::from(48000)}else{Value::Null},"channel_layout":if case.expected_audio_tracks>0 {Value::String("mono".to_owned())}else{Value::Null}})
    };
    let capabilities = json!({
        "gpu_active":if engine==Engine::NativeWgpu {Some(true)} else {None},
        "gpu_backend":child_json["backend"].as_str(),"gpu_adapter":child_json["adapter"].as_str(),
        "gpu_driver":child_json["driver"].as_str(),"software_fallback":child_json["software_fallback"].as_bool(),
        "capture_mode":if engine==Engine::Hyperframes {child_json["capture_requested"].as_str().or(Some("auto"))}else{None}
    });
    let equivalence_level = case
        .equivalence
        .get(engine.as_str())
        .cloned()
        .unwrap_or_else(|| "unsupported".to_owned());
    let warnings = if outcome.stderr.trim().is_empty() {
        Vec::<String>::new()
    } else {
        vec!["engine emitted stderr; inspect command.log".to_owned()]
    };
    let result = if canonical {
        json!({
            "schema_version":"phase0.result.v2","canonical_run_id":run_id,"canonical":true,
            "timestamp_utc":Utc::now().to_rfc3339_opts(SecondsFormat::Millis,true),"implementation_revision":implementation_revision,
            "worktree_clean":true,"environment_id":environment_id,"benchmark_spec_sha256":benchmark_spec_sha256,
            "upstream_lock_sha256":upstream_lock_sha256,"engine":engine.as_str(),"engine_version":engine_version,
            "upstream_commit":upstream_commit,"case_id":case.id,"profile":profile.as_str(),"repetition":repetition,
            "warmup":warmup,"equivalence_level":equivalence_level,
            "configuration":{"width":width,"height":height,"fps":"30/1","duration_seconds":duration,"worker_mode":worker_mode,"frame_order":"sequential"},
            "timings_ms":{"preflight":child_timings["preflight"],"project_prepare":child_timings["project_prepare"],"engine_startup":child_timings["engine_startup"],"frame_production":frame_production,"encode":encode,"render_command":render_command,"artifact_verify":verify_ms,"end_to_end":outcome.elapsed_ms+verify_ms},
            "resources":{"peak_rss_bytes":outcome.peak_rss_bytes,"peak_temporary_disk_bytes":outcome.peak_temporary_disk_bytes,"maximum_queued_frame_bytes":Value::Null,"output_bytes":fs::metadata(&output).ok().map(|metadata|metadata.len())},
            "capabilities":capabilities,"encoder":encoder,"verification":verification,"exit_code":outcome.exit_code.unwrap_or(-1),"timed_out":false,"warnings":warnings
        })
    } else {
        json!({"schema_version":"phase0.result.v1","run_id":run_id,"timestamp_utc":Utc::now().to_rfc3339_opts(SecondsFormat::Millis,true),"cinekernel_revision":implementation_revision,"cinekernel_dirty":git_state(root).1,"environment_id":environment_id,"engine":engine.as_str(),"engine_version":engine_version,"upstream_commit":upstream_commit,"case_id":case.id,"profile":profile.as_str(),"repetition":repetition,"configuration":{"width":width,"height":height,"fps":"30/1","duration_seconds":duration,"worker_mode":worker_mode},"timings_ms":{"prepare":child_timings["project_prepare"],"compile":Value::Null,"initialize":child_timings["engine_startup"],"frame_production":frame_production,"encode":encode,"verify":verify_ms,"total":outcome.elapsed_ms+verify_ms},"resources":{"peak_rss_bytes":outcome.peak_rss_bytes,"temporary_disk_bytes":outcome.peak_temporary_disk_bytes,"output_bytes":fs::metadata(&output).ok().map(|metadata|metadata.len())},"capabilities":capabilities,"encoder":encoder,"verification":verification,"exit_code":outcome.exit_code.unwrap_or(-1),"timed_out":false,"warnings":warnings})
    };
    if canonical {
        validate_canonical_result(root, &result)?;
    }
    fs::write(
        directory.join(if warmup {
            "warmup-result.json"
        } else {
            "result.json"
        }),
        serde_json::to_vec_pretty(&result)?,
    )?;
    if !json_output {
        println!(
            "{} {} {} rep {}: {} ms, verified={}",
            engine.as_str(),
            case.id,
            profile.as_str(),
            repetition,
            outcome.elapsed_ms.round(),
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

fn validate_canonical_result(root: &Path, result: &Value) -> Result<()> {
    let schema: Value = serde_json::from_slice(&fs::read(
        root.join("schemas/phase0/benchmark-result.schema.json"),
    )?)?;
    let validator =
        jsonschema::validator_for(&schema).context("compile canonical result schema")?;
    validator
        .validate(result)
        .map_err(|error| anyhow!("canonical result schema validation failed: {error}"))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}

fn phase0_verify(root: &Path, canonical: bool) -> AppResult<Value> {
    let runs = if canonical {
        latest_canonical_directory(root)
            .map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?
    } else {
        runtime_root(root).join("runs")
    };
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
    if canonical {
        let manifest: Value = serde_json::from_slice(
            &fs::read(runs.join("canonical-run-manifest.json"))
                .map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?,
        )
        .map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?;
        if manifest["status"] != "complete" || manifest["failure_count"] != 0 {
            return Err(Failure::new(
                EXIT_VERIFICATION_FAILURE,
                anyhow!("canonical manifest is not complete and failure-free"),
            ));
        }
        let expected = manifest["expected_result_count"]
            .as_u64()
            .unwrap_or_default() as usize;
        if result_files.len() != expected {
            return Err(Failure::new(
                EXIT_VERIFICATION_FAILURE,
                anyhow!(
                    "canonical matrix incomplete: expected {expected} results, found {}",
                    result_files.len()
                ),
            ));
        }
        let mut groups = std::collections::BTreeMap::<String, usize>::new();
        for path in &result_files {
            let value: Value = serde_json::from_slice(
                &fs::read(path).map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?,
            )
            .map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?;
            validate_canonical_result(root, &value)
                .map_err(|error| Failure::new(EXIT_VERIFICATION_FAILURE, error))?;
            if value["canonical_run_id"] != manifest["canonical_run_id"]
                || value["implementation_revision"] != manifest["implementation_revision"]
                || value["worktree_clean"] != true
                || value["implementation_revision"] == "UNBORN"
            {
                return Err(Failure::new(
                    EXIT_VERIFICATION_FAILURE,
                    anyhow!("canonical identity mismatch in {}", path.display()),
                ));
            }
            if value["verification"]["passed"] != true
                || value["exit_code"] != 0
                || value["timed_out"] != false
            {
                return Err(Failure::new(
                    EXIT_VERIFICATION_FAILURE,
                    anyhow!("canonical result failed in {}", path.display()),
                ));
            }
            let key = format!(
                "{}/{}/{}",
                value["engine"].as_str().unwrap_or("?"),
                value["case_id"].as_str().unwrap_or("?"),
                value["configuration"]["worker_mode"]
                    .as_str()
                    .unwrap_or("?")
            );
            *groups.entry(key).or_default() += 1;
        }
        for group in manifest["expected_groups"].as_array().into_iter().flatten() {
            let key = format!(
                "{}/{}/{}",
                group["engine"].as_str().unwrap_or("?"),
                group["case_id"].as_str().unwrap_or("?"),
                group["worker_mode"].as_str().unwrap_or("?")
            );
            let expected_repetitions = group["repetitions"].as_u64().unwrap_or_default() as usize;
            if groups.get(&key).copied() != Some(expected_repetitions) {
                return Err(Failure::new(EXIT_VERIFICATION_FAILURE,anyhow!("canonical group {key} expected {expected_repetitions} repetitions, found {:?}",groups.get(&key))));
            }
        }
        return Ok(
            json!({"ok":true,"canonical":true,"canonical_run_id":manifest["canonical_run_id"],"implementation_revision":manifest["implementation_revision"],"result_count":result_files.len(),"group_count":groups.len(),"matrix_complete":true}),
        );
    }
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
        json!({"ok": true, "canonical":false,"latest_groups_verified": latest.len(), "retained_result_count": result_files.len(), "retained_failed_attempts": retained_failures}),
    )
}

fn phase0_verify_artifact(root: &Path, args: &VerifyArtifactArgs) -> AppResult<Value> {
    let spec = load_benchmark_spec(root)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let case = spec
        .cases
        .iter()
        .find(|case| case.id == args.case_id)
        .ok_or_else(|| {
            Failure::new(
                EXIT_INVALID_CONFIGURATION,
                anyhow!("unknown benchmark case {}", args.case_id),
            )
        })?;
    if !case
        .supported_engines
        .iter()
        .any(|engine| engine == args.engine.as_str())
    {
        return Err(Failure::new(
            EXIT_UNSUPPORTED_CAPABILITY,
            anyhow!("{} does not support {}", args.engine.as_str(), args.case_id),
        ));
    }
    let (width, height) = args.profile.dimensions();
    let duration = (case.duration_seconds * args.profile.duration_scale()).max(1.0 / 30.0);
    let report = verify(&VerifyRequest {
        output: &args.output,
        fixtures: &runtime_root(root).join("generated/fixtures"),
        case_id: &case.id,
        engine: args.engine.as_str(),
        width,
        height,
        fps: 30,
        duration_seconds: duration,
        expected_audio_tracks: case.expected_audio_tracks,
    });
    let rejected = !report.passed;
    let value = json!({"ok":if args.expect_invalid{rejected}else{report.passed},"expect_invalid":args.expect_invalid,"rejected":rejected,"output":args.output,"verification":report});
    if (args.expect_invalid && rejected) || (!args.expect_invalid && !rejected) {
        Ok(value)
    } else {
        Err(Failure::new(
            EXIT_VERIFICATION_FAILURE,
            anyhow!(
                "artifact verification expectation failed: {}",
                serde_json::to_string(&value).unwrap_or_default()
            ),
        ))
    }
}

fn phase0_report(root: &Path, canonical: bool) -> AppResult<Value> {
    let runs = if canonical {
        latest_canonical_directory(root)
            .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?
    } else {
        runtime_root(root).join("runs")
    };
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
            result["timings_ms"][if canonical { "render_command" } else { "total" }].as_f64(),
        ) {
            if canonical && result["equivalence_level"] != "equivalent" {
                continue;
            }
            groups
                .entry(format!(
                    "{engine}/{case_id}/{profile}/{}",
                    result["configuration"]["worker_mode"]
                        .as_str()
                        .unwrap_or("default")
                ))
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
    let manifest = if canonical {
        serde_json::from_slice::<Value>(
            &fs::read(runs.join("canonical-run-manifest.json")).unwrap_or_default(),
        )
        .unwrap_or(Value::Null)
    } else {
        Value::Null
    };
    let payload = json!({"schema_version":if canonical{"phase0.canonical-aggregate.v1"}else{"phase0.aggregate.v1"},"canonical":canonical,"canonical_run_id":manifest["canonical_run_id"],"implementation_revision":manifest["implementation_revision"],"generated_at_utc":Utc::now().to_rfc3339_opts(SecondsFormat::Millis,true),"timing_view":"render_command (preflight and artifact verification excluded)","equivalent_workloads_only":canonical,"result_count":results.len(),"successful_result_count":successful_result_count,"failed_result_count":failed_result_count,"summaries":summaries,"raw_results":results});
    let json_name = if canonical {
        "CANONICAL_BASELINE_RESULTS.json"
    } else {
        "BASELINE_RESULTS.json"
    };
    let markdown_name = if canonical {
        "CANONICAL_BASELINE_RESULTS.md"
    } else {
        "BASELINE_RESULTS.md"
    };
    fs::write(
        report_dir.join(json_name),
        serde_json::to_vec_pretty(&payload).unwrap(),
    )
    .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let mut markdown = format!("# Phase 0.1 {}baseline results\n\n{} Timing view: render-command elapsed time; HyperFrames lint/check and all artifact verification are reported separately. Direct rows include only `equivalence_level: equivalent`. Failed attempts: {failed_result_count}. Successful attempts: {successful_result_count}. Total retained attempts in this evidence set: {}.\n\n| Engine / case / profile / worker | n | min ms | median ms | mean ms | max ms | stddev ms |\n|---|---:|---:|---:|---:|---:|---:|\n",if canonical{"canonical "}else{""},if canonical{format!("Canonical run `{}` at implementation `{}`.",manifest["canonical_run_id"].as_str().unwrap_or("unknown"),manifest["implementation_revision"].as_str().unwrap_or("unknown"))}else{"Historical retained results; not canonical.".to_owned()}, results.len());
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
    fs::write(report_dir.join(markdown_name), markdown)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    if canonical {
        fs::copy(
            runs.join("canonical-run-manifest.json"),
            report_dir.join("CANONICAL_RUN_MANIFEST.json"),
        )
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
        let historical = historical_summary(root)
            .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
        fs::write(report_dir.join("HISTORICAL_RESULTS_SUMMARY.md"), historical)
            .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    }
    Ok(
        json!({"ok":true,"canonical":canonical,"canonical_run_id":payload["canonical_run_id"],"result_count":payload["result_count"],"json":format!("reports/phase0/{json_name}"),"markdown":format!("reports/phase0/{markdown_name}")}),
    )
}

fn latest_canonical_directory(root: &Path) -> Result<PathBuf> {
    let pointer: Value = serde_json::from_slice(
        &fs::read(runtime_root(root).join("canonical/latest.json"))
            .context("no canonical run pointer; run canonical-run first")?,
    )?;
    let id = pointer["canonical_run_id"]
        .as_str()
        .context("canonical run id missing")?;
    Ok(runtime_root(root).join("runs").join(id))
}

fn historical_summary(root: &Path) -> Result<String> {
    let mut paths = Vec::new();
    collect_named(&runtime_root(root).join("runs"), "result.json", &mut paths)?;
    let mut v1 = 0;
    let mut v2 = 0;
    let mut failed = 0;
    for path in paths {
        if let Ok(value) = serde_json::from_slice::<Value>(&fs::read(path)?) {
            match value["schema_version"].as_str() {
                Some("phase0.result.v1") => v1 += 1,
                Some("phase0.result.v2") => v2 += 1,
                _ => {}
            };
            if value["verification"]["passed"] != true || value["exit_code"] != 0 {
                failed += 1;
            }
        }
    }
    Ok(format!("# Historical results summary\n\nRetained historical results are never merged into canonical aggregates.\n\n- Phase 0 v1 retained attempts: {v1}\n- Phase 0.1 v2 attempts across all local canonical runs: {v2}\n- Retained failed attempts: {failed}\n- Canonical selection source: `.cinekernel/canonical/latest.json` only.\n"))
}

fn phase0_probes(root: &Path, canonical: bool) -> AppResult<Value> {
    if !canonical {
        return Err(Failure::new(
            EXIT_INVALID_CONFIGURATION,
            anyhow!("Phase 0.1 probes require --canonical"),
        ));
    }
    let directory = latest_canonical_directory(root)
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let id = directory
        .file_name()
        .and_then(OsStr::to_str)
        .context("canonical id missing")
        .map_err(|error| Failure::new(EXIT_INVALID_CONFIGURATION, error))?;
    let output = Command::new(pnpm_program())
        .current_dir(root)
        .args([
            "--filter",
            "@cinekernel/phase0-common",
            "probes",
            "--",
            "--canonical-run-id",
            id,
        ])
        .output()
        .map_err(|error| Failure::new(EXIT_BENCHMARK_FAILURE, error))?;
    if !output.status.success() {
        return Err(Failure::new(
            EXIT_BENCHMARK_FAILURE,
            anyhow!(
                "probe suite failed with {}\nstdout:\n{}\nstderr:\n{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }
    let value = String::from_utf8_lossy(&output.stdout)
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str::<Value>(line).ok())
        .context("probe suite emitted no JSON")
        .map_err(|error| Failure::new(EXIT_BENCHMARK_FAILURE, error))?;
    Ok(value)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("workspace root")
            .to_path_buf()
    }

    fn temporary_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("cinekernel-{label}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).expect("create test root");
        path
    }

    #[test]
    fn canonical_revision_enforcement_rejects_dirty_unborn_and_malformed_state() {
        let revision = "a".repeat(40);
        assert!(canonical_revision_is_eligible(&revision, false));
        assert!(!canonical_revision_is_eligible(&revision, true));
        assert!(!canonical_revision_is_eligible("UNBORN", false));
        assert!(!canonical_revision_is_eligible("not-a-full-sha", false));
    }

    #[test]
    fn canonical_pointer_requires_zero_failures_and_exact_inventory() {
        assert!(canonical_run_is_complete(0, 109, 109));
        assert!(!canonical_run_is_complete(1, 109, 109));
        assert!(!canonical_run_is_complete(0, 108, 109));
        assert!(!canonical_run_is_complete(0, 110, 109));
    }

    #[test]
    fn repetition_and_worker_matrix_matches_phase_0_1_contract() {
        assert_eq!(repetition_count(Profile::Smoke, "mixed-2d-3d"), 1);
        assert_eq!(repetition_count(Profile::Full, "mixed-2d-3d"), 3);
        assert_eq!(repetition_count(Profile::Full, "typography-layout"), 5);
        assert_eq!(
            worker_modes(
                Engine::Remotion,
                "media-frame-sampling",
                Profile::Full,
                None
            )
            .expect("modes"),
            ["default", "1", "4"]
        );
        assert_eq!(
            worker_modes(
                Engine::Hyperframes,
                "media-frame-sampling",
                Profile::Full,
                None
            )
            .expect("modes"),
            ["auto", "1", "4"]
        );
        assert!(worker_modes(
            Engine::Remotion,
            "media-frame-sampling",
            Profile::Full,
            Some("unbounded")
        )
        .is_err());
    }

    #[test]
    fn required_full_matrix_has_109_measured_results() {
        let spec = load_benchmark_spec(&workspace_root()).expect("benchmark spec");
        let mut count = 0_u64;
        for engine in Engine::ALL {
            for case in &spec.cases {
                if case
                    .supported_engines
                    .iter()
                    .any(|candidate| candidate == engine.as_str())
                {
                    let modes =
                        worker_modes(engine, &case.id, Profile::Full, None).expect("worker modes");
                    count += modes.len() as u64 * repetition_count(Profile::Full, &case.id);
                }
            }
        }
        assert_eq!(count, 109);
    }

    #[test]
    fn warmup_and_measured_success_require_verifier_and_no_timeout() {
        assert!(successful_run(
            &json!({"exit_code":0,"timed_out":false,"verification":{"passed":true}})
        ));
        assert!(!successful_run(
            &json!({"exit_code":0,"timed_out":true,"verification":{"passed":true}})
        ));
        assert!(!successful_run(
            &json!({"exit_code":0,"timed_out":false,"verification":{"passed":false}})
        ));
    }

    #[test]
    fn canonical_pointer_selects_only_the_named_run() {
        let root = temporary_root("canonical-selection");
        let pointer = runtime_root(&root).join("canonical/latest.json");
        write_json(
            &pointer,
            &json!({"canonical_run_id":"selected","manifest":"ignored"}),
        )
        .expect("pointer");
        fs::create_dir_all(runtime_root(&root).join("runs/older")).expect("older");
        let selected = latest_canonical_directory(&root).expect("selected directory");
        assert_eq!(selected, runtime_root(&root).join("runs/selected"));
        fs::remove_dir_all(root).expect("cleanup test root");
    }

    #[test]
    fn historical_summary_separates_v1_v2_and_failures() {
        let root = temporary_root("historical-separation");
        write_json(
            &runtime_root(&root).join("runs/a/result.json"),
            &json!({"schema_version":"phase0.result.v1","exit_code":0,"verification":{"passed":true}}),
        )
        .expect("v1");
        write_json(
            &runtime_root(&root).join("runs/b/result.json"),
            &json!({"schema_version":"phase0.result.v2","exit_code":3,"verification":{"passed":false}}),
        )
        .expect("v2");
        let summary = historical_summary(&root).expect("summary");
        assert!(summary.contains("v1 retained attempts: 1"));
        assert!(summary.contains("v2 attempts across all local canonical runs: 1"));
        assert!(summary.contains("Retained failed attempts: 1"));
        fs::remove_dir_all(root).expect("cleanup test root");
    }

    #[test]
    fn result_v2_serialization_is_schema_compatible() {
        let result = json!({
            "schema_version":"phase0.result.v2","canonical_run_id":"run","canonical":true,
            "timestamp_utc":"2026-08-14T00:00:00.000Z","implementation_revision":"a".repeat(40),
            "worktree_clean":true,"environment_id":"b".repeat(64),"benchmark_spec_sha256":"c".repeat(64),
            "upstream_lock_sha256":"d".repeat(64),"engine":"native-2d","engine_version":"0.0.0",
            "upstream_commit":null,"case_id":"typography-layout","profile":"smoke","repetition":1,
            "warmup":false,"equivalence_level":"equivalent","configuration":{},
            "timings_ms":{"preflight":null,"project_prepare":null,"engine_startup":null,"frame_production":1,"encode":1,"render_command":2,"artifact_verify":1,"end_to_end":3},
            "resources":{"peak_rss_bytes":null,"peak_temporary_disk_bytes":1,"maximum_queued_frame_bytes":null,"output_bytes":1},
            "capabilities":{"gpu_active":null,"gpu_backend":null,"gpu_adapter":null,"gpu_driver":null,"software_fallback":null,"capture_mode":null},
            "encoder":{},"verification":{"passed":true,"issues":[]},"exit_code":0,"timed_out":false,"warnings":[]
        });
        validate_canonical_result(&workspace_root(), &result).expect("schema compatible");
        let mut dirty = result;
        dirty["worktree_clean"] = Value::Bool(false);
        assert!(validate_canonical_result(&workspace_root(), &dirty).is_err());
    }
}
