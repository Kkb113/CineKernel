# Risk register

| Risk | Probability | Impact | Evidence | Mitigation | Owner | Close by | Status |
|---|---|---|---|---|---|---|---|
| Native scope expansion | high | high | Phase 0 renderer gaps | preserve bounded roles | renderer lead | Phase 2 | open |
| Text/i18n | high | high | native 2D uses placeholders | select shaping stack + golden corpus | text lead | Phase 2 | open |
| GPU conformance/WebGPU drivers | medium | high | one Arc/Vulkan adapter only | cross-OS adapter matrix | renderer lead | Phase 3 | open |
| Media source-frame correctness | medium | critical | oracle exists; matrix incomplete | sequential/parallel probes | media lead | Phase 1 | open |
| Parallel scheduling/backpressure | medium | critical | bounded simulation only | production bounded channels | kernel lead | Phase 1 | open |
| Preview/final parity | medium | high | capture paths differ | shared evaluator + paired frame probes | kernel lead | Phase 2 | open |
| Audio omission/seams | medium | high | fixtures exist; runs incomplete | energy and sample-count gates | media lead | Phase 1 | open |
| Cache poisoning/stale reuse | medium | high | upstream caches use anchors | content-addressed keys + invalidation | kernel lead | Phase 2 | open |
| Render-time network | medium | critical | static audit passes only | certified network sandbox | security lead | Phase 1 | open |
| License/lineage | low | high | policy/inventory exist | CI completeness validation | maintainers | Phase 1 | open |
| Binary/install complexity | medium | medium | wgpu/Chrome dependencies | role-specific packages | release lead | Phase 4 | open |
| 3D material portability | medium | high | one WGSL material | conformance scenes | 3D lead | Phase 3 | open |
| FFmpeg build/license variation | medium | high | machine build captured | capability manifest | media lead | Phase 1 | open |
| Benchmark fairness/visual quality | medium | high | neutral spec exists | decoded comparisons + review | performance lead | Phase 0 | open |
| Future IR overfit | medium | critical | BenchmarkIntentSpec temporary | ADR-0004 boundary | architecture lead | Phase 1 | mitigated |

