# Research source index

All repository sources are from the locked ONDA pin and tree. Line ranges identify reviewed evidence; file hashes and blobs make the records independently checkable. No ONDA code was executed.

| ID | Class | Path or URL | Evidence |
|---|---|---|---|
| S-README | documentation | `README.md` | Publicly described authoring and rendering surfaces. |
| S-ROOT-CARGO | manifest | `Cargo.toml` | Rust package and renderer boundaries. |
| S-ROOT-PKG | manifest | `package.json` | JavaScript package and test boundaries. |
| S-REACT-HOST | implementation | `packages/react/src/host-config.ts` | Custom reconciler uses mutable HostNode children. |
| S-REACT-FRAME | implementation | `packages/react/src/frame.ts` | Frame and video configuration are context values. |
| S-REACT-SEQUENCE | implementation | `packages/react/src/sequence.ts` | Sequences translate global frame to local frame. |
| S-REACT-LOWER | implementation | `packages/react/src/reconciler.ts` | Each requested frame creates, commits, lowers, unmounts, and discards a fresh React root. |
| S-REACT-STATE | implementation | `packages/react/src/fonts.ts` | Registered font bytes are process-global mutable state. |
| S-REACT-WARM | implementation | `packages/react/src/warmers.ts` | Warmers are process-global and failures are best-effort. |
| S-REACT-TEST | test | `packages/react/src/reconciler.test.tsx` | Tests establish frame evaluation and lowering behavior. |
| S-CINEMA-TYPES | implementation | `packages/cinema/src/types.ts` | Cinema preserves scene, track, entry, role, component, choreography, transition, brand, and finish semantics before lowering. |
| S-CINEMA-TIME | implementation | `packages/cinema/src/timing.ts` | Time specifications become rounded frames. |
| S-CINEMA-COMPILER | implementation | `packages/cinema/src/index.tsx` | Cinema resolves a finite choreography and transition vocabulary to React elements. |
| S-CINEMA-INSPECT | implementation | `packages/cinema/src/inspect/index.ts` | Inspection runs over the high-level Cinema payload rather than the lowered Scene. |
| S-CINEMA-RESOLVE | implementation | `packages/cinema/src/inspect/resolve.ts` | Inspector identities derive from entry id or payload path and retain role labels. |
| S-CINEMA-TEST | test | `packages/cinema/src/validate.test.tsx` | Tests distinguish unknown component errors from unknown choreography warnings. |
| S-SCENE | implementation | `packages/scene-rs/src/lib.rs` | Scene is a composition plus a finite node tree. |
| S-ANIMATION | implementation | `packages/animation-rs/src/lib.rs` | Rust animation stores keyframe time in seconds and evaluates a cloned scene by NodeId. |
| S-LAYOUT | implementation | `packages/layout-rs/src/lib.rs` | Layout is a clone-and-rewrite prepass that materializes absolute child translations. |
| S-IMAGE | implementation | `packages/image-rs/src/lib.rs` | Image decoding attaches skipped pixel data and may downscale. |
| S-SVG | implementation | `packages/svg-rs/src/lib.rs` | SVG is parsed and lowered to supported scene shapes. |
| S-NODE-EXPORT | implementation | `packages/render/src/index.ts` | Node export materializes all React scenes into a temporary JSON array and invokes the native CLI. |
| S-PLAYER | implementation | `packages/player/src/player.tsx` | Preview renders requested frames on demand. |
| S-AUDIO | implementation | `packages/player/src/audio-engine.ts` | Preview audio has an instance clock mapping and is muted off-speed. |
| S-VIDEO | implementation | `packages/player/src/video.ts` | Preview video is bucketed at thirty source frames per second and may hold the last good decoded frame. |
| S-CANVAS | implementation | `packages/player/src/canvas-renderer.ts` | Canvas fallback omits image and SVG nodes and approximates text and procedural gradients. |
| S-WASM | implementation | `packages/wasm/src/lib.rs` | CPU WASM parses Scene JSON then conditionally runs image and layout prepasses before rendering. |
| S-WASM-VELLO | implementation | `packages/wasm-vello/src/lib.rs` | GPU WASM follows the same JSON, image, and layout boundary before asynchronous rendering. |
| S-CLI | implementation | `packages/cli-rs/src/main.rs` | Native commands parse scene JSON and run ordered source, timeline, SVG, image, layout, and render stages. |
| S-COMPONENTS | implementation | `packages/components/src/manifest.ts` | The packaged component catalog is finite and schema-described. |
| E-REACT | external-official | `https://react.dev/learn/render-and-commit` | React separates recursive render calculation from commit mutations and recommends pure rendering. |
| E-MLIR | external-official | `https://mlir.llvm.org/docs/Rationale/MLIRForGraphAlgorithms/` | Multi-level representations can preserve useful abstraction until progressive lowering is justified. |
| E-GSTREAMER | external-official | `https://gstreamer.freedesktop.org/documentation/additional/design/overview.html` | A media pipeline makes graph direction, element boundaries, messages, and a shared running-time clock explicit. |
