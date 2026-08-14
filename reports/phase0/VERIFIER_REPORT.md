# Phase 0.1 artifact verifier

The permanent verifier is `crates/phase0-verifier`. A benchmark attempt is successful only after verification; command exit 0 alone is insufficient.

General video checks cover nonempty output, exactly one video track, declared audio-track count, H.264/yuv420p, dimensions, 30/1 fps, positive time base no coarser than a frame, near-zero start, duration, counted and decoded frame totals, monotonic nonduplicate timestamps, unexpected gaps, black ratio, bounded authored holds, selected decoded SHA-256 hashes, and checkpoint metrics.

General audio checks cover AAC-decoded 48 kHz samples, channels, sample count with an explicit 2,048-sample codec tolerance, peak, 440/660/880 Hz Goertzel signatures, guarded silence windows, overlap classification, and seam jumps. Case checks cover typography structure, vector/chart progression, the complete decoded media-frame mapping, three audio clips, 3D motion/texture, and mixed scene/audio diversity.

Every successful result has a SHA-256-bound `verification-manifest.json`. Final canonical verification reported:

- canonical run `20260814T144948Z-c6e0a98a-b94a-48e9-a26e-f69faf10f048`;
- implementation `0249b40ec41673ed8ac2f22c23583ddc3629a320`;
- 23 complete groups and 109/109 passing results;
- zero failed attempts in the canonical evidence set;
- post-probe re-verification also 109/109, proving the timeout fixture did not move the canonical pointer.

Failure-path tests cover dimensions, codec, pixel format, missing audio, timestamp duplicates/gaps, corrupt output, wrong media mapping, semantics, invalid overlap, and missing clips. Probe H independently confirmed all 109 canonical results carry timestamps, hashes, codec, and case checks. Probe I produced an invalid timeout record, killed the child process tree, wrote no successful result, and then recovered with exit 0.
