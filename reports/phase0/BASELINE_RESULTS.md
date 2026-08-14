# Historical Phase 0 baseline results (non-canonical)

These pre-remediation v1 results are retained for history only. They are not merged into or cited as Phase 0.1 canonical evidence.

Generated from all retained `result.json` files under `.cinekernel/runs/`. Timing summaries include only verified successful attempts; all 14 failed attempts remain in `BASELINE_RESULTS.json` under `raw_results`. Successful attempts: 121. Total retained attempts: 135.

| Engine / case / profile | n | min ms | median ms | mean ms | max ms | stddev ms |
|---|---:|---:|---:|---:|---:|---:|
| hyperframes/3d-scene/full | 5 | 27012.3 | 27804.1 | 27589.6 | 28144.4 | 436.4 |
| hyperframes/3d-scene/smoke | 1 | 23124.2 | 23124.2 | 23124.2 | 23124.2 | 0.0 |
| hyperframes/audio-captions/full | 5 | 51319.0 | 51909.0 | 52126.4 | 53773.7 | 895.4 |
| hyperframes/audio-captions/smoke | 1 | 29194.3 | 29194.3 | 29194.3 | 29194.3 | 0.0 |
| hyperframes/chart-diagram/full | 5 | 27796.2 | 28103.7 | 28272.0 | 28980.2 | 411.1 |
| hyperframes/chart-diagram/smoke | 1 | 21774.9 | 21774.9 | 21774.9 | 21774.9 | 0.0 |
| hyperframes/media-frame-sampling/full | 5 | 25278.3 | 25550.0 | 25669.7 | 26228.6 | 362.5 |
| hyperframes/media-frame-sampling/smoke | 3 | 14217.1 | 15042.6 | 16815.3 | 21186.1 | 3109.0 |
| hyperframes/mixed-2d-3d/full | 3 | 38584.8 | 38929.1 | 38834.4 | 38989.3 | 178.2 |
| hyperframes/mixed-2d-3d/smoke | 1 | 27792.3 | 27792.3 | 27792.3 | 27792.3 | 0.0 |
| hyperframes/typography-layout/full | 10 | 25055.2 | 25405.7 | 25380.2 | 25832.8 | 248.2 |
| hyperframes/typography-layout/smoke | 2 | 20500.6 | 21121.1 | 21121.1 | 21741.6 | 620.5 |
| hyperframes/vector-effects/full | 5 | 25784.1 | 26512.7 | 26312.6 | 26919.4 | 438.2 |
| hyperframes/vector-effects/smoke | 1 | 20043.5 | 20043.5 | 20043.5 | 20043.5 | 0.0 |
| native-2d/chart-diagram/full | 5 | 4786.8 | 4838.4 | 4848.1 | 4914.0 | 55.9 |
| native-2d/chart-diagram/smoke | 1 | 780.8 | 780.8 | 780.8 | 780.8 | 0.0 |
| native-2d/typography-layout/full | 5 | 4332.7 | 4364.3 | 4422.6 | 4562.2 | 95.3 |
| native-2d/typography-layout/smoke | 2 | 800.1 | 21497.3 | 21497.3 | 42194.5 | 20697.2 |
| native-2d/vector-effects/full | 5 | 15964.6 | 16176.0 | 16499.8 | 17552.8 | 585.6 |
| native-2d/vector-effects/smoke | 1 | 1146.3 | 1146.3 | 1146.3 | 1146.3 | 0.0 |
| native-wgpu/3d-scene/full | 5 | 5739.0 | 5832.7 | 5874.3 | 6120.3 | 143.2 |
| native-wgpu/3d-scene/smoke | 2 | 1452.4 | 79803.2 | 79803.2 | 158154.0 | 78350.8 |
| native-wgpu/mixed-2d-3d/full | 3 | 9867.9 | 10166.4 | 10098.6 | 10261.3 | 167.6 |
| native-wgpu/mixed-2d-3d/smoke | 1 | 1852.2 | 1852.2 | 1852.2 | 1852.2 | 0.0 |
| remotion/3d-scene/full | 5 | 40648.9 | 42085.5 | 42065.5 | 42845.5 | 778.6 |
| remotion/3d-scene/smoke | 1 | 10967.1 | 10967.1 | 10967.1 | 10967.1 | 0.0 |
| remotion/audio-captions/full | 5 | 16012.2 | 24679.0 | 23130.8 | 25283.9 | 3568.3 |
| remotion/audio-captions/smoke | 1 | 9856.7 | 9856.7 | 9856.7 | 9856.7 | 0.0 |
| remotion/chart-diagram/full | 5 | 13971.9 | 21174.5 | 19350.4 | 21890.3 | 3005.9 |
| remotion/chart-diagram/smoke | 1 | 9724.2 | 9724.2 | 9724.2 | 9724.2 | 0.0 |
| remotion/media-frame-sampling/full | 5 | 15420.9 | 25526.1 | 21771.0 | 26003.4 | 4968.1 |
| remotion/media-frame-sampling/smoke | 3 | 5560.9 | 6400.2 | 7302.2 | 9945.6 | 1900.3 |
| remotion/mixed-2d-3d/full | 3 | 19797.3 | 23941.6 | 24353.3 | 29320.9 | 3898.9 |
| remotion/mixed-2d-3d/smoke | 1 | 12027.4 | 12027.4 | 12027.4 | 12027.4 | 0.0 |
| remotion/typography-layout/full | 5 | 18885.5 | 19185.2 | 19136.0 | 19335.5 | 148.2 |
| remotion/typography-layout/smoke | 2 | 12383.2 | 23171.0 | 23171.0 | 33958.7 | 10787.8 |
| remotion/vector-effects/full | 5 | 15692.3 | 28364.3 | 25544.2 | 28715.6 | 5011.9 |
| remotion/vector-effects/smoke | 1 | 9600.2 | 9600.2 | 9600.2 | 9600.2 | 0.0 |
