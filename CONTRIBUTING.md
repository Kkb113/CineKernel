# Contributing

Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
--all-features -- -D warnings`, `cargo test --workspace --all-features`,
`pnpm typecheck`, and `pnpm test` before submitting changes. Generated runtime
data belongs under `.cinekernel/`; do not commit upstream clones or render output.

Copied or materially derived upstream code requires a complete entry in
`docs/source-lineage/upstream-inventory.yaml`.

