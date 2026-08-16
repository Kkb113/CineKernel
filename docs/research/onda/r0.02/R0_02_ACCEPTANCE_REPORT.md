# R0.02 acceptance report

## Locked future-phase registry

- R0.03 — Native GPU, CPU, WASM, and encoding architecture
- R0.04 — Typography, layout, effects, color, and 3D architecture
- R0.05 — Agent component catalog and cinematic composition model
- R0.06 — CLI, installation, preview, embedding, and developer experience
- R0.07 — Independent benchmark and failure analysis
- R0.08 — Adoption, rejection, clean-room, and roadmap-delta matrix

## 1. Executive status

CONDITIONAL PASS — NOT LOCKED. The merged research tree is retained, but this independent-review closure branch requires exact-head three-OS evidence and explicit reviewer approval before R0.02 can lock.

## 2. CineKernel base revision

Base `974d93ef224b75383499cdb2b70cc086a0dd6f40`; accepted base tree `80ebf050ebc298b7647a403159ab59f94811468f`.

## 3. ONDA research identity

The only authority is the frozen R0.01 `UPSTREAM_LOCK.json`: repository `https://github.com/onda-engine/onda-engine.git`, pin `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`, tree `639df83ebf0262afccd6d021bf6d16ef19777d85`.

## 4. Branch and commits

PR #13 was merged before independent approval: branch head `6e7ff3d6016829357bb7f804dd916e6f7e796a64`, master merge commit `12024231b8983b07d9413cf96f4579bd9495f946`, common research tree `e01b2fe87d409e34f509847cdd66214d174eb0d6`. This corrective work is on `research/r0.02-independent-review-closure`; its exact evidence head remains pending until publication.

## 5. Clean-room confirmation

ONDA was inspected statically and never built, executed, tested, rendered, or benchmarked. Exact-file, normalized multiline-fragment, authoritative package-identity, renamed dependency, Git source, ONDA-checkout path, lockfile identity, absolute-path, tracked-upstream, and frozen-path guards pass.

## 6. Source coverage

74 pinned ONDA source records and 3 official external references are indexed. Mandatory whole-file entries are COVERAGE_ONLY; only narrow symbol-and-line entries marked CLAIM_SUPPORTING may satisfy formal claims.

## 7. Authoring surfaces discovered

Five surfaces: Cinema, React, direct Scene JSON, typed Rust Scene/Timeline, and the component registry. The inspector is a semantic analysis route, not an authoring surface.

## 8. Complete architecture graph

15 nodes and 18 edges use controlled vocabularies and record authority, mutability, time, identity, data form, validation, semantic disposition, errors, and immutable source references.

## 9. Cinema/agent authoring flow

Cinema validates and normalizes timing and semantic payload fields, invokes registry components and named patterns, then emits React. High-level names and roles are mostly consumed before Scene.

## 10. React authoring flow

React directly supports host-language frame logic, interpolation, springs, sequences, loops, transitions, and primitive Scene nodes. Fresh roots and HostNode trees are created per requested frame, lowered, and unmounted; module-global frame/DOF state still leaves concurrent or nested reentrancy unresolved.

## 11. Direct JSON flow

Direct JSON enters at Scene deserialization/version handling and composes the finite declarative Scene schema. It does not natively define a new component implementation.

## 12. Rust Scene and Timeline flow

Typed Rust construction directly builds the finite Scene and Timeline model without Cinema-registry dependence. Timeline evaluation clones and mutates a Scene using numeric NodeId targets.

## 13. Scene graph authority

Scene is authoritative at the renderer boundary, not at the high-level authoring boundary.

## 14. State ownership

12 state records identify creation, versioning, authority, scope, mutability, sharing, persistence, provenance, and evidence-backed concurrency status.

## 15. Mutability and reentrancy

Fresh React roots isolate host trees; global frame/DOF state, shared GPU coordination, caches, and external media clocks retain evidence-backed UNKNOWN or UNRESOLVED concurrency where safety is not proved.

## 16. Time ownership

12 temporal conversions record source/target domains, operation, rounding, clamping, negative/fractional behavior, rate ownership, and precision risk.

## 17. Identity and source mapping

