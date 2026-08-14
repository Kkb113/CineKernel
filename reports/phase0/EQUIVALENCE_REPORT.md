# Phase 0.1 workload equivalence

Equivalence means comparable semantic and temporal intent; it does not claim identical implementation or pixels. Only rows declared `equivalent` in `benchmarks/phase0/workload-intent.json` enter the performance tables.

| Case | Remotion | HyperFrames | Native | Mechanical evidence |
|---|---|---|---|---|
| Typography/layout | equivalent | equivalent | native-2d equivalent | authored glyphs, title/copy checkpoints, decoded structure/change |
| Vector/effects | equivalent | equivalent | native-2d equivalent | mask/path progression, saturation, motion checkpoints |
| Chart/diagram | equivalent | equivalent | native-2d equivalent | four matching labels/values and staged growth |
| Media sampling | equivalent | equivalent | unsupported | decoded 240-frame unique source oracle, fixed offset, every output frame, worker modes 1/4/default-or-auto |
| Audio/captions | equivalent | equivalent | unsupported | three independent local clips, frequency signatures, silence windows, seam bounds |
| 3D scene | equivalent | equivalent | native-wgpu equivalent | textured lit cube, camera motion, floor, overlay, hardware adapter evidence |
| Mixed 2D/3D | equivalent | equivalent | native-wgpu equivalent | exact 0-3/3-7/7-12/12-15 second scenes and three audio cues |

Canonical full evidence contains 109 successful measured results in 23 engine/case/worker groups, with zero failed attempts. Probe A found exact decoded framemd5 across repetitions except the explicitly scoped Remotion mixed WebGL row; that row passed its documented PSNR >= 35 dB and SSIM >= 0.98 bounds (minimum observed PSNR 37.184885 dB, SSIM 0.98721). Probe D passed all semantic checkpoint counts: media 9, generic visual 15, and Remotion 3D tolerance 20.

Representative mixed outputs are in `artifacts/MIXED_EQUIVALENCE_CONTACT_SHEET.png`; media-oracle outputs are in `artifacts/MEDIA_ORACLE_CONTACT_SHEET.png`. Timing comparison excludes HyperFrames preflight and all artifact verification from `render_command`, while retaining those costs separately in each v2 result.
