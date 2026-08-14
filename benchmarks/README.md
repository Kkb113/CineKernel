# Phase 0 benchmarks

`BenchmarkIntentSpec` is a temporary, framework-neutral Phase 0 interchange
format. It is explicitly not the future CineKernel VideoIR. The seven canonical
cases live in `specs/phase0-cases.json`; smoke and full measurement policies live
in `profiles/`.

All media is generated locally by `pnpm fixtures`. Generated assets, logs,
frames, videos, environment manifests, and raw results live under
`.cinekernel/`. Cross-engine comparisons use identical dimensions, frame rate,
timing, assets, codec (`h264` where available), and pixel format (`yuv420p`).

Phase timing values remain `null` when an engine does not expose them. Failed
runs are retained and never removed from statistical summaries.