10 identity transitions distinguish preservation, remapping, lowering-only use, React-key-only use, dropping, and non-representability.

## 18. Lowering and semantic preservation

31 individually mapped semantic rows record exact source/target representations, disposition, stage, reversibility, and editing/diagnostic/repair/incremental impacts.

## 19. Information loss

Roles, labels, registry names, React keys, choreography names, transition names, brand-token identity, diagnostics, fidelity class, and provenance are consumed or dropped before pixels.

## 20. Validation and error behavior

22 failure/fallback records distinguish structured, stderr, UI-only, silent demotion, and silent-skip visibility and include the native network materialization boundary.

## 21. Fallbacks and approximations

Bad preview fonts and failed preview images are silently skipped; GPU/CPU failures silently demote renderer state without an agent diagnostic; native remote media is best-effort materialized to temporary files and failed URLs are retained after stderr notice for decoder skip.

## 22. Preview/export parity

7 preview/export rows carry feature-specific differences, certification impact, and focused source references.

## 23. Serialization and versioning

Scene JSON is versioned, but forward compatibility and semantic migration guarantees remain unresolved and routed to R0.03 and R0.08.

## 24. Prepasses

Layout, image, SVG, timeline, and media resolution materialize or rewrite Scene data before rendering.

## 25. Materialization and scalability hypotheses

React export builds a complete Scene array and temporary JSON; motion blur multiplies evaluations. This is a scaling hypothesis only—measurement belongs in R0.07.

## 26. Creative-programmability assessment

Five surfaces use categorical capability states rather than ambiguous Booleans. React host-language animation and primitive descent are supported; JSON custom components and Rust registry dependence are not native; component identity is only partial before lowering.

## 27. Laptop exploded-layers litmus

The laptop case distinguishes manually supplied or presegmented hierarchy from automatic semantic/mechanical segmentation, which is not established. Sound uses audio-specific authoring/runtime evidence, and authoritative assets remain required for claims about real product construction.

## 28. Additional novel-scene litmus results

Glass-city and liquid-spacecraft cases are compared across all five surfaces. Both are partly composable but general material, lighting, and simulation capabilities are not established.

## 29. Independent primary-source comparison

Official React, MLIR, and GStreamer sources provide independent comparisons for render/commit purity, progressive lowering, and clock/pipeline ownership.

## 30. CineKernel candidate requirements

8 requirement candidates cite requirement-specific claims and sources, use locked P01–P28 program IDs, tailor all four impacts, route through the locked R0 registry, prohibit reuse, and remain CANDIDATE_ONLY.

## 31. Contradictions found

3 explicit contradictions remain recorded rather than smoothed over.

## 32. Unresolved questions

6 unresolved questions remain open and are routed without renaming later phases.

## 33. Deferred topics

6 topics preserve the exact locked definitions for R0.03–R0.08: Native GPU/CPU/WASM/encoding; visual/3D; agent components; developer experience; benchmark/failure analysis; and final adoption/roadmap decisions.

## 34. Automated tests

82 fixture-driven unit and mutation tests cover source evidence roles, symbol-in-range validation, semantic tuple independence, silent fallback visibility, renderer demotion, network materialization in the graph/prepass, independently required schema contracts, exact-head attestation, architecture contracts, provenance, clean-room copying, stable ordering, and deterministic generation.

## 35. Deterministic generation

The report command generates 16 machine projections, 20 human reports, and 17 schemas: 53 generated files. The canonical model is an input, not a generated output.

## 36. Remote reproduction

Historical final PR head `6e7ff3d6016829357bb7f804dd916e6f7e796a64` passed dedicated run `31927730892` and ordinary CI `31927730849` on Windows, Ubuntu, and macOS. Because this closure changes executable verifier/model code, new exact-closure-head runs and artifacts are mandatory and still pending.

## 37. Phase 0 and R0.01 immutability

No Phase 0 or R0.01 frozen file is modified. The R0.01 verifier is executed from the exact accepted base inside each R0.02 OS job.

## 38. Recommendation for R0.03

Do not start R0.03. Publish this focused closure as a new review PR, obtain exact-head dedicated and ordinary three-OS success, require explicit independent approval before merge, and verify post-merge tree equality.
