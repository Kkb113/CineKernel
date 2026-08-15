# Phase 0.1 closure evidence

Status: **PASS**

Implementation revision: `907a2551c3dad27c698ac43d7ecb41957236be53`.

The closure implementation adds a depth-tested native 3D floor, separates Linux Probe G into its own workflow, and makes native-wgpu execution capability-aware. Because the native renderer changed, the required 3D and mixed groups were rerun at the frozen implementation revision on the reference Windows/Intel Arc/Vulkan machine. All 24 measured outputs passed the permanent artifact verifier; each group also completed its warm-up.

| Engine | Case | Canonical run | Measured | Verified | Failures | Median `render_command` |
|---|---|---|---:|---:|---:|---:|
| native-wgpu | 3d-scene | `20260814T182238Z-f18d767c-4a06-4c64-be6b-b63bac7b7f9e` | 5 | 5 | 0 | 6,808.3 ms |
| native-wgpu | mixed-2d-3d | `20260814T182337Z-a975af9a-a0a8-4104-8c45-e203a5f0877c` | 3 | 3 | 0 | 11,173.8 ms |
| Remotion | 3d-scene | `20260814T182448Z-5246a50e-5c07-4d44-9c8c-f21796fdc06e` | 5 | 5 | 0 | 18,948.3 ms |
| Remotion | mixed-2d-3d | `20260814T182728Z-ac8daa43-d1d3-4955-b1fb-3f0ee2a73df5` | 3 | 3 | 0 | 38,520.1 ms |
| HyperFrames | 3d-scene | `20260814T183054Z-7da72e0f-3150-49c8-a844-0a65d1e24ca0` | 5 | 5 | 0 | 17,368.6 ms |
| HyperFrames | mixed-2d-3d | `20260814T183412Z-40975fc4-90b7-4a25-a011-6947ff85c2e4` | 3 | 3 | 0 | 27,647.1 ms |

The original 109-result reference-machine run remains the Phase 0.1 aggregate baseline for unchanged workloads. These focused results replace the native 3D/mixed implementation evidence and confirm the unchanged Remotion/HyperFrames 3D/mixed reference groups; they are not presented as a new whole-suite aggregate. HyperFrames preflight remains excluded from `render_command` exactly as in the established timing methodology and remains separately recorded in each raw result.

Remote closure is complete:

- capability-aware full/all run [31855973437](https://github.com/Kkb113/CineKernel/actions/runs/31855973437) rendered and verified 99/99 macOS, 101/101 Ubuntu, and 101/101 Windows results;
- macOS retained-evidence attestation [31870436549](https://github.com/Kkb113/CineKernel/actions/runs/31870436549) passed probes A-F and H-J with 9 PASS / 0 FAIL / 0 UNSUPPORTED; and
- dedicated Ubuntu network-isolation run [31855975438](https://github.com/Kkb113/CineKernel/actions/runs/31855975438) passed Probe G for Remotion and HyperFrames in a loopback-only network namespace.

The original macOS job-level failure occurred only after successful rendering and 99/99 verification, in the old per-still Probe D browser lifecycle. The corrected probe reuses one browser and does not change native 3D, canonical render output, or performance timing. The reference-machine aggregate and focused native-floor performance runs therefore remain valid. See `REMOTE_CLOSURE_ATTESTATION.md` for artifact IDs and hashes.
