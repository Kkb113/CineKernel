# CineKernel Phase 0 reviewer packet

Phase status: **PASS**. Acceptance authority: `reports/phase0/PHASE0_ACCEPTANCE_REPORT.md`.

## Five architectural conclusions

1. CineKernel must own semantic state and rational-time evaluation; neither React nor HTML is the authoritative IR.
2. Remotion and HyperFrames are useful wrapped compatibility backends with different strengths: ecosystem maturity versus explicit seek/validation instrumentation.
3. Native wgpu is the leading accelerated 2D/3D candidate, paired with a deterministic software 2D reference.
4. Media, audio, muxing, concurrency, and cache behavior require CineKernel-owned contracts and mechanical oracles.
5. 3D should be first-class; Blender remains optional and external rather than part of the core runtime.

## Five benchmark findings

1. Full mixed median: native wgpu 10.17 s, Remotion 23.94 s, HyperFrames 38.93 s.
2. Full 3D median: native wgpu 5.83 s, HyperFrames 27.80 s, Remotion 42.09 s.
3. Full typography median: native 2D 4.36 s, Remotion 19.19 s, HyperFrames 25.41 s.
4. Full HyperFrames micro-cases were relatively stable after warmup (for example chart stddev 0.41 s), but strict validation contributes material fixed overhead.
5. Smoke/cold-start samples are noisy; repeated full-profile medians, not smoke totals, should guide architecture decisions.

Every finding is supported by `reports/phase0/BASELINE_RESULTS.md` and `reports/phase0/BASELINE_RESULTS.json`. The implementations are not visual-quality-equivalent, so these are directional architecture findings.

## Five largest risks

1. Native scope expansion before text/media contracts stabilize.
2. Production text shaping, internationalization, and golden-corpus selection.
3. Cross-driver/cross-OS GPU conformance beyond the single Intel Arc/Vulkan result.
4. Certified render-time network isolation and dependency freezing.
5. FFmpeg distribution, codec capability, and license variation.

Full register: `reports/phase0/RISK_REGISTER.md`.

## Key document paths

- Acceptance: `reports/phase0/PHASE0_ACCEPTANCE_REPORT.md`
- Baselines: `reports/phase0/BASELINE_RESULTS.md`, `reports/phase0/BASELINE_RESULTS.json`
- Correctness: `reports/phase0/CORRECTNESS_PROBES.md`, `reports/phase0/CORRECTNESS_PROBES.json`
- Bakeoff: `reports/phase0/ARCHITECTURE_BAKEOFF.md`
- Archaeology: `reports/phase0/SOURCE_ARCHAEOLOGY_SUMMARY.md`, `docs/research/remotion/`, `docs/research/hyperframes/`
- Reuse: `reports/phase0/REUSE_RECOMMENDATIONS.md`, `docs/research/comparison/CINEKERNEL_REUSE_MATRIX.md`
- Decisions/risks: `reports/phase0/OPEN_DECISIONS.md`, `reports/phase0/RISK_REGISTER.md`, `docs/decisions/`
- Lineage: `docs/source-lineage/`, `benchmarks/upstreams.lock.json`
- Renderer roles: `docs/architecture/renderer-roles.md`

## Reviewer commands

```text
cargo xtask doctor --json
cargo xtask upstream verify --json
pnpm install --frozen-lockfile
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
pnpm typecheck
pnpm test
cargo xtask phase0 prepare --json
cargo xtask phase0 run --profile smoke --json
cargo xtask phase0 verify --json
pnpm --filter @cinekernel/phase0-common probes
cargo xtask phase0 report --json
```

Run the full profile engine-by-engine for isolation:

```text
cargo xtask phase0 run --profile full --engine remotion --json
cargo xtask phase0 run --profile full --engine hyperframes --json
cargo xtask phase0 run --profile full --engine native-2d --json
cargo xtask phase0 run --profile full --engine native-wgpu --json
```

## Visual and output evidence

- Contact sheet: `docs/assets/phase0/contact-sheet.png`
- Deterministic benchmark chart: `docs/assets/phase0/benchmark-chart.svg`
- Renderer diagram: `docs/architecture/renderer-roles.md`
- Comparison diagram: `docs/research/comparison/REMOTION_HYPERFRAMES_COMPARISON.md`
- All videos/logs/results: `.cinekernel/runs/`
- Representative Remotion mixed video: `.cinekernel/runs/20260814T040942Z-ea33f321-9c62-4b3a-9c4f-72f066f771b7/remotion/mixed-2d-3d/rep-3/output.mp4`
- Representative HyperFrames mixed video: `.cinekernel/runs/20260814T042952Z-04f5e721-fd2c-41d5-8284-21690c051e39/hyperframes/mixed-2d-3d/rep-3/output.mp4`
- Representative native wgpu mixed video: `.cinekernel/runs/20260814T045624Z-3c89234d-70d4-4638-8b76-d11719a534ce/native-wgpu/mixed-2d-3d/rep-3/output.mp4`

## Commit list

- `a72cdd64e5813b17bcb94640379238d8ebc690d0` — `feat: establish CineKernel Phase 0 baselines`
- `97d176318ad297cfaeb914b288e1fb46690c4356` — `docs: finalize Phase 0 acceptance packet`

The metadata-only commit that records these hashes cannot include its own content-addressed hash; use `git log --oneline --decorate -3` to inspect it.

## Changed-file summary

The foundation commit adds 119 files: Rust workspace and xtask, pnpm workspace and two browser baselines, two native experiments, schemas/specs/probes, upstream lineage, CI, 39 research/architecture/decision documents, machine-readable reports, and small visual evidence. The follow-up adds the two mandated review documents, benchmark SVG, Mermaid diagrams, final risk/bakeoff corrections, and portable-path test.

## Proposed decisions needing approval

1. Advance native wgpu plus software reference as the Phase 1 renderer architecture, without accepting it as final until cross-platform conformance.
2. Keep Remotion and HyperFrames behind compatibility adapters.
3. Select a production native text shaping/raster stack.
4. Define media-parser adoption boundaries and certified FFmpeg policy.
5. Approve the longer representative workload and its quality/correctness thresholds.

## Questions Phase 1 must resolve

- Which text stack meets shaping, fallback, variable-font, and i18n requirements?
- Which GPU backends/adapters define the certification matrix and tolerances?
- How is network denial enforced at render time across supported operating systems?
- Which decoded-media operations are adopted, wrapped, or reimplemented?
- What is the canonical long workload, and which metrics exclude cold-start noise?

## Explicit non-goals and omissions

Phase 0 did not implement the final VideoIR, production editor/player, distributed scheduler, production text engine, broad codec matrix, certified network sandbox, Blender integration, multi-adapter GPU conformance, or five-to-ten-minute production workload. The native renderers are bounded experiments; the browser integrations are compatibility baselines, not authoritative state.
