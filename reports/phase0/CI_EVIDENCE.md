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

## Manual benchmark workflow blocker

`.github/workflows/phase0-benchmarks.yml` exists in the repository and defines the required Windows/Ubuntu/macOS matrix, 90-day evidence uploads, full/all selection, and Ubuntu `unshare --net` Probe G. GitHub currently lists only the normal CI workflow because this new workflow has not been registered on the default branch.

Dispatch attempts were deliberately non-destructive and failed as follows:

- `gh workflow run phase0-benchmarks.yml --ref phase/0.1-review-remediation`: HTTP 404 (workflow not registered).
- Dispatching the registered CI workflow: HTTP 422 because that workflow has no `workflow_dispatch` trigger.

The prompt prohibits merging or changing `master` without explicit instruction. Therefore the manual workflow was not forced onto the default branch. A reviewer/maintainer must merge or otherwise register it, then dispatch `selection=full`, `probes=all`, and retain the resulting three-OS artifacts.

| Gate | Revision | Record | Result |
|---|---|---|---|
| Normal CI, three OS | implementation A | run `31810436296` | PASS |
| Post-evidence CI, three OS | attestation `8b4825e` | run `31817123555` | PASS |
| Manual canonical full/all, three OS | not dispatchable | GitHub workflow registry 404 | BLOCKED |
| Ubuntu loopback-only Probe G | depends on manual workflow | required `sudo unshare --net` execution | BLOCKED |
