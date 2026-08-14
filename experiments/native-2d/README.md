# Native software 2D experiment

The working Phase 0 path combines `tiny-skia` rasterization with `resvg` SVG
rendering. Text is intentionally represented by deterministic placeholder bars;
production text shaping, font fallback, bidi, and internationalization remain a
Phase 1+ architecture risk. Frames are driven only by exact frame time, encoded
through FFmpeg, and verified by the shared harness.

Skia and Vello are evaluated separately in `experiments/skia-feasibility/` and
the architecture bakeoff; neither is silently rejected because of setup cost.

