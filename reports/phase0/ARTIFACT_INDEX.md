# Phase 0.1 artifact index

Tracked review artifacts are small, durable summaries. Raw videos, logs, environment captures, project copies, manifests, and per-repetition results live under ignored `.cinekernel/` locally and in 90-day GitHub Actions artifacts remotely.

| Artifact | Purpose | Provenance state |
|---|---|---|
| `CANONICAL_RUN_MANIFEST.json` | clean revision/spec/lock/environment and exact matrix | pending canonical run |
| `CANONICAL_BASELINE_RESULTS.json/.md` | canonical v2 aggregation and timing summaries | pending canonical run |
| `CORRECTNESS_PROBES.json/.md` | Probes A–J with raw evidence references | pending canonical probes |
| `REVIEW_FINDINGS_RESOLUTION.md` | findings-to-fix trace | implementation current |
| `EQUIVALENCE_REPORT.md` | support and comparison eligibility | implementation current |
| `VERIFIER_REPORT.md` | permanent artifact checks and outcomes | representative smoke current |
| `CI_EVIDENCE.md` | actual remote run IDs/URLs/conclusions | pending remote gates |
| `SOURCE_ARCHAEOLOGY_SUMMARY.md` | critical-path conclusions and immutable source links | update after validation |
| `docs/assets/phase0/contact-sheet.png` | selected canonical visual checkpoints | pending regeneration |
| `docs/assets/phase0/benchmark-chart.svg` | equivalent canonical timing chart | pending regeneration |

No `node_modules`, Rust `target`, `.cinekernel`, or `Remotion-Hyperframe-SourceCode` directory is tracked or uploaded as source content.
