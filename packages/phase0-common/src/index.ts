export type Engine = "remotion" | "hyperframes" | "native-2d" | "native-wgpu";
export type Profile = "smoke" | "full";

export interface BenchmarkCase {
  readonly id: string;
  readonly title: string;
  readonly purpose: string;
  readonly duration_seconds: number;
  readonly resolution: {readonly width: number; readonly height: number};
  readonly fps: "30/1";
  readonly local_assets: readonly string[];
  readonly scene_boundaries_seconds: readonly number[];
  readonly expected_visual_events: readonly string[];
  readonly expected_audio_events: readonly string[];
  readonly expected_frame_count: number;
  readonly verification_points_seconds: readonly number[];
  readonly supported_engines: readonly Engine[];
}

export interface BenchmarkIntentSpec {
  readonly schema_version: "phase0.benchmark-intent.v1";
  readonly notice: "A Phase 0 benchmark interchange format, not the future CineKernel VideoIR.";
  readonly cases: readonly BenchmarkCase[];
}

export const sourceFrameRgb = (frame: number): readonly [number, number, number] => [
  (frame * 47 + 17) % 256,
  (frame * 97 + 31) % 256,
  (frame * 193 + 53) % 256,
];

export const frameCount = (durationSeconds: number, fps = 30): number =>
  Math.round(durationSeconds * fps);

