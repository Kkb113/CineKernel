# Phase 0.1 reviewer findings resolution

This report maps every reviewer finding to implementation and required evidence. Status is not promoted to accepted until the clean canonical run and remote gates are recorded.

| Finding | Resolution | Verification gate | Status |
|---|---|---|---|
| 1. GitHub CI failing | pnpm is installed before Node cache lookup; Rust components and commands are valid on a three-OS matrix; evidence uploads use `if: always()` | green Windows, Ubuntu, and macOS run URLs | implemented; remote evidence pending |
| 2. Timing not apples-to-apples | v2 separates preflight, project preparation, engine startup, frame production, encode, render command, artifact verify, and end-to-end; reports compare `render_command` only for equivalent workloads | canonical v2 schema and full aggregate | implemented; canonical evidence pending |
| 3. Native mixed workload not equivalent | native wgpu now renders exact title/chart/textured 3D/CTA proportions, overlay, transition, and three audio cues | mixed semantic/audio verifier and Probe D | representative smoke PASS |
| 4. Main verifier weak | central Rust verifier checks mux, timestamps, decoded frames/hashes/statistics, full media oracle, and decoded audio semantics; verifier failure invalidates the result | verifier unit tests and canonical verification manifests | implemented; representative artifacts PASS |
| 5. Probes shallow | Probes A–J execute repeated framemd5, all worker modes, shuffled native evaluation, preview/final comparisons, invalid audio, OS isolation, mux checks, timeout recovery, and real bounded FFmpeg backpressure | canonical probe JSON plus uploaded raw evidence | implemented; canonical execution pending |
| 6. Harness reliability incomplete | supervised process tree, wall/stall deadlines, heartbeats, RSS/temp sampling, bounded warm-up, last structured JSON, invalid failure records | Rust timeout/stall tests and Probe I | unit/integration PASS |
| 7. Evidence not tied to clean revision | canonical command rejects dirty/unborn state and records implementation/spec/lock hashes in one manifest; historical runs are excluded | clean detached worktree procedure | implemented; evidence commit pending |
| 8. Archaeology shallow | all 17 required critical paths now trace functions/types, ownership, concurrency, failures, cache, preview/final, tests, decision, and confidence with immutable links | lineage validator and summary | implemented; validator PASS |
| 9. Tests too small/tautological | tests read real manifests/locks, validate schemas and fixture hashes, exercise matrix/revision/selection/timeouts, verifier failures, semantics, URL bans, and invalid cases | `cargo test`, `pnpm test`, strict Clippy | local PASS |
| 10. License incomplete | root `LICENSE` contains full Apache License 2.0 terms; upstream licensing remains separately attributed | license review and lineage inventory | implemented |

Final disposition will be updated only after canonical full/probes and remote workflows complete.
