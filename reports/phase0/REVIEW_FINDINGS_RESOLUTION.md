# Phase 0.1 reviewer findings resolution

Implementation revision A is `0249b40ec41673ed8ac2f22c23583ddc3629a320`; generated evidence revision B is `b12e6c74a519fa693a49f50cd23df6dddc910b88`; canonical run is `20260814T144948Z-c6e0a98a-b94a-48e9-a26e-f69faf10f048`.

## Finding 1 — GitHub CI failed

- **Original problem:** pnpm cache lookup preceded pnpm installation; Rust/FFmpeg setup and acceptance claims were unreliable.
- **Root cause:** workflow ordering and platform assumptions had not been validated remotely.
- **Implementation change:** pnpm 11.8.0 installs first; Rust components use valid syntax; each OS uses a platform-appropriate FFmpeg installer; wgpu has a bounded timeout; evidence uploads use `if: always()` and 90-day retention.
- **Files changed:** `.github/workflows/ci.yml`, `.github/workflows/phase0-benchmarks.yml`.
- **Tests added:** workflow commands exercise frozen install, schemas, lineage, native smoke, verifier, and capability classification.
- **Evidence:** current master CI [31870422891](https://github.com/Kkb113/CineKernel/actions/runs/31870422891), jobs Windows `94978096773`, Ubuntu `94978096796`, macOS `94978096803`, all success; remote full/all and isolation workflows are also registered and executed.
- **Final status:** RESOLVED.
- **Remaining limitation:** hosted CI classifies unavailable hardware GPU capability instead of treating software fallback as hardware evidence.

## Finding 2 — Timing was not apples-to-apples

- **Original problem:** HyperFrames lint/check/render was compared against render-only paths; encoder differences were hidden.
- **Root cause:** v1 had one coarse elapsed field and no equivalence eligibility contract.
- **Implementation change:** result v2 separates preflight, prepare, startup, production, encode, render command, verify, and end-to-end; encoder settings/limitations are explicit; charts filter to `equivalent` rows.
- **Files changed:** `schemas/phase0/benchmark-result-v2.schema.json`, `benchmarks/phase0/workload-intent.json`, `crates/xtask/src/main.rs`, browser runners, report generator.
- **Tests added:** v2 serialization/schema, equivalence declarations, exact aggregation, and probe scoping.
- **Evidence:** 23 canonical summaries / 109 measured results; HyperFrames preflight is separately charted.
- **Final status:** RESOLVED.
- **Remaining limitation:** framework-internal production/encode splits are `null` where upstream CLIs do not expose them; this is disclosed, not fabricated.

## Finding 3 — Native mixed workload was not equivalent

- **Original problem:** native mixed was a full-duration cube and native typography used rectangular placeholders.
- **Root cause:** feasibility renderers had been mislabeled as equivalent.
- **Implementation change:** deterministic bitmap glyph rasterization; exact title/chart/textured-3D/CTA scene proportions; overlay, transition, and three audio cues.
- **Files changed:** `crates/phase0-native-font/`, `crates/phase0-native-2d/`, `crates/phase0-native-wgpu/`, workload intent.
- **Tests added:** glyph shape/measurement, mixed timing/content tests, decoded semantic checks.
- **Evidence:** mixed central verifier PASS, Probe D PASS, `artifacts/MIXED_EQUIVALENCE_CONTACT_SHEET.png`.
- **Final status:** RESOLVED at semantic equivalence level.
- **Remaining limitation:** bitmap font is benchmark infrastructure, not production shaping/i18n.

## Finding 4 — Main verification was weak

- **Original problem:** output existence, track count, approximate duration/frame count could accept wrong content.
- **Root cause:** verification lived in shallow harness checks rather than a case-aware permanent component.
- **Implementation change:** central Rust verifier validates mux/codec/pixel format/dimensions/fps/timebase/timestamps, decoded frames/hashes/statistics, complete media oracle, decoded audio semantics, and per-case checkpoints.
- **Files changed:** `crates/phase0-verifier/`, `crates/xtask/src/main.rs`, verification schemas.
- **Tests added:** wrong metadata, corrupt output, missing audio, duplicate/gapped timestamps, frequency detection, authored-hold bounds, and FFmpeg compatibility.
- **Evidence:** 109/109 measured outputs PASS; 109-entry `VERIFICATION_MANIFEST_INDEX.json`; post-probe verification remains 109/109.
- **Final status:** RESOLVED.
- **Remaining limitation:** the Phase 1 corpus must broaden codecs, color spaces, channels, and long-form cases.

## Finding 5 — Correctness probes were shallow

- **Original problem:** limited repeatability, sampled media frames, synthetic audio/backpressure, static-only network evidence, and weak cancellation.
- **Root cause:** probes asserted orchestration counters instead of decoded/OS/process behavior.
- **Implementation change:** A-J now use measured-only repeated framemd5, full decoded media mapping, all browser worker modes, shuffled native evaluation, preview/final checkpoints, real three-clip audio and invalid overlap, central mux verification, xtask timeout recovery, and real FFmpeg backpressure.
- **Files changed:** `packages/phase0-common/scripts/run-probes.ts`, fixtures, probe schemas/reports, harness.
- **Tests added:** async entrypoint, warm-up exclusion, scoped GPU tolerance, invalid compositions/audio, and queue bounds.
- **Evidence:** macOS retained-evidence attestation [31870436549](https://github.com/Kkb113/CineKernel/actions/runs/31870436549) has nine PASS, zero FAIL, zero UNSUPPORTED; Ubuntu Probe G [31855975438](https://github.com/Kkb113/CineKernel/actions/runs/31855975438) passes for both browser engines; Probe J maximum three frames / 24,883,200 bytes.
- **Final status:** RESOLVED.
- **Remaining limitation:** retain Linux namespace isolation as a dedicated gate because Windows and macOS cannot provide equivalent `unshare --net` evidence.

## Finding 6 — Harness reliability was incomplete

- **Original problem:** aggregate execution could idle indefinitely and required manual engine separation.
- **Root cause:** blocking `Command::output()` lacked supervision, tree cancellation, stall distinction, warm-up enforcement, and exact matrix validation.
- **Implementation change:** explicit spawned children, logs/PIDs/env, 15-second heartbeats, wall/stall timeouts, Windows task-tree and Unix group cleanup, RSS/temp sampling, structured JSON parsing, failed warm-up/group propagation, exact canonical pointer rules.
- **Files changed:** `crates/xtask/src/process.rs`, `crates/xtask/src/main.rs`.
- **Tests added:** timeout, stall, last structured JSON, tree termination, warm-up validity, exact 109-result inventory.
- **Evidence:** one full command completed 109/109 in 2,929.2 seconds; Probe I killed the tree and the next run succeeded.
- **Final status:** RESOLVED.
- **Remaining limitation:** CI adapter availability is capability-classified rather than falsely passed.

## Finding 7 — Evidence was dirty/unborn

- **Original problem:** prior results recorded `UNBORN` and dirty state.
- **Root cause:** evidence generation preceded a frozen implementation revision and canonical selector.
- **Implementation change:** canonical runs reject dirty/unborn/malformed revisions, bind SHA/spec/lock/environment, validate one exact inventory, and isolate historical results.
- **Files changed:** canonical harness/report code, v2 schema, `CANONICAL_RUN_MANIFEST.json`, `HISTORICAL_RESULTS_SUMMARY.md`.
- **Tests added:** revision enforcement, pointer selection, mixed-revision/environment rejection, historical separation, complete inventory.
- **Evidence:** clean detached worktree at revision A; manifest has `worktree_clean: true`, one environment, 109/109 results.
- **Final status:** RESOLVED.
- **Remaining limitation:** report-only commits are deliberately separate from measured revision A.

## Finding 8 — Source archaeology was shallow

- **Original problem:** maps lacked concrete call paths, owners, concurrency, failures, retries, caches, media/audio, distribution, and 3D synchronization.
- **Root cause:** documents summarized packages without a consistent evidence contract.
- **Implementation change:** required critical paths now record full pinned source links, important functions/types, call sequence, owners, behavior, tests/issues, disposition, and confidence.
- **Files changed:** `docs/research/remotion/`, `docs/research/hyperframes/`, `docs/source-lineage/`, lineage validator.
- **Tests added:** full-SHA GitHub link, local-reference, inventory completeness, source/package relationship, and license validation.
- **Evidence:** validation PASS for 31 documents and four inventory entries.
- **Final status:** RESOLVED.
- **Remaining limitation:** upgrade archaeology must be repeated when pins change.

## Finding 9 — Tests were small or tautological

- **Original problem:** literal-equals-literal assertions did not protect real repository state.
- **Root cause:** tests did not read manifests, locks, sources, or failure paths.
- **Implementation change:** tests now parse actual package manifests/locks/schemas/fixtures/sources and exercise harness/verifier/canonical failure contracts.
- **Files changed:** Rust crate tests and `packages/*/test/*.test.ts`.
- **Tests added:** 26 meaningful Rust tests and 20 TypeScript tests, plus normal CI integration smoke.
- **Evidence:** fmt/check/strict Clippy PASS; Rust 26/26; typecheck PASS; TS 20/20.
- **Final status:** RESOLVED.
- **Remaining limitation:** test count is not treated as coverage; Phase 1 needs broader corpus/integration coverage.

## Finding 10 — Apache license was incomplete

- **Original problem:** root license text was truncated and upstream status could be misunderstood.
- **Root cause:** application and third-party licensing records were not cleanly separated.
- **Implementation change:** full Apache License 2.0 at root, CineKernel notice, third-party notices, upstream license paths and SHA-256 values.
- **Files changed:** `LICENSE`, `NOTICE`, `THIRD_PARTY_NOTICES.md`, upstream lock, lineage inventory.
- **Tests added:** lineage/inventory license completeness validation.
- **Evidence:** root license review and lineage validator PASS; Remotion license hash `11d93557...`, HyperFrames `4259155f...`.
- **Final status:** RESOLVED.
- **Remaining limitation:** external upstreams remain wrapped dependencies and are not claimed to be relicensed.

Overall disposition is **PASS**. The capability-aware remote full/all matrix ran, Ubuntu Probe G passed under OS-enforced loopback-only isolation, and the corrected macOS retained-evidence attestation passed A-F and H-J with zero failures or unsupported probes. See `REMOTE_CLOSURE_ATTESTATION.md`.
