# CineKernel Phase 0.1 acceptance report

## 1. Executive status

**CONDITIONAL PASS.** Implementation, clean canonical evidence, artifact verification, locally executable correctness probes, archaeology, tests, and normal three-OS CI are complete. Unconditional acceptance is blocked because GitHub has not registered the manual benchmark workflow on `master`; the remote full/all matrix and Ubuntu OS-enforced Probe G therefore have not run.

## 2. Implementation revision

`0249b40ec41673ed8ac2f22c23583ddc3629a320` (revision A). No implementation-affecting change was made after this SHA; the canonical run records it in every result.

## 3. Evidence revision

`b12e6c74a519fa693a49f50cd23df6dddc910b88` (revision B) contains generated canonical JSON/Markdown, probes, manifest, and contact sheets. Later report-only attestation commits do not alter revision A or its measured data.

## 4. Branch

`phase/0.1-review-remediation` in `https://github.com/Kkb113/CineKernel.git`. No history rewrite, force-push, or `master` merge was performed.

## 5. Upstream commits and package relationships

- Remotion source `4e459b8b3aeec12ac8346666773ea28892a30e31`, tree `26ad029579076f7eedb87b0aca57a997846dfda1`, package/tag `4.0.509` / `v4.0.509`, release commit `6ef4fbb15937540aec723d618780a4b9d6ef133c`: pinned source is nine commits ahead.
- HyperFrames source `532caf7aa24fef382cb103013f6414bb547a4129`, tree `9c8c93a4a5c5e286093eaa9596457408a7124bf5`, package/tag `0.7.108` / `v0.7.108`, release/package commit `9ba528914dafc05c54e2191c19f99847c8f4420a`: pinned source is two commits ahead.
- Registry integrity, license path/hash, source trees, release relationships, and sparse paths are recorded in `benchmarks/upstreams.lock.json`.

## 6. Local environment

Windows 11; Intel Core Ultra 7; 32 GB RAM; Intel Arc Graphics; driver `32.0.101.8724`; native wgpu backend Vulkan; no software fallback. Environment ID: `d3e630b8b937aa97c4e37c2360ce8a61b9a9436d9235e181c312228ba9998c2f`.

## 7. Remote CI matrix results

