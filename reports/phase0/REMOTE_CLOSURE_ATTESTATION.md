# Phase 0.1 remote closure attestation

Status: **PASS**

This document records the remote evidence that closes the Phase 0.1 conditional gates. The retained performance matrix was produced from source revision `6f254eda880ab5a1463baac1d0a1819b7c68cac7`. Later changes affect probe execution and workflow reliability only; they do not change benchmark intent, fixtures, canonical renderer output, timing boundaries, or the artifact verifier. The retained performance run therefore remains valid.

## Canonical full/all matrix

Workflow run: [31855973437](https://github.com/Kkb113/CineKernel/actions/runs/31855973437).

| OS | Job | Canonical run | Verified results | Performance/verification | Probe disposition |
|---|---:|---|---:|---|---|
| macOS | `94940702065` | `20260815T011700Z-244d1132-6dcb-4c66-a8fb-12c8a11b9168` | 99/99 | PASS | original Probe D failure superseded by attestation run `31870436549` |
| Ubuntu | `94940702086` | `20260815T011746Z-6b283441-5101-4379-b3c4-88a6e98a9bf6` | 101/101 | PASS | A-F and H-J PASS |
| Windows | `94940702112` | `20260815T012122Z-790e3e20-25bd-4574-8b73-0a9fb0956a4b` | 101/101 | PASS | A-F and H-J PASS |

The workflow-level conclusion is `failure` only because the original macOS Probe D step opened a fresh browser for every still and exhausted WebGL contexts after the already-successful render and 99/99 verification steps. No canonical output failed.

Retained 90-day artifacts:

| OS | Artifact ID | Artifact name | Size |
|---|---:|---|---:|
| macOS | `9239686529` | `phase0-6f254eda880ab5a1463baac1d0a1819b7c68cac7-macos-latest-full-all` | 129,545,635 bytes |
| Ubuntu | `9240027330` | `phase0-6f254eda880ab5a1463baac1d0a1819b7c68cac7-ubuntu-latest-full-all` | 92,776,027 bytes |
| Windows | `9240241838` | `phase0-6f254eda880ab5a1463baac1d0a1819b7c68cac7-windows-latest-full-all` | 94,666,993 bytes |

## macOS retained-evidence probe attestation

Workflow run: [31870436549](https://github.com/Kkb113/CineKernel/actions/runs/31870436549), master revision `1c07e19fb0eb9b9f9c4b7c5e3cc26b6a29e54a93`.

- Reused macOS canonical run `20260815T011700Z-244d1132-6dcb-4c66-a8fb-12c8a11b9168` and verified its retained inventory before probing.
- Probes A, B, C, D, E, F, H, I, and J: **9 PASS / 0 FAIL / 0 UNSUPPORTED**.
- Probe report generated at `2026-08-15T06:56:39.227Z`.
- Artifact ID `9243323463`, name `phase0-macos-probe-attestation-1c07e19fb0eb9b9f9c4b7c5e3cc26b6a29e54a93`, size 24,252,609 bytes.
- `CORRECTNESS_PROBES.json` SHA-256: `260fb6efb93d0ac344286bb2f9200198743b384287786061af7f57c581eea08e`.

The Probe D remediation bundles once and reuses one Remotion browser, reopening it only after an actual capture failure. This matches the lifecycle used by the successful canonical renderer while retaining strict failure semantics.

## Probe G network isolation

Workflow run: [31855975438](https://github.com/Kkb113/CineKernel/actions/runs/31855975438), job `94940707719`, source revision `6f254eda880ab5a1463baac1d0a1819b7c68cac7`.

- Linux network namespace with loopback only.
- Unexpected external network availability: `false`.
- Remotion: exit 0, PASS.
- HyperFrames: exit 0, PASS.
- Artifact ID `9239053004`, name `phase0-probe-g-6f254eda880ab5a1463baac1d0a1819b7c68cac7-ubuntu-latest`, size 122,587 bytes.
- `probe-g.json` SHA-256: `586667e4344ca1aa4b29d5b984ed905564ed5eafa2a80ade41a7af853521595f`.

## Current master CI

Workflow run: [31870422891](https://github.com/Kkb113/CineKernel/actions/runs/31870422891), master revision `1c07e19fb0eb9b9f9c4b7c5e3cc26b6a29e54a93`.

| OS | Job | Result |
|---|---:|---|
| Windows | `94978096773` | PASS |
| Ubuntu | `94978096796` | PASS |
| macOS | `94978096803` | PASS |

## Closure decision

The remote full/all performance and verification matrix, macOS correctness attestation, Ubuntu OS-enforced Probe G, focused native-floor reruns, and current three-OS CI all pass. Phase 0.1 is accepted. Phase 1 may proceed within the candidate architecture and the open production risks documented in `RISK_REGISTER.md`.
