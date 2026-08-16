# Direct JSON and Rust flow

```mermaid
flowchart LR
  JSON[Direct Scene JSON] --> D[Deserialize and version handling]
  Rust[Typed Rust Scene] --> Scene
  Timeline[Rust Timeline] --> Eval[Clone and evaluate at frame/fps]
  Scene --> Eval
  Eval --> Frame[Evaluated Scene]
  D --> P[Prepasses]
  Frame --> P
```

Direct JSON bypasses Cinema and React validation. Rust Scene and Timeline construction is **explicit and close to the renderer contract**; R0.02 makes no efficiency claim because benchmarking was prohibited.
