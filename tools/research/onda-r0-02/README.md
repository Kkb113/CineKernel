# ONDA R0.02 research verifier

This standalone Rust workspace validates the clean-room R0.02 architecture evidence. It reads the authoritative ONDA repository, pin, and tree only from the frozen R0.01 upstream lock. It never builds, executes, tests, renders, or benchmarks ONDA.

Supported commands:

- `verify --json`
- `inventory --json`
- `report --json`
- `guard --json`
- `integrity --check --json`

Each command also writes a raw, ignored JSON result under `.cinekernel/research/onda/r0.02/checks/` for CI artifact publication.

## Dependencies and licenses

| Dependency | Purpose | License family |
|---|---|---|
| anyhow | structured error propagation | MIT OR Apache-2.0 |
| clap | command-line parsing | MIT OR Apache-2.0 |
| serde / serde_json | strict model and JSON processing | MIT OR Apache-2.0 |
| sha2 | SHA-256 evidence and integrity hashing | MIT OR Apache-2.0 |
| walkdir | bounded filesystem traversal | Unlicense OR MIT |
| jsonschema | Draft 2020-12 validation | MIT |

No ONDA, Remotion, or HyperFrames package is a dependency.
