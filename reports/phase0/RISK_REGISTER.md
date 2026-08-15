# Risk register

| Risk | Probability | Impact | Evidence | Mitigation | Owner | Close by | Status |
|---|---|---|---|---|---|---|---|
| Native scope expansion | high | high | Phase 0 intentionally leaves native media/audio unsupported as standalone cases | preserve declared support matrix and bounded renderer roles | renderer lead | Phase 2 | open |
| Text shaping/i18n | high | high | deterministic bitmap glyph rasterizer proves the pipeline, not production shaping | select shaping stack and multilingual golden corpus | text lead | Phase 2 | open |
| GPU conformance/WebGPU drivers | medium | high | local native evidence covers one Intel Arc adapter; normal CI covers software/build gates | add certified hardware/driver matrix and conformance scenes | renderer lead | Phase 3 | open |
| Media source-frame correctness | low | critical | complete decoded 240-frame oracle passes all six browser worker configurations | retain oracle and expand codec/keyframe corpus | media lead | Phase 1 | mitigated |
| Parallel scheduling/backpressure | medium | critical | Probe J uses a real FFmpeg subprocess and bounds the queue to three frames / 24,883,200 bytes | carry bounded channels into production scheduler | kernel lead | Phase 1 | mitigated for prototype |
| Preview/final parity | medium | high | paired browser captures and random-access probes pass on one machine | share evaluator and repeat across OS/browser versions | kernel lead | Phase 2 | mitigated |
| Audio omission/seams | low | high | decoded signatures, silence, sample-count, overlap rejection, and seam probes pass | retain audio gates and extend stereo/multichannel corpus | media lead | Phase 1 | mitigated |
| Cache poisoning/stale reuse | medium | high | upstream locks and content hashes are verified; sparse sync is idempotent | content-address all production caches and publish invalidation rules | kernel lead | Phase 2 | open |
| Render-time network | low | critical | Ubuntu `unshare --net` Probe G passes for Remotion and HyperFrames with loopback only and no unexpected external network availability | retain the dedicated isolation workflow and local-only asset policy | security lead | continuous | mitigated |
| License/lineage drift | low | high | full root license, upstream license hashes, 31-doc/four-entry validator PASS | keep lineage validation in CI and review upgrades explicitly | maintainers | Phase 1 | mitigated |
| Browser/binary installation complexity | medium | medium | Chrome, FFmpeg, Rust, and pnpm are exercised on three-OS normal CI | role-specific packages and doctor command | release lead | Phase 4 | open |
| 3D material portability | medium | high | one WGSL/textured-cube workload and one local hardware adapter | add multi-adapter conformance corpus | 3D lead | Phase 3 | open |
| FFmpeg build/license variation | medium | high | exact local build and encoder settings captured; OS CI installs differ | certified distribution and capability manifest | media lead | Phase 1 | open |
| Benchmark fairness/visual quality | medium | high | neutral intent, semantic verifier, bounded WebGL tolerance, and contact sheets exist | reviewer sign-off and additional quality metrics | performance lead | Phase 0 close | review |
| Future IR overfit | medium | critical | `BenchmarkIntentSpec` is explicitly temporary | enforce ADR-0004 boundary before VideoIR design | architecture lead | Phase 1 | mitigated |
