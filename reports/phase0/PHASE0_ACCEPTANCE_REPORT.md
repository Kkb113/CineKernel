# CineKernel Phase 0 acceptance report

## 1. Executive status

**PASS.** All required Phase 0 acceptance gates are satisfied. All ten correctness probes report `PASS`; 38 latest engine/case/profile benchmark groups verify successfully. Non-blocking environmental and historical failures are retained and explained below.

## 2. CineKernel revision

Implementation revision used for final clean-checkout validation: `a72cdd64e5813b17bcb94640379238d8ebc690d0`. Acceptance/reviewer artifact revision: `97d176318ad297cfaeb914b288e1fb46690c4356`. Benchmark records captured before the repository's root commit correctly record `UNBORN` plus `cinekernel_dirty: true`; this provenance is preserved rather than rewritten.

## 3. Branch

`master` (the pre-existing unborn branch name was preserved).

## 4. Upstream commits and package versions

| Upstream | Commit | Package version | Lock evidence |
|---|---|---:|---|
| Remotion | `4e459b8b3aeec12ac8346666773ea28892a30e31` | `4.0.509` | `benchmarks/upstreams.lock.json` |
| HyperFrames | `532caf7aa24fef382cb103013f6414bb547a4129` | `0.7.108` | `benchmarks/upstreams.lock.json` |

Sparse source snapshots and license files are verified under `.cinekernel/upstreams/` and remain untracked. Lineage records are in `docs/source-lineage/`.

## 5. Environment summary

- Windows 11 build 26200, x86_64.
- Intel Core Ultra 7 155H, 16 physical / 22 logical cores, 33,777,467,392 bytes RAM.
- Intel Arc Graphics, driver `32.0.101.8724`; native wgpu selected Vulkan and reported no software fallback.
- Rust `1.97.1`, Node `24.14.0`, pnpm `11.8.0`, Chrome `151.0.7922.138`.
- FFmpeg/ffprobe `N-125119-g4bbb7d9b99-20260619`.
- Final manifest: `.cinekernel/environments/011e0f4ee15002e7b4ff4187a5bc51e22ee0a19115fb5322f25df2180a106ebb.json`.

## 6. Work completed

- Bootstrapped the Rust/pnpm monorepo, lockfiles, policies, schemas, CI, contribution/security files, and `cargo xtask` control plane.
- Implemented deterministic fixtures, structured environment/result records, safe generated cleanup, upstream sparse sync/SHA verification, output verification, aggregation, and reporting.
- Implemented all seven applicable Remotion and HyperFrames cases, including local media/audio/fonts and mixed 2D/3D output.
- Implemented native tiny-skia/resvg 2D and real offscreen wgpu 3D/mixed renderers with exact-time and shuffled-order evaluation.
- Completed pinned source archaeology, ten ADRs, reuse matrix, renderer bakeoff, risk register, open decisions, contact sheet, diagrams, and benchmark chart.
- Implemented and executed Probes A–J mechanically.

## 7. Commands executed and exit codes

