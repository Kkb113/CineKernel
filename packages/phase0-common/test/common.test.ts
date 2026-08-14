import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
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

test("source frame markers are deterministic and unique for the first 180 frames", () => {
  const colors = new Set(Array.from({length: 180}, (_, index) => sourceFrameRgb(index).join(",")));
  assert.equal(colors.size, 180);
  assert.deepEqual(sourceFrameRgb(0), [17, 31, 53]);
});

test("benchmark intent validates against its schema", () => {
  const root = resolve(import.meta.dirname, "../../..");
  const schema = JSON.parse(readFileSync(resolve(root, "schemas/phase0/benchmark-intent.schema.json"), "utf8"));
  const spec = JSON.parse(readFileSync(resolve(root, "benchmarks/specs/phase0-cases.json"), "utf8"));
  const ajv = new Ajv2020({strict: true});
  addFormats(ajv);
  assert.equal(ajv.validate(schema, spec), true, JSON.stringify(ajv.errors));
});
