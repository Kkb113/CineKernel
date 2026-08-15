import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {resolve} from "node:path";
import test from "node:test";
import {createComposition} from "../src/composition.js";

test("HyperFrames dependency pin matches the upstream lock package version", () => {
  const root = resolve(import.meta.dirname, "../../..");
  const manifest = JSON.parse(readFileSync(resolve(root, "packages/phase0-hyperframes/package.json"), "utf8")) as {dependencies: Record<string,string>};
  const lock = JSON.parse(readFileSync(resolve(root, "benchmarks/upstreams.lock.json"), "utf8")) as {hyperframes: {release_or_package_version: string}};
  assert.equal(manifest.dependencies.hyperframes, lock.hyperframes.release_or_package_version);
});

test("composition is deterministic, local-only, seek-driven, and uses direct clips", () => {
  const html=createComposition({caseId:"mixed-2d-3d",width:640,height:360,duration:3});
  assert.match(html,/data-composition-id="main"/);
  assert.match(html,/data-start="0"/);
  assert.doesNotMatch(html,/https?:\/\//);
  assert.match(html,/hf-seek/);
  assert.match(html,/paused:true/);
  assert.equal((html.match(/class="clip"/g)??[]).length,4);
});

test("audio case registers three separate clips and scaled gaps", () => {
  const html=createComposition({caseId:"audio-captions",width:640,height:360,duration:1.6});
  assert.equal((html.match(/<audio /g)??[]).length,3);
  for (const clip of ["clip-a.wav","clip-b.wav","clip-c.wav"]) assert.match(html,new RegExp(clip.replace(".","\\.")));
  assert.doesNotMatch(html,/tone-windows\.wav/);
});

test("mixed scene contains real chart labels, deterministic texture, 3D overlay, and CTA", () => {
  const html=createComposition({caseId:"mixed-2d-3d",width:640,height:360,duration:15});
  for (const text of ["Parse","Seek","Capture","Encode","Exact-time 3D","Render. Verify. Trust."]) assert.ok(html.includes(text));
  assert.match(html,/CanvasTexture/);
  assert.match(html,/data-start="3"/);
  assert.match(html,/data-start="7"/);
  assert.match(html,/data-start="12"/);
});

test("invalid HyperFrames benchmark case is rejected", () => {
  assert.throws(()=>createComposition({caseId:"deliberately-invalid",width:640,height:360,duration:1}),/Unknown case/);
});

test("benchmark preflight gates errors without promoting software-WebGL warnings to failures", () => {
  const root = resolve(import.meta.dirname, "../../..");
  const source = readFileSync(resolve(root, "packages/phase0-hyperframes/scripts/render.ts"), "utf8");
  assert.match(source, /run\(\["check",project\]\)/);
  assert.doesNotMatch(source, /run\(\["lint",project\]\)/);
  assert.doesNotMatch(source, /--strict/);
  assert.match(source, /stdio:"inherit"/);
});
