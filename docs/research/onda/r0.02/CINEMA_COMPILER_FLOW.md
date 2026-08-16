# Cinema compiler flow

```mermaid
flowchart TD
  P[Cinema payload] --> V[Validate payload and timing]
  P --> I[Inspector semantic analysis]
  V --> T[Normalize TimeSpec]
  T --> G[Resolve scenes tracks entries roles]
  G --> C[Invoke registry components]
  C --> M[Materialize choreography transitions placement]
  M --> R[React composition]
  R --> S[Renderer-facing Scene]
```

Cinema preserves high-level identities during validation and inspection, then consumes many of them while constructing React elements. Roles, names, brand-token identity, choreography names, and transition names are generally not first-class Scene data.
