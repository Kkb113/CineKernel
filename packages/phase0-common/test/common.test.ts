import assert from "node:assert/strict";
import {spawnSync} from "node:child_process";
import {readFileSync} from "node:fs";
import {createHash} from "node:crypto";
import {createRequire} from "node:module";
import test from "node:test";
import {resolve} from "node:path";
import {frameCount, sourceFrameRgb} from "../src/index.js";

const require = createRequire(import.meta.url);
const Ajv2020 = require("ajv/dist/2020").default as new (options: {strict: boolean}) => {validate: (schema: unknown, data: unknown) => boolean; errors: unknown};
const addFormats = require("ajv-formats").default as (ajv: object) => void;

test("frame counts are exact at 30fps", () => {
  assert.equal(frameCount(15), 450);
  assert.equal(frameCount(8), 240);
});

test("source frame markers are deterministic and unique across the 240-frame guarded fixture", () => {
  const colors = new Set(Array.from({length: 240}, (_, index) => sourceFrameRgb(index).join(",")));
  assert.equal(colors.size, 240);
  assert.deepEqual(sourceFrameRgb(0), [15, 15, 15]);
});

test("benchmark intent validates against its schema", () => {
  const root = resolve(import.meta.dirname, "../../..");
  const schema = JSON.parse(readFileSync(resolve(root, "schemas/phase0/benchmark-intent.schema.json"), "utf8"));
  const spec = JSON.parse(readFileSync(resolve(root, "benchmarks/specs/phase0-cases.json"), "utf8"));
  const ajv = new Ajv2020({strict: true});
  addFormats(ajv);
  assert.equal(ajv.validate(schema, spec), true, JSON.stringify(ajv.errors));
});

test("upstream lock validates against its schema", () => {
  const root = resolve(import.meta.dirname, "../../..");
  const schema = JSON.parse(readFileSync(resolve(root, "schemas/phase0/upstream-lock.schema.json"), "utf8"));
  const lock = JSON.parse(readFileSync(resolve(root, "benchmarks/upstreams.lock.json"), "utf8"));
  const ajv = new Ajv2020({strict: true});
  addFormats(ajv);
  assert.equal(ajv.validate(schema, lock), true, JSON.stringify(ajv.errors));
});

test("canonical result v2 schema accepts a complete result and rejects dirty evidence", () => {
  const root = resolve(import.meta.dirname, "../../..");
  const schema = JSON.parse(readFileSync(resolve(root, "schemas/phase0/benchmark-result.schema.json"), "utf8"));
  const result = {schema_version:"phase0.result.v2",canonical_run_id:"run",canonical:true,timestamp_utc:"2026-08-14T00:00:00.000Z",implementation_revision:"a".repeat(40),worktree_clean:true,environment_id:"b".repeat(64),benchmark_spec_sha256:"c".repeat(64),upstream_lock_sha256:"d".repeat(64),engine:"native-2d",engine_version:"0.0.0",upstream_commit:null,case_id:"typography-layout",profile:"smoke",repetition:1,warmup:false,equivalence_level:"equivalent",configuration:{},timings_ms:{preflight:null,project_prepare:null,engine_startup:null,frame_production:1,encode:1,render_command:2,artifact_verify:1,end_to_end:3},resources:{peak_rss_bytes:null,peak_temporary_disk_bytes:1,maximum_queued_frame_bytes:null,output_bytes:1},capabilities:{gpu_active:null,gpu_backend:null,gpu_adapter:null,gpu_driver:null,software_fallback:null,capture_mode:null},encoder:{},verification:{passed:true,issues:[]},exit_code:0,timed_out:false,warnings:[]};
  const ajv = new Ajv2020({strict: true});
  addFormats(ajv);
  assert.equal(ajv.validate(schema,result),true,JSON.stringify(ajv.errors));
  assert.equal(ajv.validate(schema,{...result,worktree_clean:false}),false);
});

test("every case declares semantic checkpoints and per-engine equivalence", () => {
  const root = resolve(import.meta.dirname, "../../..");
  const spec = JSON.parse(readFileSync(resolve(root, "benchmarks/specs/phase0-cases.json"), "utf8")) as {cases: Array<{supported_engines: string[]; semantic_checkpoints: unknown[]; equivalence: Record<string,string>}>};
  for (const benchmark of spec.cases) {
    assert.ok(benchmark.semantic_checkpoints.length > 0);
    assert.deepEqual(Object.keys(benchmark.equivalence).sort(), [...benchmark.supported_engines].sort());
  }
});

