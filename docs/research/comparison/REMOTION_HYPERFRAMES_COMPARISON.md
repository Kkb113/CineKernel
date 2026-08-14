# Remotion / HyperFrames comparison

| Dimension | Remotion 4e459b8b | HyperFrames 532caf7a | CineKernel implication |
|---|---|---|---|
| Authoring | React composition tree | HTML/data attributes | Neither becomes authoritative IR |
| Timing | frame context | exact-time seek protocol | one rational-time evaluator |
| Capture | browser page/frame pool | drawElement, BeginFrame, screenshot fallback | wrapped compatibility backends |
| Media | OffthreadVideo + parser packages | extraction/injection + mixer | native media contract with oracles |
| Audio | asset collection + FFmpeg | explicit mixer/envelopes | audio is a verified first-class graph |
| 3D | R3F/ThreeCanvas/WebGPU | Three/TypeGPU adapters | native wgpu certified candidate |
| Concurrency | renderer workers/Lambda chunks | parallel coordinator/distributed producers | bounded, correctness-preserving scheduler |
| Validation | renderer tests/Studio | lint + runtime/layout/motion/contrast | unified semantic verification |
| License/lineage | external licensed dependency | Apache-2.0 dependency | wrap; record every derivation |

Both browser systems are valuable compatibility references. HyperFrames exposes stronger capture/validation instrumentation; Remotion has a mature React authoring and renderer ecosystem. Neither satisfies the locked authoritative-IR direction unchanged.
