# CineKernel Phase 0.1 acceptance report

## Executive status

**CONDITIONAL — NOT YET ACCEPTED.** The review-remediation implementation and representative local artifacts pass, but acceptance requires a clean committed implementation revision, canonical smoke/full/probes from a detached clean worktree, a separate evidence commit, and actual green GitHub Actions records on Windows, Ubuntu, and macOS.

## Proven facts at implementation stage

- Upstreams are pinned and verified: Remotion `4e459b8b3aeec12ac8346666773ea28892a30e31` / package `4.0.509`; HyperFrames `532caf7aa24fef382cb103013f6414bb547a4129` / package `0.7.108`.
- Package integrity, source-tree SHA, release relationship, license path/SHA, and sparse paths are recorded.
- Rust formatting/check/tests and strict Clippy pass locally: 24 Rust tests pass. TypeScript typecheck and all 18 TypeScript tests pass.
- Source-lineage validation passes for 31 documents and four inventory entries.
- Representative centrally verified smoke artifacts pass for native 2D typography, both browser media-oracle paths, both browser three-clip audio paths, and native wgpu mixed 2D/3D on Intel Arc/Vulkan without software fallback.
- The complete non-canonical smoke matrix passes 19/19 engine/workload groups with zero verifier failures; canonical evidence remains pending until revision A is committed.
- The root contains the full Apache License 2.0 text. Remotion upstream licensing is not represented as Apache.

## Pending mandatory evidence

1. Implementation revision A and clean detached benchmark worktree.
2. Canonical smoke and exact 109-result canonical full matrix.
3. Probes A–J and final canonical verification/report generation.
4. Small regenerated contact sheet and equivalent-workload chart.
5. Evidence revision B.
6. Green normal CI on Windows/Linux/macOS.
7. Green manual full/all benchmark workflow, including Ubuntu loopback-only network isolation.

## Architecture recommendation (provisional)

Continue to evaluate native wgpu as the accelerated renderer and native software 2D as the deterministic reference. Keep Remotion and HyperFrames as wrapped compatibility backends. Do not advance to Phase 1 until all pending mandatory evidence is complete and this status is explicitly changed to PASS.
