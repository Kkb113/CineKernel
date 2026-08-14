# Phase 0.1 artifact verifier

The permanent verifier is `crates/phase0-verifier`; every benchmark result is successful only when it passes.

General video checks: existence/nonempty file, exactly one video track, declared audio-track count, H.264, yuv420p, dimensions, 30/1 fps, positive time base no coarser than a frame, near-zero start, duration, counted/decoded frames, monotonic nonduplicate timestamps, unexpected gaps, black ratio, frozen runs, selected decoded SHA-256 hashes, and decoded checkpoint metrics.

General audio checks: AAC-decoded 48 kHz samples, channel count, sample count with explicit 2,048-sample codec tolerance, peak, 440/660/880 Hz Goertzel signatures, fixed codec-aware guarded silence windows, overlap classification, and boundary seam jumps.

Case checks use decoded evidence: typography structure/change; vector/chart motion and color; complete media mapping against the actual decoded 240-frame source; three audio clip signatures; 3D motion/texture; and mixed scene diversity plus three cues. The verifier writes a SHA-256-bound `verification-manifest.json` beside each result.

Failure-path coverage includes wrong dimensions/codec/pixel format, missing audio, timestamp duplicate/gap, corrupt output, wrong media mapping, semantic failure, and invalid overlap/missing-clip artifacts. Representative local smoke outcomes are PASS for native-2d typography, Remotion and HyperFrames media, Remotion and HyperFrames audio/captions, and native-wgpu mixed 2D/3D.

Canonical verifier totals and remote artifact links: pending clean full run and workflows.
