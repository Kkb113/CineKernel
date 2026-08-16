# R0.02 reviewer packet

## Status

**CONDITIONAL PASS pending final remediation-head remote reproduction.** PR #13 must remain draft and unmerged.

## Locked identity

- CineKernel base SHA: `974d93ef224b75383499cdb2b70cc086a0dd6f40`
- CineKernel base tree: `80ebf050ebc298b7647a403159ab59f94811468f`
- ONDA repository: `https://github.com/onda-engine/onda-engine.git`
- ONDA pin: `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`
- ONDA tree: `639df83ebf0262afccd6d021bf6d16ef19777d85`
- Branch: `research/r0.02-onda-scene-compiler-archaeology`

## Counts

| Evidence | Count |
|---|---:|
| Pinned ONDA files | 50 |
| External official references | 3 |
| Claims | 12 |
| Authoring surfaces | 5 |
| Graph nodes | 15 |
| Graph edges | 18 |
| State owners | 12 |
| Time conversions | 12 |
| Identity transitions | 10 |
| Semantic-preservation rows | 31 |
| Fallback/error rows | 21 |
| Preview/export comparisons | 7 |
| Candidate requirements | 8 |
| Contradictions | 3 |
| Open questions | 6 |
| Deferred topics | 6 |
| Generated machine projections | 16 |
| Generated human reports | 20 |
| Strict schemas | 17 |
| Standalone verifier tests | 63 |
| Integrity-manifest entries | 68 |

## Gate results

- R0.01 authoritative lock parsing: PASS
- checkout remote/detached HEAD/pin/tree/clean validation: PASS
- complete mandatory source coverage and blob/SHA/symbol/line checks: PASS
- strict nested Draft 2020-12 schemas: PASS
- exact-file and normalized multiline clean-room guard: PASS
- dependency alias and Git dependency guard: PASS
- absolute path and tracked-upstream guard: PASS
- Phase 0 and R0.01 frozen paths: PASS
- two-run byte equality: run during final reproduction
- remote workflow and artifacts: pending remediation-head run
- standard three-OS CI: pending remediation-head run

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
