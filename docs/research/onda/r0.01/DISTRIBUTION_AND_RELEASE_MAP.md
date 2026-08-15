# ONDA distribution and release map

Three independent streams are verified: GitHub embed-kit `v0.2.16` (release `353301462`, asset `475692944`, SHA-256 `d4335601dc0c66733f772261cbce2de48457767aaa61195691e70062cf331742`), public npm umbrella `onda-engine@0.6.1` (SHA-1 shasum `e48444f629cbc112f23456809e348fa67feafad8`, SHA-512 SRI retained in `RELEASE_MAP.json`), and scoped `@onda-engine/*` GitHub Packages (`AUTH_REQUIRED_NOT_VERIFIED`).

The embed kit intends to combine native binaries, bundled JavaScript, declarations, WASM, fonts/audio tooling and a manifest. Artifact contents were not downloaded: `ARTIFACT_INSPECTION_NOT_AUTHORIZED`. No binaries were executed.

PR #41 documents a real release gap: path-scoped automation missed component-only changes, and a manual umbrella `0.6.1` publication was required.
