# Architecture overview

The authoritative machine graph contains **15 nodes and 18 edges**. Cinema, React, direct Scene JSON, and typed Rust converge on a finite Scene representation through explicit validation, reconciliation, serialization, prepass, preview, renderer, fallback, and encoder boundaries.

```mermaid
flowchart LR
  C[Cinema payload] --> V[Cinema validation]
  C --> I[Inspector semantic analysis]
  C --> R[React program]
  R --> X[Custom reconciler]
  X --> H[Mutable HostNode tree]
  H --> S[Per-frame Scene]
  J[Direct Scene JSON] --> P[Prepasses]
  T[Rust Timeline] --> S
  S --> P
  P --> CPU[CPU renderer]
  P --> GPU[GPU renderer]
  GPU -. capability fallback .-> CPU
  CPU -. preview fallback .-> Canvas[Canvas approximation]
  CPU --> E[Encoder]
  GPU --> E
```

The graph is a compiler map, not a CineKernel implementation design. Every node and edge carries immutable source references.
