# Phase 0.1 canonical baseline results

Canonical run `20260814T144948Z-c6e0a98a-b94a-48e9-a26e-f69faf10f048` at implementation `0249b40ec41673ed8ac2f22c23583ddc3629a320`. Timing view: render-command elapsed time; HyperFrames lint/check and all artifact verification are reported separately. Direct rows include only `equivalence_level: equivalent`. Failed attempts: 0. Successful attempts: 109. Total retained attempts in this evidence set: 109.

| Engine / case / profile / worker | n | min ms | median ms | mean ms | max ms | stddev ms |
|---|---:|---:|---:|---:|---:|---:|
| hyperframes/3d-scene/full/auto | 5 | 15071.8 | 15357.4 | 16169.1 | 19241.1 | 1574.9 |
| hyperframes/audio-captions/full/auto | 5 | 38888.3 | 39432.5 | 39331.8 | 39608.9 | 248.8 |
| hyperframes/chart-diagram/full/auto | 5 | 15328.6 | 15440.2 | 15441.3 | 15569.8 | 85.1 |
| hyperframes/media-frame-sampling/full/1 | 5 | 17564.0 | 17724.9 | 17711.3 | 17868.5 | 101.9 |
| hyperframes/media-frame-sampling/full/4 | 5 | 11224.5 | 11463.6 | 11556.6 | 12109.9 | 295.5 |
| hyperframes/media-frame-sampling/full/auto | 5 | 12564.5 | 12794.2 | 12816.6 | 13144.9 | 189.9 |
| hyperframes/mixed-2d-3d/full/auto | 3 | 23903.1 | 24132.6 | 24231.8 | 24659.7 | 316.8 |
| hyperframes/typography-layout/full/auto | 5 | 12731.4 | 13017.7 | 12940.1 | 13106.4 | 146.6 |
| hyperframes/vector-effects/full/auto | 5 | 13640.4 | 14044.3 | 14017.0 | 14355.9 | 227.9 |
| native-2d/chart-diagram/full/default | 5 | 4494.7 | 4537.2 | 4654.5 | 5079.6 | 217.0 |
| native-2d/typography-layout/full/default | 5 | 4085.7 | 4155.3 | 4234.3 | 4649.0 | 209.7 |
| native-2d/vector-effects/full/default | 5 | 15632.8 | 16128.7 | 16086.2 | 16551.8 | 350.2 |
| native-wgpu/3d-scene/full/default | 5 | 5106.4 | 5123.0 | 5130.9 | 5158.7 | 18.8 |
| native-wgpu/mixed-2d-3d/full/default | 3 | 10604.4 | 10624.9 | 10688.4 | 10835.8 | 104.6 |
| remotion/3d-scene/full/default | 5 | 19759.0 | 20082.3 | 20186.9 | 20929.3 | 404.1 |
| remotion/audio-captions/full/default | 5 | 19659.0 | 19799.3 | 19842.5 | 20095.6 | 175.9 |
| remotion/chart-diagram/full/default | 5 | 14497.5 | 14966.5 | 14921.6 | 15219.3 | 249.9 |
| remotion/media-frame-sampling/full/1 | 5 | 21963.4 | 22401.3 | 22402.1 | 22676.3 | 258.3 |
| remotion/media-frame-sampling/full/4 | 5 | 13682.2 | 13802.3 | 13851.5 | 14104.9 | 144.2 |
| remotion/media-frame-sampling/full/default | 5 | 13052.2 | 13074.7 | 13253.1 | 13736.6 | 263.6 |
| remotion/mixed-2d-3d/full/default | 3 | 35318.1 | 35470.6 | 35496.9 | 35702.2 | 157.9 |
| remotion/typography-layout/full/default | 5 | 13253.8 | 13474.7 | 13451.9 | 13612.7 | 125.9 |
| remotion/vector-effects/full/default | 5 | 18659.5 | 18975.6 | 18965.6 | 19244.3 | 185.5 |
