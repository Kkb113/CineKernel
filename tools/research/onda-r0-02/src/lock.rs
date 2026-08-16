use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::{fs, path::Path, process::Command};

#[derive(Clone, Debug, Deserialize)]
pub struct UpstreamLock {
    pub repository: String,
    pub pinned_commit: String,
    pub pinned_tree: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckoutObservation {
    pub remote: String,
    pub head: String,
    pub tree: String,
    pub detached: bool,
    pub clean: bool,
}

pub fn read(root: &Path) -> Result<UpstreamLock> {
    let path = root.join("docs/research/onda/r0.01/UPSTREAM_LOCK.json");
    let lock: UpstreamLock = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("reading {}", path.display()))?,
    )
    .context("R0.01 upstream lock is invalid")?;
    validate_hash("pinned_commit", &lock.pinned_commit)?;
    validate_hash("pinned_tree", &lock.pinned_tree)?;
    if normalize_repository(&lock.repository).is_empty() {
        bail!("R0.01 repository is empty")
    }
    Ok(lock)
}

pub fn observe(checkout: &Path) -> Result<CheckoutObservation> {
    let remote = git(checkout, &["remote", "get-url", "origin"])?;
    let head = git(checkout, &["rev-parse", "HEAD"])?;
    let tree = git(checkout, &["rev-parse", "HEAD^{tree}"])?;
    let clean = git(checkout, &["status", "--porcelain"])?.is_empty();
    let detached = Command::new("git")
        .args(["symbolic-ref", "-q", "HEAD"])
        .current_dir(checkout)
        .output()?
        .status
        .success()
        .not();
    Ok(CheckoutObservation {
        remote,
        head,
        tree,
        detached,
        clean,
    })
}

pub fn validate_observation(lock: &UpstreamLock, observed: &CheckoutObservation) -> Result<()> {
    if normalize_repository(&observed.remote) != normalize_repository(&lock.repository) {
        bail!("ONDA checkout remote mismatch")
    }
    if observed.head != lock.pinned_commit {
        bail!("ONDA checkout pin mismatch")
    }
    if observed.tree != lock.pinned_tree {
        bail!("ONDA checkout tree mismatch")
    }
    if !observed.detached {
        bail!("ONDA checkout is not detached")
    }
    if !observed.clean {
        bail!("ONDA checkout is dirty")
    }
    Ok(())
}

pub fn normalize_repository(value: &str) -> String {
    value
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .replace("git@github.com:", "https://github.com/")
        .to_ascii_lowercase()
}

fn validate_hash(label: &str, value: &str) -> Result<()> {
    if value.len() != 40 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        bail!("R0.01 {label} is not a full Git object ID")
    }
    Ok(())
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git").args(args).current_dir(root).output()?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_owned())
}

trait BoolNot {
    fn not(self) -> bool;
}

impl BoolNot for bool {
    fn not(self) -> bool {
        !self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock() -> UpstreamLock {
        UpstreamLock {
            repository: "https://github.com/onda-engine/onda-engine.git".into(),
            pinned_commit: "a".repeat(40),
            pinned_tree: "b".repeat(40),
        }
    }

    fn observation() -> CheckoutObservation {
        CheckoutObservation {
            remote: "https://github.com/onda-engine/onda-engine".into(),
            head: "a".repeat(40),
            tree: "b".repeat(40),
            detached: true,
            clean: true,
        }
    }

    #[test]
    fn repository_normalization_accepts_git_suffix() {
        assert_eq!(
            normalize_repository("https://github.com/ONDA-ENGINE/onda-engine.git"),
            "https://github.com/onda-engine/onda-engine"
        );
    }

    #[test]
    fn checkout_rejects_pin_mismatch() {
        let mut value = observation();
        value.head = "c".repeat(40);
        assert!(validate_observation(&lock(), &value).is_err());
    }

    #[test]
    fn checkout_rejects_tree_mismatch() {
        let mut value = observation();
        value.tree = "c".repeat(40);
        assert!(validate_observation(&lock(), &value).is_err());
    }

    #[test]
    fn checkout_rejects_wrong_remote() {
        let mut value = observation();
        value.remote = "https://github.com/onda-video/onda".into();
        assert!(validate_observation(&lock(), &value).is_err());
    }

    #[test]
    fn checkout_rejects_attached_head() {
        let mut value = observation();
        value.detached = false;
        assert!(validate_observation(&lock(), &value).is_err());
    }

    #[test]
    fn checkout_rejects_dirty_state() {
        let mut value = observation();
        value.clean = false;
        assert!(validate_observation(&lock(), &value).is_err());
    }
}
