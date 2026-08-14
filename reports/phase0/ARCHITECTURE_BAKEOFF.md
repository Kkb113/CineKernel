# Architecture bakeoff

| Candidate | Evidence | Strength | Blocking risk | Phase 0 role | Confidence |
|---|---|---|---|---|---|
| Remotion browser/React | retained smoke/full results, all correctness probes, and pinned archaeology | mature authoring/media/3D ecosystem | React state and browser capture cannot be authoritative | web compatibility | medium |
| HyperFrames HTML/browser | lint+strict-check+render smoke/full evidence and paired snapshot probe | explicit seek/capture fallback and validation | HTML state and capture-path variance | web compatibility/reference | medium |
| tiny-skia/resvg | verified smoke/full native 2D outputs and random-access parity | deterministic software reference | production text shaping/i18n absent | reference renderer | medium |
| wgpu | verified smoke/full offscreen 3D on Intel Arc/Vulkan, no software fallback, random-access parity | native 2D/3D, fast readback | cross-driver conformance and text/media integration | certified native candidate | medium |
| Skia/Vello | bounded documentation probe only | broad vector/text potential | build/binary complexity unmeasured | experimental | low |
| Blender | executable unavailable | cinematic quality and mature scene tools | startup, external workflow, deterministic integration | optional external cinematic | low |

Recommendation: keep browser engines wrapped; advance a native wgpu renderer
with a slower software reference. This is Proposed pending cross-platform and
multi-adapter conformance evidence, not an Accepted final renderer choice.
