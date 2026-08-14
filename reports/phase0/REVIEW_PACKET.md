# CineKernel Phase 0.1 reviewer packet

Current status: **conditional; evidence gates pending**. The authoritative status is `PHASE0_ACCEPTANCE_REPORT.md`.

## Review order

1. `REVIEW_FINDINGS_RESOLUTION.md` — all ten findings mapped to fixes and gates.
2. `EQUIVALENCE_REPORT.md` — which engine/case rows are comparable.
3. `VERIFIER_REPORT.md` — decoded artifact acceptance contract.
4. `SOURCE_ARCHAEOLOGY_SUMMARY.md` and `docs/research/{remotion,hyperframes}/` — pinned critical paths and decisions.
5. `CI_EVIDENCE.md` — actual remote runs only; currently pending.
6. `ARTIFACT_INDEX.md` — tracked summaries and raw artifact locations.

## Reviewer commands

```text
pnpm install --frozen-lockfile
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
pnpm typecheck
pnpm test
pnpm --filter @cinekernel/phase0-common lineage:validate
cargo xtask upstream verify --json
cargo xtask phase0 prepare --json
cargo xtask phase0 canonical-run --profile smoke --json
cargo xtask phase0 verify --canonical --json
cargo xtask phase0 canonical-run --profile full --json
cargo xtask phase0 probes --canonical --json
cargo xtask phase0 verify --canonical --json
cargo xtask phase0 report --canonical --json
```

Canonical commands intentionally reject a dirty or unborn revision. Raw runtime artifacts are ignored under `.cinekernel/` and uploaded by workflows; `node_modules`, `target`, `.cinekernel`, and user-supplied upstream source are excluded from Git.

## Decisions still requiring Phase 1 authority

- native text shaping/raster stack and golden corpus;
- certified GPU backend/adapter matrix;
- provider-neutral distributed scheduling;
- media parser and FFmpeg distribution boundaries;
- long representative workload and quality thresholds.
