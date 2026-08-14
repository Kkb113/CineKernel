# Phase 0.1 source archaeology summary

Research is tied to verified detached source commits, source-tree hashes, package integrity, release relationships, and license hashes in `benchmarks/upstreams.lock.json` and `docs/source-lineage/upstream-inventory.yaml`.

Remotion critical paths trace `renderMedia`, `renderFrames`, page/frame retry, `seekToFrame`, `OffthreadVideo`, media parsing/materialization, audio collection, FFmpeg argument construction, stitching/mux, `ThreeCanvas`/WebGPU completion, Player/Studio, tests, and Lambda chunking. The decision is to wrap the browser ecosystem, derive explicit progress/readiness/bounded retry, and keep React/browser state outside authoritative CineKernel state.

HyperFrames critical paths trace runtime protocol inspection, adapter seek dispatch/completion, capture-session rollback, drawElement/BeginFrame/screenshot selection, static anchor reuse and extraction cache publication, audio parsing/mixing, bounded streaming reorder, FFmpeg encoding/mux, layered validation, Three/TypeGPU adapters, and distributed plan/assembly exports. The decision is to derive versioned exact-time protocols, bounded backpressure, atomic caches, and explicit failure classification while wrapping compatibility behavior.

Each of the 17 required documents records module, public entry, important functions/types, call sequence, state/time/resource owners, concurrency, error/retry/cache behavior, preview/final differences, relevant tests/issues, adopt/derive/reimplement/wrap/reject decision, and confidence. Immutable-link and local-reference validation is automated by `packages/phase0-common/scripts/validate-lineage.ts`.
