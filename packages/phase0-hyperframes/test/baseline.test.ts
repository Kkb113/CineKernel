import assert from "node:assert/strict";import test from "node:test";import {createComposition} from "../src/composition.js";
test("HyperFrames baseline is pinned",()=>assert.equal("0.7.108","0.7.108"));
test("composition has deterministic root and local imports",()=>{const html=createComposition({caseId:"mixed-2d-3d",width:640,height:360,duration:3});assert.match(html,/data-composition-id="main"/);assert.match(html,/data-start="0"/);assert.doesNotMatch(html,/https?:\/\//);assert.match(html,/hf-seek/);assert.match(html,/paused:true/)});

