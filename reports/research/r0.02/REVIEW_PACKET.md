# R0.02 reviewer packet

## Status

**PASS — exact remediation-head research workflow and ordinary CI succeeded on all three operating systems.** PR #13 must remain draft and unmerged.

## Locked identity

- CineKernel base SHA: `974d93ef224b75383499cdb2b70cc086a0dd6f40`
- CineKernel base tree: `80ebf050ebc298b7647a403159ab59f94811468f`
- ONDA repository: `https://github.com/onda-engine/onda-engine.git`
- ONDA pin: `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`
- ONDA tree: `639df83ebf0262afccd6d021bf6d16ef19777d85`
- Branch: `research/r0.02-onda-scene-compiler-archaeology`

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
| Standalone verifier tests | 73 |
| Integrity-manifest entries | 68 |

## Gate results

- R0.01 authoritative lock parsing: PASS
- checkout remote/detached HEAD/pin/tree/clean validation: PASS
- complete mandatory source coverage and blob/SHA/symbol/line checks: PASS
- strict nested Draft 2020-12 schemas: PASS
- exact-file and normalized multiline clean-room guard: PASS
- authoritative ONDA package identity, renamed Cargo/npm alias, workspace/dev/build/target dependency, Git source, ONDA-checkout path, and resolved lockfile guard: PASS
- absolute path and tracked-upstream guard: PASS
- Phase 0 and R0.01 frozen paths: PASS
- two-run byte equality: PASS
- remote workflow and artifacts: PASS — run 31927165059; three nonempty artifacts
- standard three-OS CI: PASS — run 31927168409

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

## Implementation and evidence commits

- `8d9d425024761715bbbb37f8a14104d1c1fd670b` — initial research packet
- `e9a4546db0962cd30858ad71041ec92c33b81fa7` — workflow token correction
- `c8d16e3d7d8029a3e2fe2e2e2019f48996533758` — immutable blob hashing correction
- `81ba1835d759e332f2d73683161de28a1f0954fc` — historical attestation
- `b528d651fa4e1f5678b098f39fc8c35ce034e1ef` — reviewer remediation evidence commit
- `ca6b4dc85d66ef1d5d9c456dd3173776a0c448b8` — semantic/governance closure evidence commit

## Final remote evidence

- Dedicated R0.02 run: `31927165059` — Windows, Ubuntu, macOS success
- Ordinary CI run: `31927168409` — Windows, Ubuntu, macOS success
- `r0-02-windows-latest-evidence`: `sha256:58d59a5d8fcc2e043d01c646ff22303188fe3c780ffcfb45c4107a368169e0d9`
- `r0-02-macos-latest-evidence`: `sha256:e2f181a75a839f034b84af8b3529f34d2f1c1b3157006ee8f565fee64101d73b`
- `r0-02-ubuntu-latest-evidence`: `sha256:6b350cf3da3292c73b84483a896d64ac0a6e3e9079f74a40fcda23b9e1fe6c1f`
