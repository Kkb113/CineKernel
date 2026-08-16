# Risks and open questions

## Contradictions

- **CON-001:** Fresh React roots isolate host trees, but module-global active frame and depth-of-field state leave concurrent or nested reentrancy unresolved.
- **CON-002:** Renderer sharing does not imply end-to-end preview/export parity because media scheduling and fallbacks differ.
- **CON-003:** A universal Scene vocabulary is still finite and Canvas preview explicitly approximates only a subset.

## Open questions

- **Q-001:** What scene-size and duration thresholds make full frame materialization unacceptable on target machines?
- **Q-002:** Which renderer capabilities are intentionally stable across CPU, Vello, and browser hosts?
- **Q-003:** What serialization evolution policy is promised beyond current version handling?
- **Q-004:** How should media clocks, frame rates, variable frame rate, and audio resampling compose?
- **Q-005:** What general material, shader, constraint, and simulation model is required for the target creative ceiling?
- **Q-006:** Which authoring semantics must remain editable after lowering and round-trip serialization?
