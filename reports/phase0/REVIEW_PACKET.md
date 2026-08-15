# CineKernel Phase 0.1 reviewer packet

Status: **PASS**. Original aggregate revision A: `0249b40ec41673ed8ac2f22c23583ddc3629a320`. Native-floor closure: `907a2551c3dad27c698ac43d7ecb41957236be53`. Remote performance revision: `6f254eda880ab5a1463baac1d0a1819b7c68cac7`. Final closure master: `1c07e19fb0eb9b9f9c4b7c5e3cc26b6a29e54a93`.

Current master CI: [run 31870422891](https://github.com/Kkb113/CineKernel/actions/runs/31870422891), all three OS jobs green. Remote full/all: [run 31855973437](https://github.com/Kkb113/CineKernel/actions/runs/31855973437). Ubuntu Probe G: [run 31855975438](https://github.com/Kkb113/CineKernel/actions/runs/31855975438). Final macOS probes: [run 31870436549](https://github.com/Kkb113/CineKernel/actions/runs/31870436549), 9/9 PASS.

## Five largest changes

1. Replaced blocking child execution with supervised process trees, heartbeat/stall/wall timeouts, recovery, resource sampling, and canonical selection.
2. Added a permanent decoded artifact verifier and 109 SHA-bound measured sidecars.
3. Made native typography and mixed 2D/3D semantically equivalent, including glyphs, four scenes, textured 3D, transitions, overlay, CTA, and audio.
4. Split preflight/render/verification timing and introduced strict v2 schemas, clean-revision provenance, exact matrices, and historical isolation.
5. Deepened pinned upstream archaeology/provenance and repaired three-OS CI, licenses, fixtures, tests, and manual evidence workflows.

## Five important benchmark findings

1. One aggregate full command completed 109/109 measured attempts and 23 warm-ups without manual engine switching.
2. On this Intel Arc/Vulkan host, mixed median render-command time was native wgpu 10,624.9 ms (`n=3`), HyperFrames 24,132.6 ms (`n=3`), Remotion 35,470.6 ms (`n=3`).
3. Native 3D median was 5,123.0 ms (`n=5`), HyperFrames 15,357.4 ms (`n=5`), Remotion 20,082.3 ms (`n=5`); this is candidate evidence, not cross-platform certification.
4. HyperFrames preflight is material (roughly 9.9-12.7 s median by workload) and is now excluded from direct render-command comparisons but retained separately.
5. Repeated decoded output is exact for all scoped rows except documented Remotion mixed WebGL variance, which passes bounded PSNR/SSIM thresholds.

## Five important limitations

1. Local hardware GPU performance evidence covers one Intel Arc/Vulkan adapter; hosted-runner software/capability evidence is not cross-driver certification.
2. The deterministic bitmap glyph path is benchmark infrastructure, not a production international text stack.
3. Phase 0 micro/mixed workloads do not replace a five-to-ten-minute production corpus or final VideoIR evaluation.
4. FFmpeg distribution/version differences across operating systems still require a certified production boundary.
5. Phase 0 selects candidate renderer roles, not an irreversible final renderer architecture.

## Review artifacts and order

1. `reports/phase0/PHASE0_ACCEPTANCE_REPORT.md` — authoritative status and all 25 required sections.
2. `reports/phase0/REVIEW_FINDINGS_RESOLUTION.md` — ten reviewer findings and traceability.
3. `reports/phase0/CANONICAL_BASELINE_RESULTS.md` / `.json` — 23 summaries and 109 raw measured results.
4. `reports/phase0/EQUIVALENCE_REPORT.md` and `reports/phase0/artifacts/EQUIVALENT_RENDER_MEDIANS.svg` — eligibility and selected timing view.
5. `reports/phase0/VERIFIER_REPORT.md`, `VERIFICATION_MANIFEST_INDEX.json`, and `CORRECTNESS_PROBES.md` — verification and adversarial evidence.
6. `reports/phase0/ARCHITECTURE_BAKEOFF.md` — proposed architecture direction.
7. `reports/phase0/SOURCE_ARCHAEOLOGY_SUMMARY.md` and `docs/research/{remotion,hyperframes}/` — pinned source analysis.
8. `reports/phase0/REMOTE_CLOSURE_ATTESTATION.md`, `RISK_REGISTER.md`, `OPEN_DECISIONS.md`, and `CI_EVIDENCE.md` — remote evidence, limitations, decisions, and closure state.
9. `reports/phase0/ARTIFACT_INDEX.md` and `ARTIFACT_SHA256.txt` — inventory and integrity.

Tracked visual artifacts are `MIXED_EQUIVALENCE_CONTACT_SHEET.png`, `MEDIA_ORACLE_CONTACT_SHEET.png`, `EQUIVALENT_RENDER_MEDIANS.svg`, and `HYPERFRAMES_PHASE_MEDIANS.svg`. Runtime videos/logs/manifests are ignored locally and configured for 90-day workflow retention.

## Exact reviewer commands

```text
corepack pnpm install --frozen-lockfile
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
corepack pnpm typecheck
corepack pnpm test
corepack pnpm --filter @cinekernel/phase0-common lineage:validate
cargo xtask doctor --json
cargo xtask environment capture --json
cargo xtask upstream sync
cargo xtask upstream verify --json
cargo xtask phase0 prepare --json
cargo xtask phase0 canonical-run --profile smoke --json
cargo xtask phase0 verify --canonical --json
cargo xtask phase0 canonical-run --profile full --json
cargo xtask phase0 probes --canonical --json
cargo xtask phase0 verify --canonical --json
cargo xtask phase0 report --canonical --json
```

Canonical commands reject dirty or unborn revisions. `node_modules`, `target`, `.cinekernel`, and `Remotion-Hyperframe-SourceCode` are excluded from Git.
