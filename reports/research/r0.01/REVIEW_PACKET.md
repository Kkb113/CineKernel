# R0.01 reviewer packet

- Status: CONDITIONAL PASS — remote reproduction pending
- CineKernel base: `5f47f341aa546b4ceb115fcad71d576d0ab85f29`
- Research branch: `research/r0.01-onda-provenance`
- Harness/schemas/tests commit: `d35e31615ab7d9ef6e348ccde1a4b243dc364bc8`
- Evidence commit: the commit containing this packet
- ONDA repository: `https://github.com/onda-engine/onda-engine.git`
- ONDA pin: `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`
- ONDA tree: `639df83ebf0262afccd6d021bf6d16ef19777d85`
- LICENSE SHA-256: `7e6fdc32986a1ea86933be194a15266419c74963187d9ebf02e2d116a473af29`
- LICENSE-APACHE SHA-256: `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`
- NOTICE SHA-256: `cba16f2312c5866182513c58d46822a34beec24b32b6521511d24a64d78db7d2`
- Cargo.lock SHA-256: `67fa301327a87135b37c11a7dc759a99c2ae9bfbf8ddecb2fdaddf7e6553d258`
- pnpm-lock.yaml SHA-256: `aae73d8537740491fab9cfc75c8966f6f1af97a98fc3c3e1d39663217eed278f`
- Rust workspace members: 19
- Resolved Rust packages: 416
- pnpm workspace packages: 13
- Resolved pnpm packages: 528
- External models/artifacts: 19 total records
- Release streams: 3
- License hotspots: 9
- Guards: dependency PASS; tracked-source PASS; exact-copy PASS; Phase 0 immutability PASS; absolute-path leakage PASS

## Reproduction

```text
cargo xtask research onda sync
cargo xtask research onda verify --json
cargo xtask research onda inventory --json
cargo xtask research onda report --json
cargo xtask research onda guard --json
cargo xtask research onda integrity --check --json
```

The dedicated workflow runs the generation pipeline twice, requires a byte-clean Git diff, and uploads raw evidence without building or executing ONDA. Raw evidence is written only below ignored `.cinekernel/research/onda/r0.01/`.
