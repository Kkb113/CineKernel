# R0.01 research source index

All ONDA links below are immutable at `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`.

| Source ID | Repository/source | Pin/version | Path/section | Fact supported | Classification |
|---|---|---|---|---|---|
| ONDA-001 | [onda-engine/onda-engine](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/Cargo.toml) | 3ddf1780c9799bf038ac90cec7d8cadb61acafbe | Cargo.toml, workspace/dependencies sections | Rust workspace/version/members/features | UPSTREAM_SOURCE |
| ONDA-002 | [root package](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/package.json) / [pnpm workspace](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/pnpm-workspace.yaml) | same pin | package metadata/workspace package list | JS toolchain and workspace topology | UPSTREAM_SOURCE |
| ONDA-003 | [LICENSE](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/LICENSE) / [LICENSE-APACHE](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/LICENSE-APACHE) / [NOTICE](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/NOTICE.md) | same pin | full license texts and NOTICE completeness statement | Current/future texts and notice limitations | LICENSE_PRIMARY_SOURCE |
| ONDA-004 | [Cargo.lock](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/Cargo.lock) / [pnpm-lock.yaml](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/pnpm-lock.yaml) | same pin | package/importer records | Exact dependency resolution/integrity | UPSTREAM_SOURCE |
| ONDA-005 | [embed workflow](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/.github/workflows/release.yml) / [npm workflow](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/.github/workflows/release-npm.yml) | same pin | jobs, triggers and publish steps | Release mechanisms | UPSTREAM_WORKFLOW |
| ONDA-006 | [build-embed-kit.sh](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/scripts/build-embed-kit.sh) / [.vendor-entry.mjs](https://github.com/onda-engine/onda-engine/blob/3ddf1780c9799bf038ac90cec7d8cadb61acafbe/.vendor-entry.mjs) | same pin | feature list and vendor entry exports | Intended embed-kit contents | UPSTREAM_SOURCE |
| ONDA-007 | GitHub Releases API | observed 2026-08-15 | release 353301462 / asset 475692944 | v0.2.16 asset size/digest | UPSTREAM_RELEASE_METADATA |
| ONDA-008 | npm registry | onda-engine@0.6.1 | dist/time/license/repository | Publication, SRI, SHA-1, signature, time | REGISTRY_METADATA |
| ONDA-009 | GitHub PR API | PR #41 | body/merge | Prior release automation gap | UPSTREAM_DOC |
| CK-001 | CineKernel | 5f47f341aa546b4ceb115fcad71d576d0ab85f29 | frozen Phase 0 paths | Accepted immutable base | CINEKERNEL_OBSERVATION |

Dependency-specific license expressions and repository metadata are retained per package in `DEPENDENCY_INVENTORY.json`; model and binary claims requiring primary-source follow-up remain `LEGAL_REVIEW_REQUIRED` or `UNRESOLVED`.
