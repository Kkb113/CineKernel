# Risks and known failures

Pinned source: `4e459b8b3aeec12ac8346666773ea28892a30e31`. The checkout is detached and verified before research. State is owned by React/browser packages; CineKernel treats this as evidence for a wrapped compatibility renderer, not future authoritative state.

| Package / decision | Entrypoint or evidence | Control flow / finding |
|---|---|---|
| issue | [issues/10274](https://github.com/remotion-dev/remotion/issues/10274) | not tested in Phase 0; no status inferred |
| issue | [issues/6041](https://github.com/remotion-dev/remotion/issues/6041) | not tested in Phase 0; no status inferred |
| renderer | [packages/renderer/src/seek-to-frame.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/renderer/src/seek-to-frame.ts) | source-frame correctness remains a probe target |

Timing is frame-derived; browser pages own evaluation, renderer workers own capture concurrency, and errors cross explicit promise/process boundaries. Preview and final paths share composition semantics but not identical capture/encode machinery. Recommendation confidence is medium until the full correctness matrix runs.