Normal CI run [31810436296](https://github.com/Kkb113/CineKernel/actions/runs/31810436296) at revision A passed: Ubuntu job `94799485842`, macOS `94799485855`, Windows `94799485868`. Post-evidence run [31817123555](https://github.com/Kkb113/CineKernel/actions/runs/31817123555) also passed: Windows `94821378357`, macOS `94821378360`, Ubuntu `94821378363`; every job uploaded smoke evidence.

## 8. Manual workflow result

**BLOCKED / not executed.** `.github/workflows/phase0-benchmarks.yml` exists on the branch, but GitHub lists only normal CI because the workflow is not registered on the default branch. Dispatch returned HTTP 404. Registering it requires a default-branch change, which the prompt forbids without explicit instruction.

## 9. Canonical run ID

Full: `20260814T144948Z-c6e0a98a-b94a-48e9-a26e-f69faf10f048`. Clean smoke precursor: `20260814T144334Z-8850e74e-96e2-490c-a259-f02569135bb2` (19/19 groups verified).

## 10. Benchmark spec hash

`61e6e1eb1ca264b1c013f01896a91cbb759e67a4873d159874a6a68e48a90a1b`.

## 11. Upstream lock hash

`1c6da0d9d27697c270429cdb50d0942ee1b07171808e7024e0a932d312f9faa9`.

## 12. Commands and exit codes

All required commands used implementation revision A in the clean evidence worktree unless marked as report-only:

| Command | Exit | Result |
|---|---:|---|
| `cargo fmt --all --check` | 0 | PASS |
| `cargo check --workspace --all-targets --all-features` | 0 | PASS |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 | PASS |
| `cargo test --workspace --all-features` | 0 | PASS, 26 tests |
| `corepack pnpm install --frozen-lockfile` | 0 | PASS, pnpm 11.8.0 |
| `corepack pnpm typecheck` | 0 | PASS |
| `corepack pnpm test` | 0 | PASS, 20 tests |
| `corepack pnpm --filter @cinekernel/phase0-common lineage:validate` | 0 | PASS, 31 docs / 4 entries |
| `cargo xtask doctor --json` | 0 | PASS |
| `cargo xtask environment capture --json` | 0 | PASS |
| `cargo xtask upstream sync` | 0 | PASS twice after idempotency fix |
| `cargo xtask upstream verify --json` | 0 | PASS |
| `cargo xtask phase0 prepare --json` | 0 | PASS |
| `cargo xtask phase0 canonical-run --profile smoke --json` | 0 | PASS, 19/19 |
| `cargo xtask phase0 verify --canonical --json` (smoke) | 0 | PASS, 19/19 |
| `cargo xtask phase0 canonical-run --profile full --json` | 0 | PASS, 109/109, 2,929.2 s |
| `cargo xtask phase0 probes --canonical --json` | 0 | PASS, 0 failures / 1 unsupported |
| `cargo xtask phase0 verify --canonical --json` (post-probe) | 0 | PASS, 109/109 |
| `cargo xtask phase0 report --canonical --json` | 0 | PASS |

Recorded failed commands: an earlier final-candidate `upstream sync` exited 2 because sparse checkout treated `README.md` as a directory; revision A adds `--skip-checks`, a regression test, and two successful reruns. The report-only typo `cargo xtask report --canonical --json` exited 2 (`report` is nested under `phase0`); the required command above passed. Manual workflow dispatch returned HTTP 404; dispatching normal CI returned HTTP 422 because it intentionally has no `workflow_dispatch` trigger.

## 13. Test results

Rust formatting/check/strict Clippy pass; 26 Rust tests pass. Frozen pnpm install, TypeScript typecheck, and 20 TypeScript tests pass. Tests cover real locks/manifests/schemas, source-frame uniqueness, composition semantics, URL bans, revision enforcement, exact 109-result matrix, canonical selection, warm-up handling, timeout/stall/tree cleanup, verifier failures, FFmpeg option compatibility, and sparse sync idempotency.

## 14. Canonical benchmark results

The full run contains 109 successful measured attempts in 23 summaries and 23 passing warm-ups. Each row's `n`, min, median, mean, max, and standard deviation is in `CANONICAL_BASELINE_RESULTS.md`. Mixed medians: native wgpu 10,624.9 ms (`n=3`), HyperFrames 24,132.6 ms (`n=3`), Remotion 35,470.6 ms (`n=3`). No unsupported speedup ratio is claimed.

## 15. Preflight versus render timing

Comparable charts use only `timings_ms.render_command`. HyperFrames lint/check preflight is separate: workload medians range 9,891.5-12,710.6 ms; artifact verification is also separate. The phase chart shows preflight/render/verify medians, and every raw v2 result retains end-to-end timing and explicit encoder limitations.

## 16. Workload equivalence status

Typography, vector, and chart are equivalent across both browser engines and native-2d. 3D and mixed are equivalent across both browser engines and native-wgpu. Media sampling and audio/captions are equivalent between Remotion and HyperFrames; native is explicitly unsupported for those standalone cases. No partial or feasibility-only row enters direct charts.

## 17. Artifact-verifier results

The central verifier passed 109/109 measured outputs, with codec/pixel format/dimensions/fps/timebase/timestamps/frame/audio/case semantics checked. `VERIFICATION_MANIFEST_INDEX.json` contains 109 sidecar SHA-256 entries; post-probe canonical verification remained 109/109.

## 18. Correctness probe results

A-F and H-J PASS; zero FAIL. Probe G is UNSUPPORTED on Windows and requires the blocked Ubuntu manual workflow. Probe A exact hashes pass except the documented Remotion mixed WebGL tolerance row (minimum PSNR 37.184885 dB, SSIM 0.98721). Probe I kills the actual xtask process tree and recovers. Probe J bounds real 1,920×1,080×4 buffers to three frames / 24,883,200 bytes.

## 19. Source-archaeology status

The required Remotion and HyperFrames critical paths are traced to concrete functions/types, owners, timing, concurrency, failure/retry/cache behavior, preview/final differences, tests, and CineKernel dispositions. Final renderer selection remains Proposed.

## 20. Source-lineage status

PASS: 31 documents and four inventory entries validate with immutable pinned-commit links and existing local references. Upstream sync is repeatable after the file-path sparse-checkout fix.

## 21. Licensing status

The root `LICENSE` contains the full Apache License 2.0 text. CineKernel application notice and third-party notices are preserved. Remotion and HyperFrames remain external wrapped dependencies; neither is claimed to be relicensed by CineKernel.

## 22. Failed or blocked gates

Manual benchmark workflow execution, remote workflow artifacts, and Ubuntu strong network isolation are blocked by GitHub default-branch workflow registration. Therefore the status cannot be `PASS`.

## 23. Remaining risks

Production text shaping/i18n, cross-driver GPU conformance, certified FFmpeg distribution, cache invalidation, browser preview/final parity across platforms, and long-workload quality remain open. See `RISK_REGISTER.md`.

## 24. Open reviewer decisions

Reviewers must decide when/how to register the manual workflow, whether Phase 1 may begin under conditional status, the production text stack, the initial certified GPU matrix, and media/FFmpeg adoption boundaries. See `OPEN_DECISIONS.md`.

## 25. Recommendation for Phase 1

The evidence supports native wgpu as an accelerated candidate, native software 2D as the deterministic reference, and both browser engines as wrapped compatibility backends. Begin Phase 1 only as bounded planning/prototyping after acknowledging this conditional status; do not declare Phase 0 fully accepted or make irreversible renderer commitments until the manual three-OS workflow and Ubuntu Probe G are green.
