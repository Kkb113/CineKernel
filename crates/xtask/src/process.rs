use anyhow::{Context, Result};
use phase0_common::directory_size;
use serde::Serialize;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, Serialize)]
pub struct ProcessOutcome {
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub stalled: bool,
    pub elapsed_ms: f64,
    pub peak_rss_bytes: Option<u64>,
    pub peak_temporary_disk_bytes: u64,
    pub stdout: String,
    pub stderr: String,
    pub child_json: Option<Value>,
    pub termination: Option<String>,
}

enum Message {
    Stdout(String),
    Stderr(String),
    Closed,
}

pub fn run_supervised(
    command: &mut Command,
    timeout: Duration,
    stall_timeout: Duration,
    heartbeat: Duration,
    working_directory: &Path,
    measurement_directory: &Path,
) -> Result<ProcessOutcome> {
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(working_directory);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let started = Instant::now();
    let mut child = command
        .spawn()
        .context("spawn supervised benchmark child")?;
    let pid = child.id();
    let stdout = child.stdout.take().context("child stdout missing")?;
    let stderr = child.stderr.take().context("child stderr missing")?;
    let (sender, receiver) = mpsc::channel();
    spawn_reader(stdout, sender.clone(), true);
    spawn_reader(stderr, sender, false);

    let mut stdout_text = String::new();
    let mut stderr_text = String::new();
    let mut last_activity = Instant::now();
    let mut last_heartbeat = Instant::now();
    let mut peak_rss = None;
    let mut peak_temporary_disk = 0_u64;
    let mut sys = System::new();
    let mut timed_out = false;
    let mut stalled = false;
    let mut termination = None;
    let status: ExitStatus;
    loop {
        drain(
            &receiver,
            &mut stdout_text,
            &mut stderr_text,
            &mut last_activity,
        );
        refresh_resources(&mut sys, pid, &mut peak_rss);
        peak_temporary_disk = peak_temporary_disk.max(directory_size(measurement_directory));
        if let Some(exit) = child.try_wait()? {
            status = exit;
            break;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            termination = Some(terminate_tree(&mut child, pid, "timeout")?);
            status = child.wait()?;
            break;
        }
        if last_activity.elapsed() >= stall_timeout {
            stalled = true;
            termination = Some(terminate_tree(&mut child, pid, "stall")?);
            status = child.wait()?;
            break;
        }
        if last_heartbeat.elapsed() >= heartbeat {
            let heartbeat_line = format!(
                "[cinekernel heartbeat] pid={pid} elapsed_s={:.1} rss_bytes={} temp_bytes={}\n",
                started.elapsed().as_secs_f64(),
                peak_rss.map_or_else(|| "unknown".to_owned(), |value| value.to_string()),
                peak_temporary_disk
            );
            stderr_text.push_str(&heartbeat_line);
            eprint!("{heartbeat_line}");
            last_heartbeat = Instant::now();
        }
        thread::sleep(Duration::from_millis(100));
    }
    drain_until_closed(
        &receiver,
        &mut stdout_text,
        &mut stderr_text,
        &mut last_activity,
    );
    let child_json = parse_last_json(&stdout_text);
    Ok(ProcessOutcome {
        exit_code: status.code(),
        timed_out,
        stalled,
        elapsed_ms: started.elapsed().as_secs_f64() * 1000.0,
        peak_rss_bytes: peak_rss,
        peak_temporary_disk_bytes: peak_temporary_disk,
        stdout: stdout_text,
        stderr: stderr_text,
        child_json,
        termination,
    })
}

fn spawn_reader<R: std::io::Read + Send + 'static>(
    reader: R,
    sender: mpsc::Sender<Message>,
    stdout: bool,
) {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            let Ok(line) = line else { break };
            let message = if stdout {
                Message::Stdout(line)
            } else {
                Message::Stderr(line)
            };
            if sender.send(message).is_err() {
                return;
            }
        }
        let _ = sender.send(Message::Closed);
    });
}

fn drain(
    receiver: &Receiver<Message>,
    stdout: &mut String,
    stderr: &mut String,
    last_activity: &mut Instant,
) {
    loop {
        match receiver.try_recv() {
            Ok(Message::Stdout(line)) => {
                stdout.push_str(&line);
                stdout.push('\n');
                println!("{line}");
                *last_activity = Instant::now();
            }
            Ok(Message::Stderr(line)) => {
                stderr.push_str(&line);
                stderr.push('\n');
                eprintln!("{line}");
                *last_activity = Instant::now();
            }
            Ok(Message::Closed) => {}
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
        }
    }
}

fn drain_until_closed(
    receiver: &Receiver<Message>,
    stdout: &mut String,
    stderr: &mut String,
    last_activity: &mut Instant,
) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        drain(receiver, stdout, stderr, last_activity);
        match receiver.try_recv() {
            Err(TryRecvError::Disconnected) => break,
            Ok(message) => match message {
                Message::Stdout(line) => {
                    stdout.push_str(&line);
                    stdout.push('\n');
                }
                Message::Stderr(line) => {
                    stderr.push_str(&line);
                    stderr.push('\n');
                }
                Message::Closed => {}
            },
            Err(TryRecvError::Empty) => thread::sleep(Duration::from_millis(20)),
        }
    }
}

