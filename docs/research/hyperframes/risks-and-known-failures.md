# Risks and known failures

Pinned source: `532caf7aa24fef382cb103013f6414bb547a4129`. Significant claims below use immutable commit links. HyperFrames owns timing in HTML data attributes plus a seekable browser runtime; CineKernel evaluates it as a wrapped web compatibility renderer and a source of protocol/conformance ideas.

| Package / decision | Entrypoint or evidence | Control flow / finding |
|---|---|---|
| issue | [issues/2775](https://github.com/heygen-com/hyperframes/issues/2775) | not tested; no fixed claim |
| issue | [issues/2057](https://github.com/heygen-com/hyperframes/issues/2057) | not tested; no fixed claim |
| issue | [issues/2107](https://github.com/heygen-com/hyperframes/issues/2107) | not tested; no fixed claim |
| engine | [packages/engine/src/services/captureFailure.ts](https://github.com/heygen-com/hyperframes/blob/532caf7aa24fef382cb103013f6414bb547a4129/packages/engine/src/services/captureFailure.ts) | fallback classification |

Errors and fallbacks are explicit in engine/producer services; caches require verified anchors; final success still depends on encoded-artifact verification. Recommendation confidence is medium pending full-profile probes.

## Failure model and mitigations

The linked issues are context only; no open/fixed status is inferred for the pinned commit. Concrete code risks include navigation/readiness timeouts, transient browser closure, memory exhaustion, BeginFrame stalls, drawElement verification drift, screenshot fallback differences, sparse-keyframe seeks, invalid static reuse, audio extraction/mix failure, FFmpeg backpressure, distributed chunk mismatch, GPU software fallback, and incomplete muxes. `captureFailure` classifies recoverable browser errors; session rollback/close bounds resource lifetime.

CineKernel contains these risks with strict lint/check, supervised process trees, heartbeat and stall monitoring, bounded warm-up/repetition groups, worker-mode matrices, decoded artifact verification, and Probes A–J. Probe G records network isolation as PASS or UNSUPPORTED rather than inferring offline behavior from source scanning. Probe I proves timeout cleanup/recovery; Probe J measures a real bounded queue. GPU evidence must name backend/adapter and software-fallback state.

Retries and caches are never correctness evidence. A renderer exit zero with a bad frame, missing tone, wrong pixel format, or timestamp gap fails. Canonical selection requires a clean revision and one exact manifest; historical results remain separately labeled.

Decision: **wrap** HyperFrames behind these boundaries, **derive** its explicit failure classification and rollback, **reimplement** authoritative evaluation/verification, and **reject** silent fallback or unbounded retry. Confidence: **medium-high** after representative local passes; final confidence remains conditional on full remote artifacts.
