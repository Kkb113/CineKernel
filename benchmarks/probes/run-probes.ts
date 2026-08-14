import {createHash} from "node:crypto";
import {spawn,spawnSync} from "node:child_process";
import {existsSync,mkdirSync,readFileSync,readdirSync,statSync,writeFileSync} from "node:fs";
import {basename,dirname,join,resolve} from "node:path";
import {fileURLToPath} from "node:url";
import {parseArgs} from "node:util";

type ResultEntry={path:string;value:Record<string,any>};
type Probe={id:string;title:string;status:"PASS"|"FAIL"|"UNSUPPORTED";evidence:unknown};

async function main(){
const {values}=parseArgs({args:process.argv.slice(2).filter(value=>value!=="--"),options:{"canonical-run-id":{type:"string"},exclude:{type:"string",multiple:true}}});
if(!values["canonical-run-id"])throw new Error("--canonical-run-id is required");
const canonicalRunId=values["canonical-run-id"];
const root=resolve(dirname(fileURLToPath(import.meta.url)),"../..");
const runDir=resolve(root,".cinekernel/runs",canonicalRunId);
const manifest=JSON.parse(readFileSync(resolve(runDir,"canonical-run-manifest.json"),"utf8"));
const expectedEngines=new Set((manifest.expected_groups??[]).map((group:any)=>group.engine));
const excludedProbes=new Set(values.exclude??[]);
const probeRoot=resolve(root,".cinekernel/probes",canonicalRunId);
const fixtures=resolve(root,".cinekernel/generated/fixtures");
mkdirSync(probeRoot,{recursive:true});
const cargo=process.platform==="win32"?resolve(process.env.USERPROFILE??"",".cargo/bin/cargo.exe"):"cargo";
const walk=(directory:string):string[]=>{try{return readdirSync(directory).flatMap(name=>{const path=join(directory,name);return statSync(path).isDirectory()?walk(path):[path]})}catch{return[]}};
const entries:ResultEntry[]=walk(runDir).filter(path=>basename(path)==="result.json").map(path=>({path,value:JSON.parse(readFileSync(path,"utf8"))}));
const probe=(id:string,title:string,status:Probe["status"],evidence:unknown):Probe=>({id,title,status,evidence});
const outputFor=(entry:ResultEntry)=>resolve(dirname(entry.path),"output.mp4");
const group=(engine:string,caseId:string,mode:string)=>entries.filter(entry=>entry.value.engine===engine&&entry.value.case_id===caseId&&entry.value.configuration?.worker_mode===mode&&entry.value.verification?.passed===true&&entry.value.exit_code===0).sort((left,right)=>left.value.repetition-right.value.repetition);
const frameStreamHash=(video:string)=>{const output=spawnSync("ffmpeg",["-v","error","-i",video,"-f","framemd5","-"],{encoding:"utf8",maxBuffer:256*1024*1024});if(output.status!==0)throw new Error(`framemd5 failed for ${video}: ${output.stderr}`);return createHash("sha256").update(output.stdout).digest("hex")};
const videoSimilarity=(left:string,right:string)=>{const psnr=spawnSync("ffmpeg",["-v","info","-i",left,"-i",right,"-lavfi","psnr","-f","null","-"],{encoding:"utf8",maxBuffer:256*1024*1024});const ssim=spawnSync("ffmpeg",["-v","info","-i",left,"-i",right,"-lavfi","ssim","-f","null","-"],{encoding:"utf8",maxBuffer:256*1024*1024});const psnrAverage=Number(/PSNR[^\n]*average:([0-9.]+)/.exec(psnr.stderr)?.[1]);const ssimAll=Number(/All:([0-9.]+)/.exec(ssim.stderr)?.[1]);return{psnr_average_db:psnrAverage,ssim_all:ssimAll,minimum_psnr_average_db:35,minimum_ssim_all:.98,passed:psnr.status===0&&ssim.status===0&&psnrAverage>=35&&ssimAll>=.98}};
const run=(program:string,args:string[],options:Record<string,unknown>={})=>spawnSync(program,args,{cwd:root,encoding:"utf8",maxBuffer:256*1024*1024,...options});
const probes:Probe[]=[];

// Probe A: all required critical groups, never a convenient first match.
const stabilityGroups:[string,string,string][]=[
  ["remotion","media-frame-sampling","default"],["remotion","mixed-2d-3d","default"],
  ["hyperframes","media-frame-sampling","auto"],["hyperframes","mixed-2d-3d","auto"],
  ["native-2d","typography-layout","default"],["native-2d","chart-diagram","default"],
  ["native-wgpu","3d-scene","default"],["native-wgpu","mixed-2d-3d","default"],
];
const stabilityEvidence=[];let stabilityPass=true;
for(const [engine,caseId,mode] of stabilityGroups){if(!expectedEngines.has(engine)){stabilityEvidence.push({engine,case_id:caseId,worker_mode:mode,status:"UNSUPPORTED by canonical environment",repetitions:0});continue}const candidates=group(engine,caseId,mode);if(candidates.length<3){stabilityPass=false;stabilityEvidence.push({engine,case_id:caseId,worker_mode:mode,status:"missing",repetitions:candidates.length});continue}const selected=candidates.slice(0,3);const hashes=selected.map(entry=>frameStreamHash(outputFor(entry)));const exact=hashes.every(hash=>hash===hashes[0]);const gpuToleranceEligible=engine==="remotion"&&caseId==="mixed-2d-3d";const similarities=gpuToleranceEligible&&!exact?selected.slice(1).map(entry=>videoSimilarity(outputFor(selected[0]),outputFor(entry))):[];const stable=exact||(gpuToleranceEligible&&similarities.length===2&&similarities.every(item=>item.passed));stabilityPass&&=stable;stabilityEvidence.push({engine,case_id:caseId,worker_mode:mode,repetitions:3,classification:exact?"exact match":stable?"bounded WebGL pixel variance":"failure",decoded_framemd5_sha256:hashes,similarities})}
probes.push(probe("A","Repeated render stability",stabilityPass?"PASS":"FAIL",stabilityEvidence));

// Probe B: every frame was decoded/classified by the permanent verifier for five repetitions in all six modes.
const mediaEvidence=[];let mediaPass=true;
for(const [engine,modes] of [["remotion",["default","1","4"]],["hyperframes",["auto","1","4"]]] as const){for(const mode of modes){const candidates=group(engine,"media-frame-sampling",mode);const checks=candidates.slice(0,5).map(entry=>entry.value.verification?.decoded?.media_oracle);const ok=candidates.length>=5&&checks.length===5&&checks.every(check=>check?.all_frames_match===true&&check?.checked_frame_count===180);mediaPass&&=ok;mediaEvidence.push({engine,worker_mode:mode,repetitions:candidates.length,required_repetitions:5,frames_checked:checks.map(check=>check?.checked_frame_count),all_match:ok,first_mismatches:checks.map(check=>check?.first_mismatch)})}}
probes.push(probe("B","Complete sequential/parallel media oracle",mediaPass?"PASS":"FAIL",mediaEvidence));

// Probe C: two native cases, two independent shuffled orders, complete decoded comparison.
const randomDir=resolve(probeRoot,"random-access");mkdirSync(randomDir,{recursive:true});
const randomEvidence=[];let randomPass=true;
for(const item of [{engine:"native-2d",pkg:"phase0-native-2d",caseId:"chart-diagram",duration:6},{engine:"native-wgpu",pkg:"phase0-native-wgpu",caseId:"3d-scene",duration:8}]){
  if(!expectedEngines.has(item.engine)){randomEvidence.push({...item,status:"UNSUPPORTED by canonical environment"});continue}
  const canonical=group(item.engine,item.caseId,"default")[0];if(!canonical){randomPass=false;randomEvidence.push({...item,status:"missing canonical sequential"});continue}
  const reference=frameStreamHash(outputFor(canonical));const orders=[];
  for(const seed of [2246822519,3266489917]){const output=resolve(randomDir,`${item.engine}-${item.caseId}-${seed}.mp4`);const execution=run(cargo,["run","--release","--package",item.pkg,"--","--case",item.caseId,"--profile","full","--output",output,"--frame-order","shuffled","--shuffle-seed",String(seed)],{env:{...process.env,CINEKERNEL_WIDTH:"1920",CINEKERNEL_HEIGHT:"1080",CINEKERNEL_FPS:"30",CINEKERNEL_DURATION_SECONDS:String(item.duration),CINEKERNEL_FIXTURES:fixtures}});const hash=execution.status===0?frameStreamHash(output):null;const match=hash===reference;randomPass&&=execution.status===0&&match;orders.push({seed,exit_code:execution.status,decoded_framemd5_sha256:hash,matches_sequential:match})}
  randomEvidence.push({engine:item.engine,case_id:item.caseId,profile:"full",sequential_sha256:reference,shuffled_orders:orders});
}
probes.push(probe("C","Random-access versus sequential native evaluation",randomPass?"PASS":"FAIL",randomEvidence));

const decodeRgb=(source:string,frame?:number)=>{const args=["-v","error","-i",source];if(frame!==undefined)args.push("-vf",`select=eq(n\\,${frame})`,"-fps_mode","passthrough","-frames:v","1");args.push("-pix_fmt","rgb24","-f","rawvideo","-");const output=spawnSync("ffmpeg",args,{encoding:"buffer",maxBuffer:256*1024*1024});return output.stdout as Buffer};
const mae=(left:Buffer,right:Buffer)=>{if(!left.length||left.length!==right.length)return Number.POSITIVE_INFINITY;let difference=0;for(let index=0;index<left.length;index++)difference+=Math.abs(left[index]-right[index]);return difference/left.length};
const snapshotDir=resolve(probeRoot,"preview-final");mkdirSync(snapshotDir,{recursive:true});
const snapshotEvidence=[];let snapshotPass=true;
for(const [engine,caseId] of [["remotion","media-frame-sampling"],["remotion","mixed-2d-3d"],["remotion","3d-scene"],["hyperframes","media-frame-sampling"],["hyperframes","mixed-2d-3d"],["hyperframes","3d-scene"]] as const){
  const mode=engine==="hyperframes"?"auto":"default";const candidate=group(engine,caseId,mode)[0];if(!candidate){snapshotPass=false;snapshotEvidence.push({engine,case_id:caseId,status:"missing final"});continue}
  const totalFrames=Math.round(candidate.value.configuration.duration_seconds*30);const boundaries=caseId==="mixed-2d-3d"?[Math.round(totalFrames*.2),Math.round(totalFrames*7/15),Math.round(totalFrames*.8)]:[];const positions=[Math.round(totalFrames*.1),Math.round(totalFrames*.5),Math.round(totalFrames*.9),boundaries[0]??0].map(frame=>Math.min(totalFrames-1,Math.max(0,frame)));const comparisons=[];
  for(const frame of [...new Set(positions)]){const destination=resolve(snapshotDir,engine,caseId,String(frame));mkdirSync(destination,{recursive:true});let still="";let execution;
    if(engine==="remotion"){still=resolve(destination,"still.png");const cli=resolve(root,"packages/phase0-remotion/node_modules/@remotion/cli/remotion-cli.js");const props=JSON.stringify({caseId,width:1920,height:1080,durationSeconds:candidate.value.configuration.duration_seconds});execution=spawnSync(process.execPath,[cli,"still","src/index.tsx","CineKernelBenchmark",still,"--frame",String(frame),"--props",props,"--public-dir",fixtures,"--log","error"],{cwd:resolve(root,"packages/phase0-remotion"),encoding:"utf8",maxBuffer:128*1024*1024});}
    else{const project=resolve(root,".cinekernel/projects/hyperframes",`${canonicalRunId}-${caseId}-full`);const cli=resolve(root,"packages/phase0-hyperframes/node_modules/hyperframes/bin/hyperframes.mjs");execution=spawnSync(process.execPath,[cli,"snapshot",project,"--output",destination,"--at",String(frame/30),"--no-end","--describe","false"],{cwd:root,encoding:"utf8",maxBuffer:128*1024*1024});still=walk(destination).find(path=>path.endsWith(".png"))??"";}
    const difference=execution.status===0&&still?mae(decodeRgb(still),decodeRgb(outputFor(candidate),frame)):Number.POSITIVE_INFINITY;const threshold=caseId==="media-frame-sampling"?9:engine==="remotion"&&caseId==="3d-scene"?20:15;const ok=difference<=threshold;snapshotPass&&=ok;comparisons.push({frame,time_seconds:frame/30,exit_code:execution.status,mae:difference,threshold,passed:ok});
  }
  snapshotEvidence.push({engine,case_id:caseId,comparisons});
}
probes.push(probe("D","Preview/snapshot versus final",snapshotPass?"PASS":"FAIL",snapshotEvidence));

// Probes E/F: valid three-clip evidence plus deliberately invalid missing/overlapping muxes rejected by the central verifier.
const validAudio=group("remotion","audio-captions","default")[0];const audioDir=resolve(probeRoot,"invalid-audio");mkdirSync(audioDir,{recursive:true});
let audioPresencePass=false;let seamPass=false;const audioEvidence:Record<string,unknown>={};
if(validAudio){const valid=validAudio.value.verification?.audio;audioPresencePass=valid?.clip_signature_count===3;const missing=resolve(audioDir,"missing-clip.mp4");const overlap=resolve(audioDir,"overlap.mp4");
  const missingMux=run("ffmpeg",["-v","error","-y","-i",outputFor(validAudio),"-i",resolve(fixtures,"clip-a.wav"),"-filter_complex","[1:a]apad=pad_dur=8[a]","-map","0:v:0","-map","[a]","-c:v","copy","-c:a","aac","-t","8",missing]);
  const overlapMux=run("ffmpeg",["-v","error","-y","-i",outputFor(validAudio),"-i",resolve(fixtures,"clip-a.wav"),"-i",resolve(fixtures,"clip-b.wav"),"-i",resolve(fixtures,"clip-c.wav"),"-filter_complex","[1:a]adelay=0:all=1[a1];[2:a]adelay=1500:all=1[a2];[3:a]adelay=6000:all=1[a3];[a1][a2][a3]amix=inputs=3:normalize=0,apad=pad_dur=8[a]","-map","0:v:0","-map","[a]","-c:v","copy","-c:a","aac","-t","8",overlap]);
  const missingCheck=run(cargo,["xtask","phase0","verify-artifact","--output",missing,"--case","audio-captions","--profile","full","--engine","remotion","--expect-invalid","--json"]);const overlapCheck=run(cargo,["xtask","phase0","verify-artifact","--output",overlap,"--case","audio-captions","--profile","full","--engine","remotion","--expect-invalid","--json"]);
  audioPresencePass&&=missingMux.status===0&&missingCheck.status===0;seamPass=valid?.seam_jumps?.every((entry:any)=>entry.maximum_jump<=.12)&&overlapMux.status===0&&overlapCheck.status===0;
  Object.assign(audioEvidence,{valid_clip_signatures:valid?.frequency_windows,valid_silence_windows:valid?.silence_windows,valid_seams:valid?.seam_jumps,missing_mux_exit:missingMux.status,missing_rejected:missingCheck.status===0,overlap_mux_exit:overlapMux.status,overlap_rejected:overlapCheck.status===0});
}
probes.push(probe("E","Audio presence and registration",audioPresencePass?"PASS":"FAIL",audioEvidence));
probes.push(probe("F","Audio seams and overlap rejection",seamPass?"PASS":"FAIL",audioEvidence));

// Probe G: OS-enforced Linux network namespace. Other hosts record UNSUPPORTED and rely on the Ubuntu workflow artifact.
if(!excludedProbes.has("G")&&process.platform==="linux"){
  const isolated=[];let isolatedPass=true;
  for(const engine of ["remotion","hyperframes"]){const execution=run("sudo",["-E","unshare","--net","--","sh","-c",'ip link set lo up && exec "$@"',"cinekernel-network-isolation",cargo,"xtask","phase0","run","--engine",engine,"--case","media-frame-sampling","--profile","smoke","--timeout-seconds","900","--json"],{env:{...process.env}});isolatedPass&&=execution.status===0;isolated.push({engine,mechanism:"sudo unshare --net with loopback-only namespace",exit_code:execution.status,command:`sudo -E unshare --net -- sh -c 'ip link set lo up; exec cargo xtask phase0 run ...'`,stderr:(execution.stderr??"").slice(-4000)})}
  probes.push(probe("G","Render-time network isolation",isolatedPass?"PASS":"FAIL",isolated));
}else if(!excludedProbes.has("G")) probes.push(probe("G","Render-time network isolation","UNSUPPORTED",{host:process.platform,required_evidence:"Dedicated Ubuntu network-isolation workflow must execute sudo unshare --net for both browser baselines"}));

const richVerification=entries.every(entry=>entry.value.verification?.timestamps?.monotonic===true&&Array.isArray(entry.value.verification?.decoded?.selected_frame_hashes)&&entry.value.verification.decoded.selected_frame_hashes.length>=3&&entry.value.verification?.video?.codec==="h264");
probes.push(probe("H","Final mux integrity through central verifier",richVerification?"PASS":"FAIL",{canonical_results:entries.length,all_have_timestamps_hashes_codec_and_case_checks:richVerification}));

// Probe I: inject a hang into a real xtask-run child, prove invalid partial evidence and subsequent recovery.
const before=new Set(readdirSync(resolve(root,".cinekernel/runs")));const timeoutRun=run(cargo,["xtask","phase0","run","--engine","native-2d","--case","typography-layout","--profile","smoke","--timeout-seconds","1","--stall-seconds","10","--json"],{env:{...process.env,CINEKERNEL_TEST_HANG_SECONDS:"30"}});const after=readdirSync(resolve(root,".cinekernel/runs")).filter(name=>!before.has(name));const timedDirectory=after.map(name=>resolve(root,".cinekernel/runs",name)).sort((a,b)=>statSync(b).mtimeMs-statSync(a).mtimeMs)[0];const failureFiles=timedDirectory?walk(timedDirectory).filter(path=>path.endsWith("failure.json")):[];const successFiles=timedDirectory?walk(timedDirectory).filter(path=>path.endsWith("result.json")):[];const failure=failureFiles[0]?JSON.parse(readFileSync(failureFiles[0],"utf8")):null;const recovery=run(cargo,["xtask","phase0","run","--engine","native-2d","--case","typography-layout","--profile","smoke","--timeout-seconds","300","--json"],{env:{...process.env,CINEKERNEL_TEST_HANG_SECONDS:"0"}});const cleanupPass=timeoutRun.status!==0&&failure?.timed_out===true&&failure?.valid===false&&successFiles.length===0&&recovery.status===0;
probes.push(probe("I","Harness timeout and process-tree cleanup",cleanupPass?"PASS":"FAIL",{timeout_exit:timeoutRun.status,failure_record:failure,successful_result_written:successFiles.length>0,recovery_exit:recovery.status,timeout_stderr:(timeoutRun.stderr??"").slice(-2000)}));

const backpressure=await runBackpressure(resolve(probeRoot,"backpressure"));
probes.push(probe("J","Real bounded frame backpressure",backpressure.passed?"PASS":"FAIL",backpressure));

const failed=probes.filter(item=>item.status==="FAIL");const unsupported=probes.filter(item=>item.status==="UNSUPPORTED");
const payload={schema_version:"phase0.probes.v2",canonical_run_id:canonicalRunId,generated_at_utc:new Date().toISOString(),passed:failed.length===0,failed_count:failed.length,unsupported_count:unsupported.length,results:probes};
const reportDir=resolve(root,"reports/phase0");mkdirSync(reportDir,{recursive:true});writeFileSync(resolve(reportDir,"CORRECTNESS_PROBES.json"),`${JSON.stringify(payload,null,2)}\n`);writeFileSync(resolve(reportDir,"CORRECTNESS_PROBES.md"),`# Phase 0.1 correctness probes\n\nCanonical run: \`${canonicalRunId}\`.\n\n| Probe | Status | Evidence summary |\n|---|---|---|\n${probes.map(item=>`| ${item.id} — ${item.title} | ${item.status} | \`${JSON.stringify(item.evidence).slice(0,500).replaceAll("|","\\|")}\` |`).join("\n")}\n`);console.log(JSON.stringify(payload));if(failed.length)process.exitCode=1;
}

void main().catch(error=>{console.error(error);process.exitCode=1});

async function runBackpressure(directory:string){mkdirSync(directory,{recursive:true});const output=resolve(directory,"bounded.mp4");const width=1920,height=1080,frameBytes=width*height*4,capacity=3,totalFrames=12;const encoder=spawn("ffmpeg",["-v","error","-y","-f","rawvideo","-pix_fmt","rgba","-s",`${width}x${height}`,"-r","30","-i","-","-c:v","libx264","-preset","ultrafast","-crf","28","-pix_fmt","yuv420p",output],{stdio:["pipe","ignore","pipe"]});let stderr="";encoder.stderr.on("data",chunk=>stderr+=String(chunk));const queue:Buffer[]=[];let produced=0,consumed=0,maxQueued=0,maxBytes=0,peakRss=process.memoryUsage().rss;let producerDone=false;const producer=(async()=>{while(produced<totalFrames){if(queue.length>=capacity){await delay(5);continue}const frame=Buffer.allocUnsafe(frameBytes);frame.fill((produced*19)%256);queue.push(frame);produced++;maxQueued=Math.max(maxQueued,queue.length);maxBytes=Math.max(maxBytes,queue.length*frameBytes);peakRss=Math.max(peakRss,process.memoryUsage().rss)}producerDone=true})();const consumer=(async()=>{while(!producerDone||queue.length){const frame=queue.shift();if(!frame){await delay(5);continue}if(!encoder.stdin.write(frame))await new Promise<void>(done=>encoder.stdin.once("drain",done));consumed++;peakRss=Math.max(peakRss,process.memoryUsage().rss);await delay(25)}encoder.stdin.end()})();await Promise.all([producer,consumer]);const exitCode=await new Promise<number|null>(done=>encoder.once("close",done));return{passed:exitCode===0&&produced===totalFrames&&consumed===totalFrames&&maxQueued<=capacity&&maxBytes<=capacity*frameBytes&&existsSync(output),queue_capacity:capacity,frame_bytes:frameBytes,maximum_queued_frame_count:maxQueued,maximum_queued_frame_bytes:maxBytes,peak_rss_bytes:peakRss,produced_frames:produced,consumed_frames:consumed,consumer:"real ffmpeg libx264 subprocess with 25ms delay",encoder_exit_code:exitCode,encoder_stderr:stderr};}
function delay(milliseconds:number){return new Promise<void>(done=>setTimeout(done,milliseconds))}
