# Testing and conformance

Pinned source: `4e459b8b3aeec12ac8346666773ea28892a30e31`. The checkout is detached and verified before research. State is owned by React/browser packages; CineKernel treats this as evidence for a wrapped compatibility renderer, not future authoritative state.

| Package / decision | Entrypoint or evidence | Control flow / finding |
|---|---|---|
| renderer | [packages/renderer/src](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/renderer/src) | renderer unit/integration tests |
| bundler | [packages/bundler/src/test/relocatable-bundle.test.ts](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/bundler/src/test/relocatable-bundle.test.ts) | bundle relocation |
| media-parser | [packages/media-parser/src/test](https://github.com/remotion-dev/remotion/blob/4e459b8b3aeec12ac8346666773ea28892a30e31/packages/media-parser/src/test) | media corpus tests |

Timing is frame-derived; browser pages own evaluation, renderer workers own capture concurrency, and errors cross explicit promise/process boundaries. Preview and final paths share composition semantics but not identical capture/encode machinery. Recommendation confidence is medium until the full correctness matrix runs.

## Concrete conformance strategy

Upstream renderer tests exercise API validation, page/capture behavior, codecs, media, and error handling near `packages/renderer/src`; bundler relocation tests protect bundles from absolute-path assumptions; the media-parser corpus validates container parsing. These tests establish upstream behavior at the pinned commit, not CineKernel's benchmark intent or our platform matrix.

CineKernel tests the integration boundary at four levels. Unit tests read actual package manifests and lock metadata, validate every workload's semantic checkpoints, and assert separate clip registration and exact mixed-scene proportions. Rust tests cover timestamp failures, frequency detection, decoded hash selection, path normalization, redaction, cleanup guards, and structured child output. Smoke renders exercise real Chromium, FFmpeg, and native binaries. Canonical full runs add repetitions, worker modes, probes, and immutable revision/spec/lock hashes.

Conformance success requires both process success and central verifier success. Historical results cannot satisfy canonical selection. Warm-ups cannot be counted. A dirty revision cannot create canonical evidence. Remote CI must pass on Windows, Linux, and macOS, and uploaded artifacts retain structured logs/results.

Decision: **adopt** the upstream test corpus as dependency evidence, **derive** its focused package tests, and **reimplement** cross-engine semantic/mux/concurrency conformance around CineKernel's contract. Confidence: **high** in the test architecture; final acceptance remains conditional on recorded remote runs.
