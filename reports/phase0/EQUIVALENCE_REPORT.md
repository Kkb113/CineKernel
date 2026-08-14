# Phase 0.1 workload equivalence

Equivalence means semantic and temporal intent is comparable; it does not imply identical implementation or pixels. Only `equivalent` rows enter performance comparisons.

| Case | Remotion | HyperFrames | Native | Mechanical evidence |
|---|---|---|---|---|
| Typography/layout | equivalent | equivalent | native-2d equivalent | real glyphs, title/copy checkpoints, decoded change/structure |
| Vector/effects | equivalent | equivalent | native-2d equivalent | mask/path progression, saturation and motion |
| Chart/diagram | equivalent | equivalent | native-2d equivalent | four labels/values and staged growth |
| Media sampling | equivalent | equivalent | unsupported | 240-frame decoded source, 15-frame offset, all 180 output frames, six browser worker modes |
| Audio/captions | equivalent | equivalent | unsupported | three distinct clips, spectral signatures, silence and seams |
| 3D scene | equivalent | equivalent | native-wgpu equivalent | textured lit cube, camera, floor, overlay, GPU/capture capability |
| Mixed 2D/3D | equivalent | equivalent | native-wgpu equivalent | exact 0–3/3–7/7–12/12–15 scenes and three cues |

The benchmark intent file is authoritative for support/equivalence declarations. Native renderers are not inserted into unsupported media/audio rows. Timing reports exclude preflight and artifact verification from the comparable `render_command` view while retaining those costs separately.

Canonical result counts: pending clean full run. Probe A–D equivalence outcomes: pending canonical probes.
