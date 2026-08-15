# R0.02 acceptance report

## 1. Status

PASS. Local source archaeology, deterministic verification, the dedicated R0.02 workflow, and repository CI are complete across Windows, Ubuntu, and macOS.

## 2. Scope

Static, clean-room architecture mapping only. No ONDA execution, benchmark, product implementation, or IR selection occurred.

## 3. Locked base

CineKernel base 974d93ef224b75383499cdb2b70cc086a0dd6f40 and ONDA pin/tree match the R0.01 lock.

## 4. Method

Implementation, tests, manifests, repository documentation, official external sources, comments, then explicit inference.

## 5. Source coverage

Mandatory React, Cinema, Scene, animation, prepass, player, WASM, CLI/export, manifest, test, and repository boundaries are indexed.

## 6. Hypotheses

All seven registered hypotheses have a verified or rejected verdict with sources.

## 7. Authoring surfaces

Five distinct surfaces were mapped with ownership, identity, time, validation, extension, creativity, and output contracts.

## 8. React flow

A new reconciler root is mounted, committed, lowered, and unmounted for every requested frame.

## 9. Cinema flow

Cinema validates and resolves high-level editorial intent into React component structure.

## 10. Direct JSON

Scene JSON bypasses high-level authoring semantics and enters the typed renderer contract.

## 11. Rust flow

Typed Scene and optional second-based Timeline evaluation converge on Scene.

## 12. Scene graph

Composition plus a finite node tree is the universal renderer-facing language.

## 13. State

Evaluation, instance, module, process, and engine-shared mutable state were distinguished.

## 14. Mutability

Reconciliation and prepasses mutate or clone-and-rewrite; shared engines and media seeks require serialization.

## 15. Time

Composition, local, seconds, wall clock, audio, and video-bucket domains were mapped.

## 16. Identity

High-level string identities and payload paths do not form a complete source map into Scene nodes and pixels.

## 17. Semantic loss

Named intent, layout intent, SVG structure, and asset representation are consumed or materialized.

## 18. Validation

Hard errors, warnings, placeholders, and omissions exist at different boundaries.

## 19. Fallbacks

Preview fallback improves availability but can reduce fidelity without a typed end-to-end diagnostic.

## 20. Parity

Core renderer parity is strong only when equivalent prepasses and media are used; end-to-end parity is conditional.

## 21. Serialization

JSON version one is observed; a formal evolution and unknown-field contract remains open.

## 22. Prepasses

Source materialization, timeline selection, SVG expansion, image decode, video decode, and layout have distinct ownership.

## 23. Materialization

Whole-video Scene arrays and JSON are structurally duration-proportional; no performance claim is made.

## 24. Scalability

Bounded streaming is a candidate requirement, not an implemented result.

## 25. Creative programmability

Procedural authoring is broad; renderer vocabulary and advanced material semantics remain bounded.

## 26. Laptop litmus

Exploded layers are plausible with groups, transforms, assets, and depth, but semantic assembly constraints are weak.

## 27. Other litmus

Glass and chrome/liquid scenes permit stylized approximations; physical materials and simulation are not established.

## 28. React comparison

The fresh-per-frame root differs materially from normal retained React identity.

## 29. MLIR comparison

Explicit progressive IR levels would preserve intent and make semantic loss reviewable.

## 30. GStreamer comparison

Explicit clocks, flow, backpressure, and diagnostics are useful reference properties.

## 31. Requirements

Eight abstract, nonfinal candidate requirements are registered.

## 32. Clean room

Only facts, prose, identifiers needed for citation, and abstract diagrams are stored; no ONDA source is copied or translated.

## 33. Independence

Permanent dependencies on ONDA, Remotion, and HyperFrames remain zero.

## 34. Tests

The standalone verifier, source hashes, references, outputs, frozen-path and dependency guards, integrity, determinism, schema mutation checks, root format/check/clippy, and JavaScript typecheck/tests pass.

A local Windows timing assertion was slow, but the complete root Rust suite passed on all three hosted CI systems. The dedicated R0.02 suite also passed on all three systems.

## 35. Remote reproduction

PASS at commit c8d16e3d7d8029a3e2fe2e2e2019f48996533758. Dedicated R0.02 run 31898496016 and repository CI run 31898496054 both completed successfully on Windows, Ubuntu, and macOS. The remote attestation records the immutable run identifiers.

## 36. Immutability

Phase 0 and R0.01 frozen artifacts are unchanged.

## 37. Contradictions and questions

Claims of universal preview parity conflict with explicit Canvas and media fallback behavior. The frozen R0.01 integrity checker also scans future schema namespaces, so it must run in an exact-base worktree once R0.02 schemas exist. Open questions are routed to R0.03 through R0.08.

## 38. Recommendation

Proceed to R0.03 research. Do not select or implement a CineKernel IR from R0.02 alone, and keep this review PR draft and unmerged until reviewer sign-off.
