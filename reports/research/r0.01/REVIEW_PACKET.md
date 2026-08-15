# R0.01 reviewer packet

- Status: PASS
- CineKernel base: `5f47f341aa546b4ceb115fcad71d576d0ab85f29`
- Research branch: `research/r0.01-onda-provenance`
- Harness/schemas/tests commit: `d35e31615ab7d9ef6e348ccde1a4b243dc364bc8`
- Evidence commit: the commit containing this packet
- ONDA repository: `https://github.com/onda-engine/onda-engine.git`
- ONDA pin: `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`
- ONDA tree: `639df83ebf0262afccd6d021bf6d16ef19777d85`
- LICENSE SHA-256: `608b7a8dc76cd64ecd90172e382dd9851adb306b47de874b21f2ff52ab32bcc9`
- LICENSE-APACHE SHA-256: `3ddf9be5c28fe27dad143a5dc76eea25222ad1dd68934a047064e56ed2fa40c5`
- NOTICE SHA-256: `21578be770583e25069042415dc63189ed33dd8605b2ddfd3a205926483e1647`
- Cargo.lock SHA-256: `b455bb801e1b22b86d5258527fe68d24c353491365c493a434a94e8f77f46afb`
- pnpm-lock.yaml SHA-256: `ebfda5d4f51a20eb3f850f04011ddb1952ab33dca5b51ce83e4345d39e6eb639`
- Rust workspace members: 19
- Resolved Rust packages: 416
- pnpm workspace packages: 13
- External models/artifacts: 13 total records (5 model/downloaded-binary records)
- Release streams: 3
- License hotspots: 9
- Unresolved factual items: 2 lock-level items; all additional inconsistencies remain explicit in the register
- Legal-review items: 9 hotspot chains
- R0.01 harness tests: 32 passed, 0 failed
- Full tests: 65 Rust passed, 27 JavaScript passed, 0 final failures
- Determinism: 15 documents compared, 0 SHA-256 differences
- Guards: dependency PASS; tracked-source PASS; exact-copy PASS; Phase 0 immutability PASS

## Verification exit codes

| Command | Exit |
|---|---:|
| `cargo fmt --all --check` | 0 |
| `cargo check --workspace --all-targets --all-features` | 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | 0 |
| `cargo test --workspace --all-features` | 0 |
| `corepack pnpm install --frozen-lockfile` | 0 |
| `corepack pnpm typecheck` | 0 |
| `corepack pnpm test` | 0 |
| each required `cargo xtask research onda ... --json` invocation | 0 |

One restricted-sandbox Rust attempt could not terminate the synthetic timeout-test process and exited 101; the unchanged full suite rerun with normal Windows process-tree control exited 0. This was an execution-environment limitation, not a source change.

## Reproduction

```text
cargo xtask research onda sync
cargo xtask research onda verify --json
cargo xtask research onda inventory --json
cargo xtask research onda report --json
cargo xtask research onda guard --json
```

All commands must exit 0. Raw evidence is written only below ignored `.cinekernel/research/onda/r0.01/`. Committed artifacts are under `docs/research/onda/r0.01/`, schemas under `schemas/research/`, and this packet plus the integrity manifest under `reports/research/r0.01/`.