test("environment manifest and retained v1 result validate against their schemas", () => {
  const root = resolve(import.meta.dirname, "../../..");
  const ajv = new Ajv2020({strict: true});
  addFormats(ajv);
  const environmentSchema = JSON.parse(readFileSync(resolve(root,"schemas/phase0/environment.schema.json"),"utf8"));
  const environment={schema_version:"phase0.environment.v1",environment_id:"a".repeat(64),captured_at_utc:"2026-08-14T00:00:00.000Z",os:"test",architecture:"x86_64",cpu:"test",logical_cores:1,physical_cores:1,ram_bytes:1,gpu:null,tools:{node:"v24",pnpm:"11",cargo:"1",rustc:"1",ffmpeg:"1",ffprobe:"1",git:"1",chrome:"1"},render_environment:{},upstreams:{remotion:{commit:"a".repeat(40),package_version:"4.0.509"},hyperframes:{commit:"b".repeat(40),package_version:"0.7.108"}},cinekernel:{revision:"c".repeat(40),dirty:false}};
  assert.equal(ajv.validate(environmentSchema,environment),true,JSON.stringify(ajv.errors));
  const v1Schema=JSON.parse(readFileSync(resolve(root,"schemas/phase0/benchmark-result-v1.schema.json"),"utf8"));
  const v1={schema_version:"phase0.result.v1",run_id:"legacy",timestamp_utc:"2026-08-14T00:00:00.000Z",cinekernel_revision:"c".repeat(40),cinekernel_dirty:true,environment_id:"a".repeat(64),engine:"native-2d",engine_version:"0.0.0",upstream_commit:null,case_id:"typography-layout",profile:"smoke",repetition:1,configuration:{},timings_ms:{total:1},resources:{},capabilities:{},encoder:{},verification:{passed:true,issues:[]},exit_code:0,timed_out:false,warnings:[]};
  assert.equal(ajv.validate(v1Schema,v1),true,JSON.stringify(ajv.errors));
});

test("fixture manifest hashes every required generated asset", () => {
  const root=resolve(import.meta.dirname,"../../..");
  const fixtureRoot=resolve(root,".cinekernel/generated/fixtures");
  const manifest=JSON.parse(readFileSync(resolve(fixtureRoot,"manifest.json"),"utf8")) as {assets:Array<{name:string;sha256:string}>};
  const paths=new Set(manifest.assets.map((asset)=>asset.name));
  for(const required of ["color-coded-source.mp4","color-coded-oracle.json","clip-a.wav","clip-b.wav","clip-c.wav","invalid-overlap.json","texture.png","font/inter-latin.woff2"]) assert.ok(paths.has(required),required);
  for(const asset of manifest.assets){
    const hash=createHash("sha256").update(readFileSync(resolve(fixtureRoot,asset.name))).digest("hex");
    assert.equal(hash,asset.sha256,asset.name);
  }
});

test("probe driver transforms and reaches its argument guard without top-level-await failure", () => {
  const root=resolve(import.meta.dirname,"../../..");
  const tsx=resolve(import.meta.dirname,"../node_modules/tsx/dist/cli.mjs");
  const execution=spawnSync(process.execPath,[tsx,resolve(root,"benchmarks/probes/run-probes.ts")],{cwd:root,encoding:"utf8"});
  assert.notEqual(execution.status,0);
  assert.match(execution.stderr,/--canonical-run-id is required/);
  assert.doesNotMatch(execution.stderr,/Top-level await/);
});

test("probe driver excludes warmups and scopes GPU tolerance to documented rows", () => {
  const root=resolve(import.meta.dirname,"../../..");
  const source=readFileSync(resolve(root,"benchmarks/probes/run-probes.ts"),"utf8");
  assert.match(source,/basename\(path\)===\"result\.json\"/);
  assert.match(source,/engine===\"remotion\"&&caseId===\"mixed-2d-3d\"/);
  assert.match(source,/minimum_psnr_average_db:35/);
  assert.match(source,/minimum_ssim_all:\.98/);
  assert.match(source,/attempt<=3/);
  assert.match(source,/stderr:\(execution\.stderr\?\?""\)\.slice\(-4000\)/);
  assert.match(source,/engine===\"remotion\"&&caseId===\"3d-scene\"\?20:15/);
  assert.match(source,/excludedProbes\.has\(\"G\"\)/);
  assert.match(source,/UNSUPPORTED by canonical environment/);
});

test("Probe G has a dedicated Linux-only driver and workflow", () => {
  const root=resolve(import.meta.dirname,"../../..");
  const source=readFileSync(resolve(root,"benchmarks/probes/run-network-isolation.ts"),"utf8");
  const workflow=readFileSync(resolve(root,".github/workflows/phase0-network-isolation.yml"),"utf8");
  assert.match(source,/process\.platform!==\"linux\"/);
  assert.match(source,/`PATH=\$\{runnerPath\}`/);
  assert.match(source,/\"unshare\",\"--net\"/);
  assert.match(source,/\[\"remotion\",\"hyperframes\"\]/);
  assert.match(workflow,/runs-on: ubuntu-latest/);
  assert.match(workflow,/network-probe/);
  assert.match(workflow,/CINEKERNEL_BROWSER_EXECUTABLE: \/usr\/bin\/google-chrome/);
  assert.match(workflow,/HYPERFRAMES_BROWSER_PATH: \/usr\/bin\/google-chrome/);
  assert.match(workflow,/\.cinekernel\/logs\//);
});

test("manual evidence workflow is GPU capability-aware and excludes Probe G", () => {
  const root=resolve(import.meta.dirname,"../../..");
  const workflow=readFileSync(resolve(root,".github/workflows/phase0-benchmarks.yml"),"utf8");
  assert.match(workflow,/--capability-only/);
  assert.match(workflow,/--exclude-engine','native-wgpu'/);
  assert.match(workflow,/--max-worker-count/);
  assert.match(workflow,/\[Environment\]::ProcessorCount/);
  assert.match(workflow,/CINEKERNEL_REMOTION_GL = 'angle'/);
  assert.match(workflow,/phase0 probes --canonical --exclude G/);
});
