# R0.02 reviewer packet

## Status

**CONDITIONAL PASS — NOT LOCKED.** The focused closure requires exact-head three-OS evidence and explicit independent approval before merge.

## Locked identity

- CineKernel base SHA: `974d93ef224b75383499cdb2b70cc086a0dd6f40`
- CineKernel base tree: `80ebf050ebc298b7647a403159ab59f94811468f`
- ONDA repository: `https://github.com/onda-engine/onda-engine.git`
- ONDA pin: `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`
- ONDA tree: `639df83ebf0262afccd6d021bf6d16ef19777d85`
- Closure branch: `research/r0.02-independent-review-closure`
- PR #13 merged branch head: `6e7ff3d6016829357bb7f804dd916e6f7e796a64`
- Master merge commit: `12024231b8983b07d9413cf96f4579bd9495f946`
- Common merged research tree: `e01b2fe87d409e34f509847cdd66214d174eb0d6`
- Process note: PR #13 was merged before independent approval; it is not reverted.

## Locked future-phase registry

- R0.03 — Native GPU, CPU, WASM, and encoding architecture
- R0.04 — Typography, layout, effects, color, and 3D architecture
- R0.05 — Agent component catalog and cinematic composition model
- R0.06 — CLI, installation, preview, embedding, and developer experience
- R0.07 — Independent benchmark and failure analysis
- R0.08 — Adoption, rejection, clean-room, and roadmap-delta matrix

## Counts

| Evidence | Count |
|---|---:|
| Pinned ONDA source records | 74 |
| Unique pinned ONDA files | 50 |
| External official references | 3 |
| Claims | 12 |
| Authoring surfaces | 5 |
| Graph nodes | 15 |
| Graph edges | 18 |
| State owners | 12 |
| Time conversions | 12 |
| Identity transitions | 10 |
| Semantic-preservation rows | 31 |
| Fallback/error rows | 22 |
| Preview/export comparisons | 7 |
| Candidate requirements | 8 |
| Contradictions | 3 |
| Open questions | 6 |
| Deferred topics | 6 |
| Generated machine projections | 16 |
| Generated human reports | 20 |
| Strict schemas | 17 |
| Standalone verifier tests | 82 |
| Integrity-manifest entries | 68 |

## Gate results

- R0.01 authoritative lock parsing: PASS
- checkout remote/detached HEAD/pin/tree/clean validation: PASS
- coverage-only versus claim-supporting source roles, immutable blob/SHA, and symbol-inside-line-range checks: PASS
- strict nested Draft 2020-12 schemas: PASS
- exact-file and normalized multiline clean-room guard: PASS
- authoritative ONDA package identity, renamed Cargo/npm alias, workspace/dev/build/target dependency, Git source, ONDA-checkout path, and resolved lockfile guard: PASS
- absolute path and tracked-upstream guard: PASS
- Phase 0 and R0.01 frozen paths: PASS
- two-run byte equality: run during final reproduction
- historical final-head dedicated run 31927730892 and artifacts: PASS; new exact closure-head run: PENDING
- historical final-head ordinary CI run 31927730849: PASS; new exact closure-head CI: PENDING

## Known check behavior

The standalone frozen R0.01 workflow scans future `schemas/research/**` paths against its frozen manifest. R0.02 therefore runs the unchanged verifier in an exact-base worktree on every OS. No R0.01 file is modified.

## Review paths

- `docs/research/onda/r0.02/R0_02_ACCEPTANCE_REPORT.md`
- `docs/research/onda/r0.02/R0_02_RESEARCH_MODEL.json`
- `docs/research/onda/r0.02/SOURCE_INDEX.json`
- `docs/research/onda/r0.02/ARCHITECTURE_GRAPH.json`
- `reports/research/r0.02/INTEGRITY_MANIFEST.sha256`
- `reports/research/r0.02/REMOTE_REPRODUCTION_ATTESTATION.json`

## Reproduction commands

```text
cargo xtask research onda sync --json
cargo xtask research onda verify --json
cargo xtask research onda integrity --check --json
cargo fmt --manifest-path tools/research/onda-r0-02/Cargo.toml --all --check
cargo clippy --locked --manifest-path tools/research/onda-r0-02/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path tools/research/onda-r0-02/Cargo.toml
cargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- inventory --json
cargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- verify --json
cargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- report --json
cargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- guard --json
cargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- integrity --check --json
```

## Historical final-head evidence and closure requirement

Final PR head `6e7ff3d6016829357bb7f804dd916e6f7e796a64` passed dedicated run `31927730892` and ordinary CI `31927730849` on all three operating systems. The prior attestation was stale because executable `tools/research/onda-r0-02/src/reports.rs` changed after its recorded evidence commit. This closure changes executable verifier/model code, so none of that historical evidence substitutes for fresh exact-closure-head evidence or independent approval.
