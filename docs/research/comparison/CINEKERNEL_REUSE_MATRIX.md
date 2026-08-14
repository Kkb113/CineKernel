# CineKernel reuse matrix

| Subsystem | Decision | Required contract | Risk | Conformance test | Confidence |
|---|---|---|---|---|---|
| Remotion renderer | WRAP | verified VideoIR-to-composition adapter | preview/final drift | still vs decoded final | medium |
| HyperFrames engine | WRAP | local-only deterministic composition | capture fallback drift | cross-path frame comparison | medium |
| Media parsers | ADOPT | bounded, provenance-aware reads | codec edge cases | color-frame oracle | medium |
| Seek protocols | DERIVE | exact rational time, idempotent seek | hidden browser state | shuffled evaluation | high |
| Authoritative React/HTML | REJECT | VideoIR/SceneIR authority | source-state coupling | serialization conformance | high |
| Native 2D | REIMPLEMENT | reference pixels and text shaping | i18n scope | golden frames | medium |
| Native wgpu | REIMPLEMENT | deterministic offscreen 2D/3D | driver conformance | cross-adapter probes | medium |
| Encoding/backpressure | DERIVE | bounded queues, verified mux | RSS growth | slow-consumer probe | high |
