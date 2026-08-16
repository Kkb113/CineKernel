# React reconciler flow

```mermaid
sequenceDiagram
  participant Caller
  participant Global as Module-global frame/DOF
  participant Root as Fresh reconciler root
  participant Host as Mutable HostNode tree
  participant Scene
  Caller->>Global: install requested integer/fractional frame
  Caller->>Root: synchronous render
  Root->>Host: commit mutations
  Host->>Scene: lower with toNode
  Root-->>Caller: unmount
  Caller->>Global: restore evaluation state
```

A new root isolates each host tree and hook lifetime. It does **not** prove concurrent or nested reentrancy: module-global active frame and depth-of-field state remain shared. `renderFrames` accumulates the resulting Scene snapshots, while motion blur requests fractional subframes.
