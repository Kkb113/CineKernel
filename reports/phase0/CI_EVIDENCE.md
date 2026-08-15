# Phase 0.1 CI evidence

## Green normal CI

Normal CI is green on implementation revision `0249b40ec41673ed8ac2f22c23583ddc3629a320`:

| OS | Job ID | Conclusion |
|---|---:|---|
| Ubuntu | `94799485842` | success |
| macOS | `94799485855` | success |
| Windows | `94799485868` | success |

Workflow run: [31810436296](https://github.com/Kkb113/CineKernel/actions/runs/31810436296). This supersedes earlier failed remediation runs while retaining them as diagnostic history.

Post-evidence/report CI is also green at attestation revision `8b4825e2657eda163a5984c9a3a0ca7182841a89`:

| OS | Job ID | Conclusion |
|---|---:|---|
| Windows | `94821378357` | success |
| macOS | `94821378360` | success |
| Ubuntu | `94821378363` | success |

Workflow run: [31817123555](https://github.com/Kkb113/CineKernel/actions/runs/31817123555). All three jobs uploaded smoke evidence successfully.

## Retained failure history

- Run `31777834639`: all jobs failed because `actions/setup-node` attempted pnpm cache resolution before pnpm was installed.
- Later remediation runs exposed and fixed pnpm ordering, async probe entry, FFmpeg action incompatibility on macOS ARM, deprecated FFmpeg `-vsync`, wgpu case timeout, and sparse-checkout file-path idempotency.
- A local mistaken invocation, `cargo xtask report --canonical --json`, exited 2 because `report` is nested under `phase0`; `cargo xtask phase0 report --canonical --json` then passed and generated the committed reports.

## Remote Phase 0.1 closure workflows

The required workflows are registered on `master` and have been dispatched.

| Gate | Revision | Record | Result |
|---|---|---|---|
| Capability-aware canonical full/all, three OS | `6f254eda880ab5a1463baac1d0a1819b7c68cac7` | [run 31855973437](https://github.com/Kkb113/CineKernel/actions/runs/31855973437) | Windows/Ubuntu complete PASS; macOS render + 99/99 verification PASS, original Probe D failure superseded below |
| Ubuntu loopback-only Probe G | `6f254eda880ab5a1463baac1d0a1819b7c68cac7` | [run 31855975438](https://github.com/Kkb113/CineKernel/actions/runs/31855975438) | PASS for Remotion and HyperFrames |
| macOS retained canonical probes A-F, H-J | `1c07e19fb0eb9b9f9c4b7c5e3cc26b6a29e54a93` | [run 31870436549](https://github.com/Kkb113/CineKernel/actions/runs/31870436549) | 9 PASS / 0 FAIL / 0 UNSUPPORTED |
| Current master CI, three OS | `1c07e19fb0eb9b9f9c4b7c5e3cc26b6a29e54a93` | [run 31870422891](https://github.com/Kkb113/CineKernel/actions/runs/31870422891) | PASS |

The canonical run retains 90-day artifacts for all three operating systems. The dedicated Probe G and macOS attestation workflows retain their own evidence artifacts. Exact job IDs, artifact IDs, sizes, generated timestamps, and SHA-256 hashes are recorded in `REMOTE_CLOSURE_ATTESTATION.md`.

## Final CI disposition

**PASS.** Earlier registration, package-feed, workflow, and macOS Probe D failures remain in GitHub as diagnostic history. They are not acceptance evidence and are superseded by the green runs above.
