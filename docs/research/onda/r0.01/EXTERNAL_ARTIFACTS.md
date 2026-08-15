# R0.01 external binaries, models and data

| Artifact | Kind | Required by | Phase | Status |
|---|---|---|---|---|
| FFmpeg | system executable | onda-video/CLI video | RUNTIME_EXTERNAL | LEGAL_REVIEW_REQUIRED |
| Vulkan/lavapipe | system dependency | onda-vello | RUNTIME_EXTERNAL | LEGAL_REVIEW_REQUIRED |
| CMake and C/C++ toolchain | build tool | whisper-rs/espeak-rs/ONNX Runtime | BUILD_TIME | VERIFIED_AT_PIN |
| espeak-rs crate | registry package | onda-tts speak | BUILD_TIME | LEGAL_REVIEW_REQUIRED |
| vendored eSpeak NG source | vendored source in dependency | espeak-rs -> onda-tts speak | BUILD_TIME | LEGAL_REVIEW_REQUIRED |
| eSpeak NG language/data assets | data dependency | onda-tts speak | BUILD_TIME | LEGAL_REVIEW_REQUIRED |
| system espeak-ng/espeak-ng-data packages | system dependency | embed-kit speak feature release build | BUILD_TIME | LEGAL_REVIEW_REQUIRED |
| wasm-bindgen CLI | build tool | WASM packages | BUILD_TIME | VERIFIED_AT_PIN |
| Node.js | runtime tool | pnpm workspace | BUILD_TIME | VERIFIED_AT_PIN |
| pnpm | build tool | JavaScript workspace | BUILD_TIME | VERIFIED_AT_PIN |
| Bun | build tool | release tooling | BUILD_TIME | UNRESOLVED |
| Rust toolchain | build tool | all Rust modules | BUILD_TIME | VERIFIED_AT_PIN |
| U2-Net model | downloaded model | onda-segment | MODEL_DATA | LEGAL_REVIEW_REQUIRED |
| Whisper tiny.en model | downloaded model | onda-transcribe | MODEL_DATA | LEGAL_REVIEW_REQUIRED |
| Whisper base.en model | downloaded model | onda-transcribe | MODEL_DATA | LEGAL_REVIEW_REQUIRED |
| Whisper small.en model | downloaded model | onda-transcribe | MODEL_DATA | LEGAL_REVIEW_REQUIRED |
| Kokoro ONNX model | downloaded model | onda-tts speak | MODEL_DATA | LEGAL_REVIEW_REQUIRED |
| Kokoro voice bundle | downloaded model | onda-tts speak | MODEL_DATA | LEGAL_REVIEW_REQUIRED |
| ONNX Runtime prebuilt binary | downloaded binary | ort/onda-segment/onda-tts | BUILD_TIME | LEGAL_REVIEW_REQUIRED |