| Command | Final exit | Result |
|---|---:|---|
| `cargo fmt --all --check` | 0 | Pass |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 | Pass after two idiomatic initializer/return fixes |
| `cargo test --workspace --all-features` | 0 | Pass |
| `corepack enable` | 1 | Environment limitation: `EPERM` opening `C:\Program Files\nodejs\pnpm.CMD`; existing pnpm remained usable |
| `pnpm install --frozen-lockfile` | 0 | Pass |
| `pnpm typecheck` | 0 | Pass |
| `pnpm test` | 0 | Pass |
| `cargo xtask doctor --json` | 0 | Pass, no required tools missing |
| `cargo xtask environment capture --json` | 0 | Pass |
| `cargo xtask upstream sync` | 0 | Pass, sparse pinned source acquired |
| `cargo xtask upstream verify --json` | 0 | Pass, both SHAs and licenses verified |
| `cargo xtask phase0 prepare --json` | 0 | Pass; repeated after required clean |
| `cargo xtask phase0 run --profile smoke --json` | 0 | Latest 19 applicable groups pass |
| `cargo xtask phase0 run --profile full --json` | terminated | Became idle after all 33 Remotion results; permitted engine-isolated route used and evidence preserved |
| `cargo xtask phase0 run --profile full --engine remotion --json` | 0 equivalent retained sweep | 33 verified measured results retained in the interrupted aggregate run |
| `cargo xtask phase0 run --profile full --engine hyperframes --json` | 3 then 0 for corrected case | Five strict-check failures isolated to 3D DOM-motion observability; corrected and five-repetition rerun passed |
| `cargo xtask phase0 run --profile full --engine native-2d --json` | 0 | 15 verified measured results |
| `cargo xtask phase0 run --profile full --engine native-wgpu --json` | 0 | 8 verified measured results |
| `cargo xtask phase0 verify --json` | 0 | 38 latest groups verified; 14 historical failures retained |
| `pnpm --filter @cinekernel/phase0-common probes` | 0 | Probes A–J pass |
| `cargo xtask phase0 report --json` | 0 | 135 raw / 121 successful / 14 failed attempts aggregated |
| `cargo xtask phase0 clean --scope generated --json` | 0 | Removed only `.cinekernel/generated` |
| Temporary clean-checkout frozen install/check/typecheck/tests | 0 | All build/test commands passed at `a72cdd6` |

The temporary clean-checkout removal returned exit 1 because Windows left dependency/build directories non-empty at `C:\Users\karth\AppData\Local\Temp\cinekernel-clean-a72cdd6`. Git removed the worktree registration; the remaining directory is outside the repository and is an environment cleanup limitation.

## 8. Test results

- Rust: 4 unit tests pass, including cleanup boundary checks and portable paths with spaces/Unicode; all crate/doc test targets pass.
- TypeScript/JavaScript: 7 tests pass across common, Remotion, and HyperFrames packages.
- `pnpm typecheck`: all three TypeScript packages pass.
- rustfmt and clippy with warnings denied pass.
- A detached clean checkout passed frozen install, Rust check/tests, TypeScript check, and JavaScript tests.

## 9. Benchmark cases and configurations

Cases are defined in `benchmarks/specs/phase0-cases.json`: typography/layout, vector/effects, chart/diagram, media sampling, audio/captions, 3D scene, and mixed 2D/3D.

- Smoke: 640×360, 30 fps, duration scale 0.2, one repetition; `benchmarks/profiles/smoke.json`.
- Full: 1920×1080, 30 fps, one warmup, five micro-case repetitions and three mixed-showcase repetitions; `benchmarks/profiles/full.json`.
- H.264/yuv420p output, local-only frozen fixtures, ffprobe duration/frame/track verification.

## 10. Raw benchmark artifact locations

- All retained runs: `.cinekernel/runs/`.
- Remotion full sweep: `.cinekernel/runs/20260814T040942Z-ea33f321-9c62-4b3a-9c4f-72f066f771b7/remotion/`.
- HyperFrames full sweep with retained strict-check failures: `.cinekernel/runs/20260814T042952Z-04f5e721-fd2c-41d5-8284-21690c051e39/hyperframes/`.
- Corrected HyperFrames 3D rerun: `.cinekernel/runs/20260814T045036Z-5e16112c-a0cd-45af-85fc-7cc1dfcd81d0/hyperframes/3d-scene/`.
- Native 2D full sweep: `.cinekernel/runs/20260814T045334Z-95020ef7-10d1-4134-b24a-f5af01141ea9/native-2d/`.
- Native wgpu full sweep: `.cinekernel/runs/20260814T045624Z-3c89234d-70d4-4638-8b76-d11719a534ce/native-wgpu/`.
- Machine-readable aggregate: `reports/phase0/BASELINE_RESULTS.json`.

## 11. Benchmark summary

