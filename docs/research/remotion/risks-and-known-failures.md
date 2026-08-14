# Risks and known failures

Pinned source: `4e459b8b3aeec12ac8346666773ea28892a30e31`. The checkout is detached and verified before research. State is owned by React/browser packages; CineKernel treats this as evidence for a wrapped compatibility renderer, not future authoritative state.

| Package / decision | Entrypoint or evidence | Control flow / finding |
|---|---|---|
| issue | [issues/10274](https://github.com/remotion-dev/remotion/issues/10274) | not tested in Phase 0; no status inferred |
| issue | [issues/6041](https://github.com/remotion-dev/remotion/issues/6041) | not tested in Phase 0; no status inferred |
| renderer | [packages/renderer/src/seek-to-frame.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/renderer/src/seek-to-frame.ts) | source-frame correctness remains a probe target |

Timing is frame-derived; browser pages own evaluation, renderer workers own capture concurrency, and errors cross explicit promise/process boundaries. Preview and final paths share composition semantics but not identical capture/encode machinery. Recommendation confidence is medium until the full correctness matrix runs.

## Failure model and mitigations

The cited issues are context, not proof that the pinned revision is affected or fixed. The code trace identifies concrete failure classes: closed browser targets, decode readiness, sparse keyframes, seek rounding, download/materialization errors, delay-render deadlocks, GPU completion, audio registration, FFmpeg failure, and partial/invalid muxes. `waitForReady`/`seekToFrame` and the bounded target-close retry expose some failures; others surface only in decoded output.

State and resource leaks are particularly dangerous because a hung Chromium/FFmpeg descendant can outlive the direct child. CineKernel's supervisor samples the process tree, emits heartbeats, enforces wall and stall deadlines, and kills the tree. Probe I injects a controlled hang, requires an invalid failure record with no valid result, and then proves recovery. Probe G removes network access on supported Linux runners. Probes A/B/D/H target instability, frame selection, preview/final drift, and mux integrity.

Cache or retry must never hide a deterministic semantic failure. Canonical results record warnings, capabilities, child timings, RSS/temp/output size, encoder settings, verifier output, clean revision, and source hashes. The risk remains that browser/GPU/codec behavior varies across platforms; remote artifacts are therefore part of acceptance, not optional corroboration.

Decision: **wrap** with strict supervision and verification, **derive** explicit readiness/retry classification, **reject** unbounded retries and process-only success, and avoid importing Remotion internals into the authoritative core. Confidence: **medium-high** after local smoke; **conditional** until all remote gates are green.
