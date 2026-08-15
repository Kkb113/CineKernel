# CineKernel Phase 0.1 acceptance report

## 1. Executive status

**PASS.** Implementation, clean canonical and focused performance evidence, permanent artifact verification, correctness probes, source archaeology, three-OS CI, the capability-aware remote full/all matrix, macOS retained-evidence attestation, and Ubuntu OS-enforced Probe G are complete. The former default-branch registration and macOS Probe D blockers are resolved. Exact remote records are in `REMOTE_CLOSURE_ATTESTATION.md`.

## 2. Implementation revision

The original aggregate baseline uses `0249b40ec41673ed8ac2f22c23583ddc3629a320` (revision A). The native 3D floor closure implementation is `907a2551c3dad27c698ac43d7ecb41957236be53`, with the required focused 3D/mixed reruns recorded in `PHASE0_1_CLOSURE_EVIDENCE.md`. The remote capability-aware matrix uses `6f254eda880ab5a1463baac1d0a1819b7c68cac7`. Final probe/workflow closure is on master at `1c07e19fb0eb9b9f9c4b7c5e3cc26b6a29e54a93`; these later probe/workflow changes do not alter canonical renderer output or performance timing.

## 3. Evidence revision

`b12e6c74a519fa693a49f50cd23df6dddc910b88` (revision B) contains the original generated canonical JSON/Markdown, probes, manifest, and contact sheets. Remote closure adds retained artifacts from runs `31855973437`, `31855975438`, and `31870436549`; identifiers, sizes, hashes, and validity boundaries are recorded in `REMOTE_CLOSURE_ATTESTATION.md`.

## 4. Branch

Accepted on `master` in `https://github.com/Kkb113/CineKernel.git`. Closure fixes and attestation prerequisites were reviewed and merged through PRs #9 and #10; no history rewrite or force-push was used.

## 5. Upstream commits and package relationships

- Remotion source `4e459b8b3aeec12ac8346666773ea28892a30e31`, tree `26ad029579076f7eedb87b0aca57a997846dfda1`, package/tag `4.0.509` / `v4.0.509`, release commit `6ef4fbb15937540aec723d618780a4b9d6ef133c`: pinned source is nine commits ahead.
- HyperFrames source `532caf7aa24fef382cb103013f6414bb547a4129`, tree `9c8c93a4a5c5e286093eaa9596457408a7124bf5`, package/tag `0.7.108` / `v0.7.108`, release/package commit `9ba528914dafc05c54e2191c19f99847c8f4420a`: pinned source is two commits ahead.
- Registry integrity, license path/hash, source trees, release relationships, and sparse paths are recorded in `benchmarks/upstreams.lock.json`.

## 6. Local environment

Windows 11; Intel Core Ultra 7; 32 GB RAM; Intel Arc Graphics; driver `32.0.101.8724`; native wgpu backend Vulkan; no software fallback. Environment ID: `d3e630b8b937aa97c4e37c2360ce8a61b9a9436d9235e181c312228ba9998c2f`.

## 7. Remote CI matrix results