Timing summaries use only verified successful attempts; the JSON retains all failures. On the full mixed workload, median elapsed times were native wgpu 10.17 s, Remotion 23.94 s, and HyperFrames 38.93 s. On full 3D, medians were native wgpu 5.83 s, HyperFrames 27.80 s, and Remotion 42.09 s. These pipelines are architecture probes, not pixel-quality-equivalent products. Full statistics and sample counts are in `reports/phase0/BASELINE_RESULTS.md` and `reports/phase0/BASELINE_RESULTS.json`.

## 12. Correctness probe results

Probes A–J all pass. Highlights: exact six-mode media oracle matches; identical sequential/shuffled native framemd5 streams; Remotion still/final MAE 0.723 and HyperFrames snapshot/final MAE 1.933; correct tone/silence RMS; seam jumps below 0.08; 121 structurally verified outputs; bounded queue maximum 4; deliberate process-tree termination left zero survivors and the next run succeeded. Evidence: `reports/phase0/CORRECTNESS_PROBES.md` and `reports/phase0/CORRECTNESS_PROBES.json`.

## 13. Renderer bakeoff conclusion

Keep Remotion and HyperFrames as wrapped web compatibility/reference backends. Advance native wgpu as the certified accelerated candidate and tiny-skia/resvg as the slower deterministic software reference. Do not make browser authoring state authoritative. Evidence: `reports/phase0/ARCHITECTURE_BAKEOFF.md`.

## 14. Reuse recommendations

Wrap browser engines; derive exact-time seek and bounded scheduling concepts; adopt media parsing only behind CineKernel contracts; reimplement authoritative state/evaluation/native rendering/transactional operations; reject unverified success and unrestricted render-time dependencies. Evidence: `reports/phase0/REUSE_RECOMMENDATIONS.md` and `docs/research/comparison/CINEKERNEL_REUSE_MATRIX.md`.

## 15. Source-lineage status

PASS. Policy, permission basis, inventory, immutable commits, package versions, license paths, sparse paths, and source hashes are recorded in `docs/source-lineage/POLICY.md`, `docs/source-lineage/permission-basis.md`, `docs/source-lineage/upstream-inventory.yaml`, and `benchmarks/upstreams.lock.json`.

## 16. CI status

PASS for Phase 0 definition scope. `.github/workflows/ci.yml` defines lightweight Windows/Linux/macOS checks; `.github/workflows/phase0-benchmarks.yml` is manually dispatchable for heavy benchmarks. Remote workflows were not run because no push was requested. Local Windows equivalents and a clean detached checkout pass.

## 17. Known limitations

- Native GPU evidence covers one Intel Arc/Vulkan adapter on Windows; cross-driver and cross-OS conformance remain Phase 1 work.
- Native 2D is a feasibility/reference implementation, not production text shaping or internationalization.
- Browser totals include validation/capture startup and are not a visual-quality parity ranking.
- Network isolation Probe G is a static dependency audit; a certified OS-level network sandbox remains required.
- Blender was unavailable and remains an optional external-cinematic candidate.
- The representative workload is 15 seconds, not the future five-to-ten-minute production target.

## 18. Failed or blocked gates

No acceptance gate remains blocked. Non-blocking failures are: `corepack enable` (`EPERM`, environment); one all-engine full orchestration process became idle (engine-isolated execution explicitly allowed and completed); 14 historical implementation attempts remain in raw evidence; and temporary clean-worktree directory removal failed outside the repository. All latest benchmark groups and all correctness probes pass.

## 19. Open reviewer decisions

The five decisions in `reports/phase0/OPEN_DECISIONS.md` remain: approve the proposed wgpu direction after cross-platform evidence; select native text shaping; define the long target workload; choose media parser adoption boundaries; and set FFmpeg distribution/license policy.

## 20. Recommendation for Phase 1

Proceed to Phase 1 with CineKernel-owned rational-time evaluation, verified media/audio contracts, bounded scheduling, native wgpu as the accelerated candidate, and native software 2D as the reference oracle. Keep both browser engines behind compatibility adapters. Treat final renderer selection as proposed—not accepted—until the cross-platform adapter matrix, text stack, network sandbox, and longer workload are proven.
