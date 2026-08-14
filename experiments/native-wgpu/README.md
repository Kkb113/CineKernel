# Native wgpu 2D/3D experiment

This experiment renders a textured, lit, depth-tested cube to an offscreen wgpu
texture. Every camera and model transform is `f(exact_time, parameters, seed)`;
there is no wall clock. Each frame is synchronized with
`device.poll(Maintain::Wait)`, copied to a mapped readback buffer, composited with
a deterministic 2D overlay, encoded by FFmpeg, and verified by the shared
harness. Adapter, backend, driver, device type, and CPU/software fallback are
reported in structured output.