Normal CI run [31810436296](https://github.com/Kkb113/CineKernel/actions/runs/31810436296) at revision A passed on Ubuntu, macOS, and Windows. Current master CI run [31870422891](https://github.com/Kkb113/CineKernel/actions/runs/31870422891) also passed: Windows job `94978096773`, Ubuntu `94978096796`, macOS `94978096803`.

## 8. Manual workflow result

**PASS (composite closure).** The capability-aware full/all workflow ran at source revision `6f254eda880ab5a1463baac1d0a1819b7c68cac7`: Windows and Ubuntu jobs passed completely; macOS rendered and verified 99/99 outputs before its original Probe D browser-lifecycle failure. The corrected retained-evidence attestation reran macOS probes A-F and H-J with 9/9 PASS, and the dedicated Ubuntu workflow passed Probe G for both browser engines under a loopback-only network namespace. Because no canonical renderer, fixture, benchmark intent, timing boundary, or verifier changed, the existing performance results remain valid.

## 9. Canonical run ID

Reference-machine full: `20260814T144948Z-c6e0a98a-b94a-48e9-a26e-f69faf10f048`. Remote full/all: macOS `20260815T011700Z-244d1132-6dcb-4c66-a8fb-12c8a11b9168` (99/99), Ubuntu `20260815T011746Z-6b283441-5101-4379-b3c4-88a6e98a9bf6` (101/101), Windows `20260815T012122Z-790e3e20-25bd-4574-8b73-0a9fb0956a4b` (101/101).

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

Recorded diagnostic failures remain part of the audit history. An early `upstream sync` failure, a report-command typo, default-branch workflow registration, FFmpeg package-feed failures, and the original macOS per-still WebGL context exhaustion were each corrected and followed by green regression evidence. No failed diagnostic run is used as acceptance evidence.

## 13. Test results

Rust formatting/check/strict Clippy pass; 26 Rust tests pass. Frozen pnpm install, TypeScript typecheck, and the current 27 TypeScript tests pass. Tests cover real locks/manifests/schemas, source-frame uniqueness, composition semantics, URL bans, revision enforcement, canonical selection, warm-up handling, timeout/stall/tree cleanup, verifier failures, FFmpeg compatibility, sparse sync idempotency, capability-aware GPU behavior, dedicated Probe G execution, reusable Remotion probe browsers, and retained-evidence attestation prerequisites.

## 14. Canonical benchmark results

The reference full run contains 109 successful measured attempts in 23 summaries and 23 passing warm-ups. Each row's `n`, min, median, mean, max, and standard deviation is in `CANONICAL_BASELINE_RESULTS.md`. Mixed medians: native wgpu 10,624.9 ms (`n=3`), HyperFrames 24,132.6 ms (`n=3`), Remotion 35,470.6 ms (`n=3`). The required native-floor focused reruns passed 24/24. The remote capability-aware matrix independently verified 99 macOS, 101 Ubuntu, and 101 Windows results. No unsupported speedup ratio is claimed.

## 15. Preflight versus render timing

Comparable charts use only `timings_ms.render_command`. HyperFrames lint/check preflight is separate: workload medians range 9,891.5-12,710.6 ms; artifact verification is also separate. The phase chart shows preflight/render/verify medians, and every raw v2 result retains end-to-end timing and explicit encoder limitations.

## 16. Workload equivalence status

Typography, vector, and chart are equivalent across both browser engines and native-2d. 3D and mixed are equivalent across both browser engines and native-wgpu. Media sampling and audio/captions are equivalent between Remotion and HyperFrames; native is explicitly unsupported for those standalone cases. No partial or feasibility-only row enters direct charts.

## 17. Artifact-verifier results

The central verifier passed 109/109 measured outputs, with codec/pixel format/dimensions/fps/timebase/timestamps/frame/audio/case semantics checked. `VERIFICATION_MANIFEST_INDEX.json` contains 109 sidecar SHA-256 entries; post-probe canonical verification remained 109/109.

## 18. Correctness probe results

A-F and H-J PASS with zero FAIL and zero UNSUPPORTED in the final macOS attestation. Probe G separately PASSes for Remotion and HyperFrames under Ubuntu `unshare --net` with loopback only and no unexpected external network availability. Probe A retains bounded WebGL tolerance where exact hashes are inappropriate. Probe I kills the actual xtask process tree and recovers. Probe J bounds real 1,920×1,080×4 buffers to three frames / 24,883,200 bytes.

## 19. Source-archaeology status

The required Remotion and HyperFrames critical paths are traced to concrete functions/types, owners, timing, concurrency, failure/retry/cache behavior, preview/final differences, tests, and CineKernel dispositions. Final renderer selection remains Proposed.

## 20. Source-lineage status

PASS: 31 documents and four inventory entries validate with immutable pinned-commit links and existing local references. Upstream sync is repeatable after the file-path sparse-checkout fix.

## 21. Licensing status

The root `LICENSE` contains the full Apache License 2.0 text. CineKernel application notice and third-party notices are preserved. Remotion and HyperFrames remain external wrapped dependencies; neither is claimed to be relicensed by CineKernel.

## 22. Failed or blocked gates

None. The former remote-workflow registration, macOS Probe D, and Ubuntu Probe G gates are closed by runs `31855973437`, `31870436549`, and `31855975438`. Historical failed attempts remain visible for diagnosis but are superseded by green evidence on the corrected paths.

## 23. Remaining risks

Production text shaping/i18n, cross-driver GPU conformance, certified FFmpeg distribution, cache invalidation, browser preview/final parity across platforms, and long-workload quality remain open. See `RISK_REGISTER.md`.

## 24. Open reviewer decisions

Reviewers may approve Phase 1 entry. Remaining decisions concern the production text stack, representative long-workload corpus, initial certified GPU matrix, and media/FFmpeg adoption boundaries. See `OPEN_DECISIONS.md`.

## 25. Recommendation for Phase 1

Proceed to Phase 1. The evidence supports native wgpu as an accelerated candidate, native software 2D as the deterministic reference, and both browser engines as wrapped compatibility backends. Phase 0.1 acceptance is not a final renderer certification: Phase 1 must preserve the documented support boundaries and address production text, codec/distribution, long-workload, and multi-adapter conformance risks before irreversible commitments.
