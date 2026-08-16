# R0.02 acceptance report

## 1. Executive status

CONDITIONAL PASS pending a final remediation-head remote reproduction and updated attestation. All local remediation gates pass before publication.

## 2. CineKernel base revision

Base `974d93ef224b75383499cdb2b70cc086a0dd6f40`; accepted base tree `80ebf050ebc298b7647a403159ab59f94811468f`.

## 3. ONDA research identity

The only authority is the frozen R0.01 `UPSTREAM_LOCK.json`: repository `https://github.com/onda-engine/onda-engine.git`, pin `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`, tree `639df83ebf0262afccd6d021bf6d16ef19777d85`.

## 4. Branch and commits

Branch `research/r0.02-onda-scene-compiler-archaeology`; PR #13 remains draft and unmerged. Existing commits are recorded in the reviewer packet; the final attestation distinguishes the evidence commit from any later documentation-only commit.

## 5. Clean-room confirmation

ONDA was inspected statically and never built, executed, tested, rendered, or benchmarked. Exact-file, normalized multiline-fragment, dependency-alias, Git-dependency, absolute-path, tracked-upstream, and frozen-path guards pass.

## 6. Source coverage

50 pinned ONDA files are fully verified and 3 official external references are indexed separately. All mandatory paths are present.

## 7. Authoring surfaces discovered

Five surfaces: Cinema, React, direct Scene JSON, typed Rust Scene/Timeline, and the component registry. The inspector is a semantic analysis route, not an authoring surface.

## 8. Complete architecture graph

15 nodes and 18 edges use controlled vocabularies and record authority, mutability, time, identity, data form, validation, semantic disposition, errors, and immutable source references.

## 9. Cinema/agent authoring flow

Cinema validates and normalizes timing and semantic payload fields, invokes registry components and named patterns, then emits React. High-level names and roles are mostly consumed before Scene.

## 10. React authoring flow

Fresh roots and HostNode trees are created per requested frame, lowered, and unmounted. Module-global active frame/DOF state leaves concurrent or nested reentrancy unresolved.

## 11. Direct JSON flow

Direct JSON enters at Scene deserialization/version handling and therefore bypasses Cinema and React contracts.

## 12. Rust Scene and Timeline flow

Typed Rust construction is explicit and close to the renderer contract. Timeline evaluation clones and mutates a Scene using numeric NodeId targets.

## 13. Scene graph authority

Scene is authoritative at the renderer boundary, not at the high-level authoring boundary.

## 14. State ownership

12 state records identify authority, scope, mutability, sharing, persistence, provenance, and reentrancy.

## 15. Mutability and reentrancy

Fresh React roots isolate host trees; global frame/DOF state, shared GPU coordination, caches, and external media clocks require explicit serialization or further proof.

## 16. Time ownership

12 temporal conversions record source/target domains, operation, rounding, clamping, negative/fractional behavior, rate ownership, and precision risk.

## 17. Identity and source mapping

10 identity transitions distinguish preservation, remapping, lowering-only use, React-key-only use, dropping, and non-representability.

## 18. Lowering and semantic preservation

31 semantic rows record disposition, stage, reversibility, and editing/diagnostic/repair/incremental impacts.

## 19. Information loss

Roles, labels, registry names, React keys, choreography names, transition names, brand-token identity, diagnostics, fidelity class, and provenance are consumed or dropped before pixels.

## 20. Validation and error behavior

21 failure/fallback records cover every mandatory behavior and explicitly state diagnosis, semantic/visual/timing/determinism impact, parity difference, and repairability.

## 21. Fallbacks and approximations

Fallbacks include warnings, omission, placeholders, default substitution, asynchronous repaint, GPU-to-CPU demotion, and Canvas approximation; quality reduction is recorded explicitly.

## 22. Preview/export parity

7 preview/export rows distinguish shared evaluation from browser/native media, scheduling, renderer, and fallback differences.

## 23. Serialization and versioning

Scene JSON is versioned, but forward compatibility and semantic migration guarantees remain unresolved and deferred.

## 24. Prepasses

Layout, image, SVG, timeline, and media resolution materialize or rewrite Scene data before rendering.

## 25. Materialization and scalability hypotheses

React export builds a complete Scene array and temporary JSON; motion blur multiplies evaluations. This is a scaling hypothesis only—R0.02 performed no benchmark.

## 26. Creative-programmability assessment

Five surfaces are assessed categorically across primitive access, procedural logic, extension, registry/pattern dependence, descent to primitives, inspectability, editability, source mapping, novel-scene expression, and black-box risk. No numeric score is used.

## 27. Laptop exploded-layers litmus

The laptop case separately covers hierarchy, segmentation, exploded-view planning, collision avoidance, geometry, materials, emissive lighting, camera, particles, timing, sound, semantic grouping, inspectability, editability, and asset truthfulness.

## 28. Additional novel-scene litmus results

Glass-city and liquid-spacecraft cases are compared across all five surfaces. Both are partly composable but general material, lighting, and simulation capabilities are not established.

## 29. Independent primary-source comparison

Official React, MLIR, and GStreamer sources provide independent comparisons for render/commit purity, progressive lowering, and clock/pipeline ownership.

## 30. CineKernel candidate requirements

8 requirement candidates use CK-R002 IDs, cite ONDA and independent sources, identify affected programs and impacts, require later research, prohibit reuse, and remain CANDIDATE_ONLY.

## 31. Contradictions found

3 explicit contradictions remain recorded rather than smoothed over.

## 32. Unresolved questions

6 unresolved questions remain open.

## 33. Deferred topics

6 topics are routed to R0.03–R0.08.

## 34. Automated tests

63 fixture-driven unit and mutation tests pass, covering lock/checkout failures, provenance, strict nested schemas, graph integrity, semantic/fallback/requirement completeness, creative-taxonomy and novel-scene mutations, clean-room aliases and fragments, absolute paths, sorting, and two-run determinism.

## 35. Deterministic generation

The report command generates 16 machine projections, 20 human reports, and 17 schemas: 53 generated files. The canonical model is an input, not a generated output.

## 36. Remote reproduction

Pending remediation-head Windows, Ubuntu, and macOS reproduction. The attestation is updated only after those runs complete and a nonempty raw-evidence artifact is published.

## 37. Phase 0 and R0.01 immutability

No Phase 0 or R0.01 frozen file is modified. The R0.01 verifier is executed from the exact accepted base inside each R0.02 OS job.

## 38. Recommendation for R0.03

Do not start R0.03 until remediation-head three-OS evidence succeeds and the reviewer accepts promotion from CONDITIONAL PASS to PASS.
