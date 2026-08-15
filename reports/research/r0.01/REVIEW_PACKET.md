# R0.01 reviewer packet

- Status: CONDITIONAL PASS — remote reproduction pending
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
