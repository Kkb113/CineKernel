# Package map and ownership

Pinned source: `4e459b8b3aeec12ac8346666773ea28892a30e31`. The checkout is detached and verified before research. State is owned by React/browser packages; CineKernel treats this as evidence for a wrapped compatibility renderer, not future authoritative state.

| Package / decision | Entrypoint or evidence | Control flow / finding |
|---|---|---|
| core | [packages/core/src/index.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/core/src/index.ts) | authoring primitives and React state |
| renderer | [packages/renderer/src/render-media.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/renderer/src/render-media.ts) | local render orchestration |
| bundler | [packages/bundler/src/bundle.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/bundler/src/bundle.ts) | project bundling |
| player | [packages/player/src/index.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/player/src/index.ts) | embedded preview |
| studio | [packages/studio/src/index.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/studio/src/index.ts) | editing UI |
| three | [packages/three/src/ThreeCanvas.tsx](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/three/src/ThreeCanvas.tsx) | R3F integration |

Timing is frame-derived; browser pages own evaluation, renderer workers own capture concurrency, and errors cross explicit promise/process boundaries. Preview and final paths share composition semantics but not identical capture/encode machinery. Recommendation confidence is medium until the full correctness matrix runs.

```mermaid
flowchart LR
A[React entry] --> B[Bundle] --> C[Browser pages] --> D[Frames] --> E[FFmpeg]
```
