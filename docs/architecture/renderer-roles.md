# Renderer roles

```mermaid
flowchart LR
  A["CineKernel semantic evaluation"] --> B["Software 2D reference"]
  A --> C["Native wgpu candidate"]
  A --> D["Remotion compatibility wrapper"]
  A --> E["HyperFrames compatibility wrapper"]
  A -. optional .-> F["External cinematic renderer"]
  B --> V["Decoded-frame and mux verification"]
  C --> V
  D --> V
  E --> V
  F --> V
```

- Reference renderer: native software 2D, intentionally slow and testable.
- Certified native candidate: wgpu offscreen 2D/3D plus verified FFmpeg encoding.
- Web compatibility: wrapped Remotion and HyperFrames baselines.
- External cinematic: optional Blender executable only.
- Experimental: Skia/Vello feasibility until cross-platform evidence exists.
