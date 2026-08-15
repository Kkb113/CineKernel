# 1. Executive status

PASS — exact source/release/licensing/provenance lock established; legal questions are quarantined and no reuse decision depends on them.

# 2. CineKernel base revision

5f47f341aa546b4ceb115fcad71d576d0ab85f29

# 3. Research branch and commits

`research/r0.01-onda-provenance`; harness/schemas/tests commit `d35e31615ab7d9ef6e348ccde1a4b243dc364bc8`; the evidence commit is the commit containing this generated report.

# 4. ONDA selected research pin

3ddf1780c9799bf038ac90cec7d8cadb61acafbe

# 5. Repository and tree identity

639df83ebf0262afccd6d021bf6d16ef19777d85

# 6. Commit verification

GitHub signature VERIFIED; independent ls-remote and API observations matched the selected pin.

# 7. Module inventory

See `MODULE_INVENTORY.*`.

# 8. Rust workspace inventory

19 members at workspace version 0.1.0.

# 9. JavaScript workspace inventory

13 packages/apps; umbrella version 0.6.1.

# 10. Dependency graph

416 locked Cargo packages plus statically parsed pnpm workspaces.

# 11. Feature/dependency reachability

Default CLI and optional video, segment, transcribe, speak, WASM, GPU, audio, typography and layout surfaces are separated.

# 12. Build-time dependencies

CMake, native C/C++/clang, wasm-bindgen, Node, pnpm, Bun and Rust are separately recorded.

# 13. Runtime external dependencies

FFmpeg and Vulkan/lavapipe are explicit external boundaries.

# 14. Model and data artifacts

U2-Net, Whisper, Kokoro model/voices and ONNX Runtime metadata recorded; weights were not downloaded.

# 15. Release streams

3 independent streams.

# 16. GitHub release provenance

Latest embed kit v0.2.16, release 353301462, asset 475692944.

# 17. npm release provenance

Public onda-engine@0.6.1 verified; registry signature observed; no provenance attestation observed.

# 18. Distribution surfaces

Source tree, npm umbrella, scoped packages, WASM packages and native embed kit remain distinct.

# 19. License files and hashes

Exact Git blob and SHA-256 values are in `UPSTREAM_LOCK.json`.

# 20. Dependency license surface

Derived inventory is not replaced by upstream NOTICE.

# 21. Copyleft/license hotspots

9 hotspots; factual chains recorded; LEGAL_REVIEW_REQUIRED.

# 22. FSL future-license evidence

CURRENT_FSL; FUTURE_APACHE_TEXT_PRESENT; CANDIDATE_DATE_CALCULATED; LEGAL_EFFECTIVE_DATE_NOT_CONFIRMED.

# 23. Clean-room policy

Committed and governing R0.02–R0.08.

# 24. Independence guards

PASS.

# 25. Exact-copy guard

PASS; exact nontrivial source-content check, not a general plagiarism proof.

# 26. Phase 0 immutability

PASS; frozen paths unchanged.

# 27. Automated test results

PASS: 65 Rust tests and 27 JavaScript tests passed; 32 of the Rust tests directly cover R0.01. Formatting, all-target/all-feature check, strict Clippy and JavaScript typecheck passed.

# 28. Reproducibility/idempotency results

PASS: two consecutive report generations produced 15 byte-identical committed research documents with zero SHA-256 differences.

# 29. Inconsistencies

See structured inconsistency register.

# 30. Unresolved factual questions

Scoped registry contents and artifact internals remain unverified.

# 31. Legal-review-required questions

FSL effect, FFmpeg build terms, GPL/eSpeak chain, model/data rights and distribution-specific obligations.

# 32. Risks carried into R0.02

Researchers must produce abstract requirements only and use primary sources before specification.

# 33. R0.02 recommendation

PROCEED only under the committed clean-room policy and this immutable pin.

Machine summary: Rust members 19; resolved packages 416; JavaScript packages 13; artifacts 13; release streams 3; hotspots 9.
