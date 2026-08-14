import {createHash} from "node:crypto";
import {execFileSync} from "node:child_process";
import {createRequire} from "node:module";
import {mkdirSync, readFileSync, rmSync, writeFileSync, copyFileSync} from "node:fs";
import {fileURLToPath} from "node:url";
import {dirname, resolve} from "node:path";
import {sourceFrameRgb} from "../src/index.js";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../../..");
const out = resolve(root, ".cinekernel/generated/fixtures");
const frames = resolve(out, "source-frames");
mkdirSync(frames, {recursive: true});
mkdirSync(resolve(out, "font"), {recursive: true});

const sha256 = (path: string): string =>
  createHash("sha256").update(readFileSync(path)).digest("hex");

const ppm = (width: number, height: number, pixel: (x: number, y: number) => readonly [number, number, number]): Buffer => {
  const header = Buffer.from(`P6\n${width} ${height}\n255\n`, "ascii");
  const data = Buffer.alloc(width * height * 3);
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const [r, g, b] = pixel(x, y);
      const offset = (y * width + x) * 3;
      data[offset] = r;
      data[offset + 1] = g;
      data[offset + 2] = b;
    }
  }
  return Buffer.concat([header, data]);
};

const oracle: Array<{frame: number; rgb: readonly [number, number, number]}> = [];
for (let frame = 0; frame < 240; frame++) {
  const rgb = sourceFrameRgb(frame);
  oracle.push({frame, rgb});
  const bits = frame;
  const data = ppm(320, 180, (x, y) => {
    if (y < 24 && x < 144) {
      const cell = Math.floor(x / 18);
      const on = ((bits >> cell) & 1) === 1;
      return on ? [255, 255, 255] : [0, 0, 0];
    }
    return rgb;
  });
  writeFileSync(resolve(frames, `frame-${String(frame).padStart(4, "0")}.ppm`), data);
}

const sourceVideo = resolve(out, "color-coded-source.mp4");
execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y", "-framerate", "30", "-i", resolve(frames, "frame-%04d.ppm"), "-c:v", "libx264", "-preset", "medium", "-crf", "12", "-pix_fmt", "yuv420p", "-g", "30", "-keyint_min", "30", "-sc_threshold", "0", "-movflags", "+faststart", sourceVideo], {stdio: "inherit"});
writeFileSync(resolve(out, "color-coded-oracle.json"), `${JSON.stringify({schema_version: "phase0.color-oracle.v1", width: 320, height: 180, fps: 30, frames: oracle}, null, 2)}\n`);

const sampleRate = 48_000;
const makeTone = (frequency: number, durationSeconds = 2): Buffer => {
  const samples = sampleRate * durationSeconds;
  const pcm = Buffer.alloc(samples * 2);
  for (let i = 0; i < samples; i++) {
    const t = i / sampleRate;
    const edge = Math.min(1, t / 0.02, (durationSeconds - t) / 0.02);
    const value = Math.round(Math.sin(2 * Math.PI * frequency * t) * 0.2 * Math.max(0, edge) * 32767);
    pcm.writeInt16LE(value, i * 2);
  }
  const wav = Buffer.alloc(44 + pcm.length);
  wav.write("RIFF", 0); wav.writeUInt32LE(36 + pcm.length, 4); wav.write("WAVE", 8);
  wav.write("fmt ", 12); wav.writeUInt32LE(16, 16); wav.writeUInt16LE(1, 20); wav.writeUInt16LE(1, 22);
  wav.writeUInt32LE(sampleRate, 24); wav.writeUInt32LE(sampleRate * 2, 28); wav.writeUInt16LE(2, 32); wav.writeUInt16LE(16, 34);
  wav.write("data", 36); wav.writeUInt32LE(pcm.length, 40); pcm.copy(wav, 44);
  return wav;
};
writeFileSync(resolve(out, "clip-a.wav"), makeTone(440));
writeFileSync(resolve(out, "clip-b.wav"), makeTone(660));
writeFileSync(resolve(out, "clip-c.wav"), makeTone(880));
writeFileSync(resolve(out, "invalid-overlap.json"), `${JSON.stringify({schema_version: "phase0.invalid-audio.v1", clips: [{asset: "clip-a.wav", start_seconds: 0}, {asset: "clip-b.wav", start_seconds: 1.5}, {asset: "missing-clip.wav", start_seconds: 6}], expected_verifier_result: "FAIL"}, null, 2)}\n`);

writeFileSync(resolve(out, "captions.vtt"), `WEBVTT\n\n00:00:00.000 --> 00:00:02.000\nDeterministic beginnings\n\n00:00:03.000 --> 00:00:05.000\nMeasured motion\n\n00:00:06.000 --> 00:00:08.000\nVerified output\n`);
writeFileSync(resolve(out, "chart-data.json"), `${JSON.stringify({labels: ["Parse", "Seek", "Capture", "Encode"], values: [42, 76, 61, 88]}, null, 2)}\n`);
writeFileSync(resolve(out, "vector-fixture.svg"), `<svg xmlns="http://www.w3.org/2000/svg" width="640" height="360" viewBox="0 0 640 360"><defs><linearGradient id="g"><stop stop-color="#65d6ff"/><stop offset="1" stop-color="#8b5cf6"/></linearGradient><filter id="s"><feGaussianBlur stdDeviation="8"/></filter><clipPath id="c"><circle cx="320" cy="180" r="130"/></clipPath></defs><rect width="640" height="360" fill="#08101f"/><circle cx="330" cy="195" r="125" fill="#000" opacity=".6" filter="url(#s)"/><path d="M120 240 C200 60 420 60 520 240 L450 290 H190Z" fill="url(#g)" clip-path="url(#c)"/></svg>\n`);

const texturePpm = resolve(out, "texture.ppm");
writeFileSync(texturePpm, ppm(64, 64, (x, y) => ((Math.floor(x / 8) + Math.floor(y / 8)) % 2 === 0 ? [46, 213, 255] : [139, 92, 246])));
execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-y", "-i", texturePpm, resolve(out, "texture.png")], {stdio: "inherit"});

const require = createRequire(import.meta.url);
const fontSource = require.resolve("@fontsource/inter/files/inter-latin-400-normal.woff2");
copyFileSync(fontSource, resolve(out, "font/inter-latin.woff2"));

writeFileSync(resolve(out, "cube.gltf"), `${JSON.stringify({asset: {version: "2.0", generator: "CineKernel Phase 0"}, scenes: [{nodes: [0]}], scene: 0, nodes: [{mesh: 0}], meshes: [{name: "ProceduralCube", primitives: []}]}, null, 2)}\n`);

const assetNames = ["color-coded-source.mp4", "color-coded-oracle.json", "clip-a.wav", "clip-b.wav", "clip-c.wav", "invalid-overlap.json", "captions.vtt", "chart-data.json", "vector-fixture.svg", "texture.png", "cube.gltf", "font/inter-latin.woff2"];
const manifest = {
  schema_version: "phase0.fixtures.v2",
  generator_version: "2.0.0",
  generation_command: "pnpm fixtures",
  assets: assetNames.map((name) => ({name, sha256: sha256(resolve(out, name)), provenance: name.includes("inter-") ? "@fontsource/inter 5.2.8, SIL Open Font License 1.1" : "generated original CineKernel Phase 0 fixture", expected_properties: name})),
};
writeFileSync(resolve(out, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
rmSync(frames, {recursive: true, force: true});
console.log(JSON.stringify({ok: true, output: ".cinekernel/generated/fixtures", assets: manifest.assets.length}));
