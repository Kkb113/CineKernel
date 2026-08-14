# Local render pipeline

Pinned source: `4e459b8b3aeec12ac8346666773ea28892a30e31`. The checkout is detached and verified before research. State is owned by React/browser packages; CineKernel treats this as evidence for a wrapped compatibility renderer, not future authoritative state.

| Package / decision | Entrypoint or evidence | Control flow / finding |
|---|---|---|
| renderer | [packages/renderer/src/render-media.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/renderer/src/render-media.ts) | top-level render contract |
| renderer | [packages/renderer/src/render-frames.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/renderer/src/render-frames.ts) | parallel frame production |
| renderer | [packages/renderer/src/stitch-frames-to-video.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/renderer/src/stitch-frames-to-video.ts) | encode assembly |

Timing is frame-derived; browser pages own evaluation, renderer workers own capture concurrency, and errors cross explicit promise/process boundaries. Preview and final paths share composition semantics but not identical capture/encode machinery. Recommendation confidence is medium until the full correctness matrix runs.

```mermaid
flowchart LR
A[renderMedia] --> B[serve/bundle] --> C[page pool] --> D[renderFrames] --> E[stitch]
```

## Concrete trace and ownership

The public `renderMedia(RenderMediaOptions)` wrapper delegates to the error-handled `internalRenderMedia`, whose raw implementation coordinates composition selection, browser acquisition, frame work, audio collection, and stitching. `RenderMediaProgress`, `StitchingState`, and `SlowFrame` make progress observable without making it authoritative timing evidence. Frame work crosses into `internalRenderFrames`; each returned `FrameAndAssets` couples a captured frame with discovered assets and emitted artifacts. `internalStitchFramesToVideo` consumes the completed frame range and audio description and owns final assembly.

| Concern | Owner | Behavior |
|---|---|---|
| state | renderer invocation plus browser page | options and progress live in the host; React state lives in each page |
| time | composition frame and fps | browser evaluation receives an exact frame; wall time only measures execution |
| resources | browser/page pool and FFmpeg children | pages, temporary frames, downloads, and encoders are released by cleanup paths |
| concurrency | `renderFrames` scheduler | frames may finish out of order; stitching restores ordinal output |
| errors/retries | error wrapper and frame retry layer | failures reject the render; only classified page/frame failures get bounded retry |
| cache | bundle/download/browser caches | cache reuse is an optimization and cannot establish artifact correctness |

The final path is `renderMedia → internalRenderMediaRaw → internalRenderFrames → page seek/capture → audio collection → innerStitchFramesToVideo`. Player and Studio use the same frame-derived composition model, but neither executes this exact page-pool, frame-file, and FFmpeg lifecycle. Relevant upstream coverage is concentrated beside `packages/renderer/src` and in CLI integration tests; CineKernel additionally verifies the mux independently because upstream success does not prove our benchmark contract.

Decision: **wrap** Remotion as a web-compatibility renderer; **derive** its explicit progress and cancellation ideas; **reject** browser/React state as CineKernel's authoritative render state. Confidence: **high** for the traced local call path, **medium** for cross-platform operational behavior until remote CI artifacts pass.
