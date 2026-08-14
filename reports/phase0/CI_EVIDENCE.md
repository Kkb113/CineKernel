# Phase 0.1 CI evidence

## Previous failure

The Phase 0 workflow run `31777834639` failed on Windows, Ubuntu, and macOS because `actions/setup-node` attempted pnpm cache resolution before pnpm was installed. That run is retained as failure evidence and is not acceptance evidence.

## Remediation

Both workflows install pnpm 11.8.0 before `actions/setup-node`, use the repository Node version, install with a frozen lockfile, install valid Rust tooling, run strict formatting/lint/tests, and upload evidence for 90 days. Normal CI runs on all three operating systems. The manual workflow runs canonical selections on all three operating systems and runs Probes A–J for `full/all`.

## Required remote records

| Gate | Revision | Run ID / URL | Result |
|---|---|---|---|
| Normal CI Windows/Linux/macOS | pending | pending | pending |
| Manual canonical full/all Windows/Linux/macOS | pending | pending | pending |
| Ubuntu loopback-only network namespace Probe G | pending | pending | pending |

This report must not claim remote success until actual GitHub Actions conclusions and artifact records are inserted.
