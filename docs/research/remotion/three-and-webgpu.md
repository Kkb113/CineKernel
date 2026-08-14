# Three.js and WebGPU

Pinned source: `4e459b8b3aeec12ac8346666773ea28892a30e31`. The checkout is detached and verified before research. State is owned by React/browser packages; CineKernel treats this as evidence for a wrapped compatibility renderer, not future authoritative state.

| Package / decision | Entrypoint or evidence | Control flow / finding |
|---|---|---|
| three | [packages/three/src/ThreeCanvas.tsx](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/three/src/ThreeCanvas.tsx) | R3F canvas and frame-driven render |
| three | [packages/three/src/webgpu.tsx](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/three/src/webgpu.tsx) | WebGPU integration |
| core | [packages/core/src/use-current-frame.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/core/src/use-current-frame.ts) | deterministic time source |

Timing is frame-derived; browser pages own evaluation, renderer workers own capture concurrency, and errors cross explicit promise/process boundaries. Preview and final paths share composition semantics but not identical capture/encode machinery. Recommendation confidence is medium until the full correctness matrix runs.

```mermaid
flowchart LR
A[useCurrentFrame] --> B[ThreeCanvas] --> C[R3F scene]
B --> D[WebGL/WebGPU]
```
