import {spawnSync} from "node:child_process";
import {createRequire} from "node:module";
import {cpSync,copyFileSync,mkdirSync,writeFileSync} from "node:fs";
import {dirname,resolve} from "node:path";
import {fileURLToPath} from "node:url";
import {parseArgs} from "node:util";
import {createComposition} from "../src/composition.js";

const {values}=parseArgs({args:process.argv.slice(2).filter((value)=>value!=="--"),options:{case:{type:"string"},profile:{type:"string"},output:{type:"string"},capture:{type:"string"}}});
if(!values.case||!values.profile||!values.output) throw new Error("--case, --profile, and --output are required");
const width=Number(process.env.CINEKERNEL_WIDTH??(values.profile==="smoke"?640:1920));const height=Number(process.env.CINEKERNEL_HEIGHT??(values.profile==="smoke"?360:1080));const duration=Number(process.env.CINEKERNEL_DURATION_SECONDS??1);const fixtures=process.env.CINEKERNEL_FIXTURES;if(!fixtures)throw new Error("CINEKERNEL_FIXTURES is required");
const here=dirname(fileURLToPath(import.meta.url));const root=resolve(here,"../../..");const runId=process.env.CINEKERNEL_RUN_ID??"manual";const project=resolve(root,".cinekernel/projects/hyperframes",`${runId}-${values.case}-${values.profile}`);mkdirSync(resolve(project,"vendor"),{recursive:true});cpSync(fixtures,resolve(project,"assets"),{recursive:true,force:true});writeFileSync(resolve(project,"index.html"),createComposition({caseId:values.case,width,height,duration}));writeFileSync(resolve(project,"hyperframes.json"),JSON.stringify({version:1,compositions:["index.html"]},null,2));
const require=createRequire(import.meta.url);const threeBuild=dirname(require.resolve("three"));copyFileSync(require.resolve("gsap/dist/gsap.min.js"),resolve(project,"vendor/gsap.min.js"));copyFileSync(resolve(threeBuild,"three.module.js"),resolve(project,"vendor/three.module.js"));copyFileSync(resolve(threeBuild,"three.core.js"),resolve(project,"vendor/three.core.js"));
const hyperframesCli=resolve(here,"../node_modules/hyperframes/bin/hyperframes.mjs");
const run=(args:string[])=>{const result=spawnSync(process.execPath,[hyperframesCli,...args],{cwd:root,stdio:"inherit",shell:false});if(result.error)throw result.error;if(result.status!==0)process.exit(result.status??1)};
run(["lint",project]);run(["check",project,"--strict"]);const renderArgs=["render",project,"--output",values.output,"--fps","30","--quality",values.profile==="smoke"?"draft":"high","--strict","--sdr"];const workers=process.env.CINEKERNEL_WORKERS;if(workers)renderArgs.push("--workers",workers);if(values.capture==="screenshot")process.env.HYPERFRAMES_CAPTURE_MODE="screenshot";run(renderArgs);console.log(JSON.stringify({ok:true,engine:"hyperframes",case:values.case,profile:values.profile,workers:workers??"auto",capture_requested:values.capture??"auto",project:project.replace(root,".")}));