fn refresh_resources(system: &mut System, root_pid: u32, peak: &mut Option<u64>) {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::new().with_memory(),
    );
    let root = Pid::from_u32(root_pid);
    let mut total = 0_u64;
    for (pid, process) in system.processes() {
        if *pid == root || is_descendant(system, *pid, root) {
            total = total.saturating_add(process.memory());
        }
    }
    *peak = Some(peak.unwrap_or_default().max(total));
}

fn is_descendant(system: &System, mut pid: Pid, root: Pid) -> bool {
    for _ in 0..64 {
        let Some(parent) = system.process(pid).and_then(sysinfo::Process::parent) else {
            return false;
        };
        if parent == root {
            return true;
        }
        if parent == pid {
            return false;
        }
        pid = parent;
    }
    false
}

fn parse_last_json(output: &str) -> Option<Value> {
    output
        .lines()
        .rev()
        .find_map(|line| serde_json::from_str(line.trim()).ok())
}

fn terminate_tree(child: &mut std::process::Child, pid: u32, reason: &str) -> Result<String> {
    #[cfg(windows)]
    {
        let graceful = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/T"])
            .output();
        if child.try_wait()?.is_none() {
            thread::sleep(Duration::from_millis(500));
        }
        let forced = if child.try_wait()?.is_none() {
            Some(
                Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/T", "/F"])
                    .output()?,
            )
        } else {
            None
        };
        let graceful_text = graceful
            .ok()
            .map(|value| String::from_utf8_lossy(&value.stdout).trim().to_owned())
            .unwrap_or_default();
        let forced_text = forced
            .map(|value| String::from_utf8_lossy(&value.stdout).trim().to_owned())
            .unwrap_or_default();
        Ok(format!(
            "{reason}: taskkill tree graceful=[{graceful_text}] forced=[{forced_text}]"
        ))
    }
    #[cfg(unix)]
    {
        let group = format!("-{}", pid);
        let _ = Command::new("kill").args(["-TERM", "--", &group]).status();
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if child.try_wait()?.is_some() {
                return Ok(format!("{reason}: process group terminated with SIGTERM"));
            }
            thread::sleep(Duration::from_millis(50));
        }
        let status = Command::new("kill")
            .args(["-KILL", "--", &group])
            .status()?;
        if !status.success() {
            anyhow::bail!("failed to kill process group {group}");
        }
        Ok(format!("{reason}: process group required SIGKILL"))
    }
}

pub fn write_log(path: PathBuf, outcome: &ProcessOutcome) -> Result<()> {
    std::fs::write(
        path,
        format!(
            "exit_code: {:?}\ntimed_out: {}\nstalled: {}\nelapsed_ms: {:.3}\ntermination: {:?}\nstdout:\n{}\nstderr:\n{}",
            outcome.exit_code, outcome.timed_out, outcome.stalled, outcome.elapsed_ms, outcome.termination, outcome.stdout, outcome.stderr
        ),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sleeping_command(seconds: u64) -> Command {
        #[cfg(windows)]
        {
            let mut command = Command::new("powershell.exe");
            command.args([
                "-NoProfile",
                "-Command",
                &format!("Start-Sleep -Seconds {seconds}"),
            ]);
            command
        }
        #[cfg(unix)]
        {
            let mut command = Command::new("sh");
            command.args(["-c", &format!("sleep {seconds}")]);
            command
        }
    }

    #[test]
    fn child_json_uses_last_structured_line() {
        let output = "noise\n{\"ok\":false}\nmore\n{\"ok\":true,\"value\":3}\n";
        assert_eq!(parse_last_json(output).expect("json")["value"], 3);
    }

    #[test]
    fn supervised_child_times_out_and_records_tree_termination() {
        let measurement =
            std::env::temp_dir().join(format!("cinekernel-timeout-test-{}", std::process::id()));
        std::fs::create_dir_all(&measurement).expect("measurement directory");
        let outcome = run_supervised(
            &mut sleeping_command(10),
            Duration::from_millis(250),
            Duration::from_secs(5),
            Duration::from_secs(5),
            &measurement,
            &measurement,
        )
        .expect("supervised timeout");
        assert!(outcome.timed_out);
        assert!(!outcome.stalled);
        assert!(outcome
            .termination
            .as_deref()
            .is_some_and(|value| value.contains("timeout")));
        assert!(outcome.elapsed_ms < 5_000.0);
        std::fs::remove_dir_all(measurement).expect("cleanup measurement directory");
    }

    #[test]
    fn supervised_child_stall_is_distinct_from_wall_timeout() {
        let measurement =
            std::env::temp_dir().join(format!("cinekernel-stall-test-{}", std::process::id()));
        std::fs::create_dir_all(&measurement).expect("measurement directory");
        let outcome = run_supervised(
            &mut sleeping_command(10),
            Duration::from_secs(5),
            Duration::from_millis(250),
            Duration::from_secs(5),
            &measurement,
            &measurement,
        )
        .expect("supervised stall");
        assert!(!outcome.timed_out);
        assert!(outcome.stalled);
        assert!(outcome
            .termination
            .as_deref()
            .is_some_and(|value| value.contains("stall")));
        std::fs::remove_dir_all(measurement).expect("cleanup measurement directory");
    }
}
