import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {resolve} from "node:path";
import test from "node:test";

const packageRoot = resolve(import.meta.dirname, "..");

test("Remotion dependency pin matches the upstream lock package version", () => {
  const manifest = JSON.parse(readFileSync(resolve(packageRoot, "package.json"), "utf8")) as {dependencies: Record<string,string>};
  const lock = JSON.parse(readFileSync(resolve(packageRoot, "../../benchmarks/upstreams.lock.json"), "utf8")) as {remotion: {release_or_package_version: string}};
  assert.equal(manifest.dependencies.remotion, lock.remotion.release_or_package_version);
  assert.equal(manifest.dependencies["@remotion/cli"], lock.remotion.release_or_package_version);
});

test("audio workload registers three independent local clips at scaled intervals", () => {
  const source = readFileSync(resolve(packageRoot, "src/video.tsx"), "utf8");
  for (const clip of ["clip-a.wav", "clip-b.wav", "clip-c.wav"]) {
    assert.match(source, new RegExp(`staticFile\\(\"${clip.replace(".", "\\.")}\"\\)`));
  }
  assert.doesNotMatch(source, /tone-windows\.wav/);
  assert.match(source, /from=\{at\(3\)\}/);
  assert.match(source, /from=\{at\(6\)\}/);
});

test("mixed workload retains exact four-scene proportions and textured 3D", () => {
  const source = readFileSync(resolve(packageRoot, "src/video.tsx"), "utf8");
  assert.match(source, /durationInFrames\*\.2/);
  assert.match(source, /durationInFrames\*4\/15/);
  assert.match(source, /durationInFrames\/3/);
  assert.match(source, /map=\{texture\}/);
  assert.match(source, /Render\. Verify\. Trust\./);
  assert.match(source, /const labels=\["Parse","Seek","Capture","Encode"\]/);
});

test("Remotion render-time sources contain no remote asset URLs", () => {
  const source=readFileSync(resolve(packageRoot,"src/video.tsx"),"utf8");
  const renderer=readFileSync(resolve(packageRoot,"scripts/render.ts"),"utf8");
  assert.doesNotMatch(`${source}\n${renderer}`,/https?:\/\//);
});

test("Remotion can use a pre-resolved browser for network-isolated renders", () => {
  const renderer=readFileSync(resolve(packageRoot,"scripts/render.ts"),"utf8");
  assert.match(renderer,/process\.env\.CINEKERNEL_BROWSER_EXECUTABLE/);
  assert.match(renderer,/args\.push\("--browser-executable",browserExecutable\)/);
});

test("Remotion can select a capability-compatible browser graphics backend", () => {
  const renderer=readFileSync(resolve(packageRoot,"scripts/render.ts"),"utf8");
  assert.match(renderer,/process\.env\.CINEKERNEL_REMOTION_GL/);
  assert.match(renderer,/args\.push\("--gl",gl\)/);
});

test("audio renders are padded and trimmed to the declared composition duration", () => {
  const renderer=readFileSync(resolve(packageRoot,"scripts/render.ts"),"utf8");
  assert.match(renderer,/apad,atrim=duration=/);
  assert.match(renderer,/["']-t["'],String\(durationSeconds\)/);
  assert.match(renderer,/["']--timeout["'],["']120000["']/);
});

test("Probe D still renderer reuses one browser and restarts it only after capture failure", () => {
  const source=readFileSync(resolve(packageRoot,"scripts/probe-stills.ts"),"utf8");
  assert.match(source,/await bundle\(/);
  assert.match(source,/browser\?\?=await openBrowser/);
  assert.match(source,/puppeteerInstance:browser/);
  assert.match(source,/await renderStill\(/);
  assert.match(source,/attempt<=3/);
  assert.match(source,/await closeBrowser\(\)/);
});
