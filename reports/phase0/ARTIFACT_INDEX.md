# Phase 0.1 artifact index

Tracked review artifacts are durable summaries from evidence commit `b12e6c74a519fa693a49f50cd23df6dddc910b88`. Raw videos, command logs, environment captures, per-attempt results, and 109 sidecar verification manifests remain under ignored `.cinekernel/` locally and are configured for 90-day GitHub Actions retention.

| Artifact | Purpose | Provenance |
|---|---|---|
| `CANONICAL_RUN_MANIFEST.json` | clean revision/spec/lock/environment and exact 109-result inventory | canonical run `20260814T144948Z-c6e0a98a-b94a-48e9-a26e-f69faf10f048` |
| `CANONICAL_BASELINE_RESULTS.json/.md` | complete v2 results and equivalent render-command aggregates | 109 successes / 23 groups / zero failures |
| `CORRECTNESS_PROBES.json/.md` | Probes A-J and raw measured evidence | 9 PASS / 0 FAIL / 1 UNSUPPORTED |
| `VERIFICATION_MANIFEST_INDEX.json/.md` | measured sidecar paths, sidecar hashes, output hashes, and pass state | 109 entries / all pass |
| `artifacts/MIXED_EQUIVALENCE_CONTACT_SHEET.png` | four scene checkpoints across three equivalent engines | visually inspected, final canonical rep 1 |
| `artifacts/MEDIA_ORACLE_CONTACT_SHEET.png` | six decoded source checkpoints across both browser engines | visually inspected, final canonical rep 1 |
| `artifacts/EQUIVALENT_RENDER_MEDIANS.svg` | selected equivalent median comparisons | canonical `render_command` timings |
| `artifacts/HYPERFRAMES_PHASE_MEDIANS.svg` | separated HyperFrames preflight/render/verify medians | canonical raw v2 timings |
| `REVIEW_FINDINGS_RESOLUTION.md` | ten findings mapped to fixes and gates | final attestation |
| `CI_EVIDENCE.md` | actual CI records and manual-dispatch blocker | normal CI green; manual blocked |
| `PHASE0_1_CLOSURE_EVIDENCE.md` | focused reruns required by the native 3D floor change | 24/24 verified at closure implementation `907a2551` |
| `SOURCE_ARCHAEOLOGY_SUMMARY.md` | critical-path conclusions and immutable source links | validator PASS |
| `ARTIFACT_SHA256.txt` | review-artifact integrity manifest | 27 entries, regenerated and verified after closure evidence |

No `node_modules`, Rust `target`, `.cinekernel`, or `Remotion-Hyperframe-SourceCode` path is tracked.
