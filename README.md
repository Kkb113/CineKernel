# CineKernel

CineKernel is an open-source, agent-native video compiler and deterministic
2D/3D media runtime. This repository currently contains the Phase 0 evidence
harness: comparable browser baselines, native renderer feasibility experiments,
correctness probes, reproducible upstream archaeology, and architecture reports.

## Phase 0 quickstart

Prerequisites: Git, Rust 1.97.1, Node 24, Corepack/pnpm 11.8, FFmpeg/FFprobe, and a
Chrome-family browser. Docker and Blender are optional.

```powershell
corepack enable
pnpm install --frozen-lockfile
cargo xtask doctor
cargo xtask upstream sync
cargo xtask upstream verify
cargo xtask phase0 prepare
cargo xtask phase0 run --profile smoke
cargo xtask phase0 verify
cargo xtask phase0 report
```

Machine-specific and large generated artifacts are isolated under
`.cinekernel/`. `BenchmarkIntentSpec` is only a Phase 0 interchange format; it is
not the future CineKernel VideoIR.
