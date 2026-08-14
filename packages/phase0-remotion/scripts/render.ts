import {spawnSync} from "node:child_process";
import {unlinkSync} from "node:fs";
import {createRequire} from "node:module";
import {dirname,resolve} from "node:path";
import {parseArgs} from "node:util";

const {values}=parseArgs({args:process.argv.slice(2).filter((value)=>value!=="--"),options:{case:{type:"string"},profile:{type:"string"},output:{type:"string"}}});
if(!values.case||!values.profile||!values.output) throw new Error("--case, --profile, and --output are required");
const width=Number(process.env.CINEKERNEL_WIDTH??(values.profile==="smoke"?640:1920));
const height=Number(process.env.CINEKERNEL_HEIGHT??(values.profile==="smoke"?360:1080));
const durationSeconds=Number(process.env.CINEKERNEL_DURATION_SECONDS??1);
const fixtures=process.env.CINEKERNEL_FIXTURES;
if(!fixtures) throw new Error("CINEKERNEL_FIXTURES is required");
const concurrency=process.env.CINEKERNEL_CONCURRENCY;
const hasAudio=values.case==="audio-captions"||values.case==="mixed-2d-3d";
const renderOutput=hasAudio?`${values.output}.intermediate.mp4`:values.output;
const args=["render","src/index.tsx","CineKernelBenchmark",renderOutput,"--props",JSON.stringify({caseId:values.case,width,height,durationSeconds}),"--codec","h264","--pixel-format","yuv420p","--image-format","png","--color-space","bt709","--crf","18","--x264-preset","medium","--public-dir",fixtures,"--timeout","120000","--log","verbose"];
const browserExecutable=process.env.CINEKERNEL_BROWSER_EXECUTABLE;
if(browserExecutable)args.push("--browser-executable",browserExecutable);
if(hasAudio)args.push("--audio-bitrate","192k");else args.push("--muted");
if(concurrency) args.push("--concurrency",concurrency);
const started=performance.now();
const require=createRequire(import.meta.url);const cli=resolve(dirname(require.resolve("@remotion/cli")),"../remotion-cli.js");
const result=spawnSync(process.execPath,[cli,...args],{cwd:new URL("..",import.meta.url),stdio:"inherit",shell:false});
if(result.error) throw result.error;
if(result.status!==0) process.exit(result.status??1);
if(hasAudio){
  const mux=spawnSync("ffmpeg",["-v","error","-i",renderOutput,"-filter_complex",`[0:a:0]apad,atrim=duration=${durationSeconds}[a]`,"-map","0:v:0","-map","[a]","-c:v","copy","-c:a","aac","-b:a","192k","-ar","48000","-ac","1","-t",String(durationSeconds),"-movflags","+faststart","-y",values.output],{stdio:"inherit",shell:false});
  if(mux.error) throw mux.error;
  if(mux.status!==0) process.exit(mux.status??1);
  unlinkSync(renderOutput);
}
const renderCommandMs=performance.now()-started;
console.log(JSON.stringify({ok:true,engine:"remotion",case:values.case,profile:values.profile,concurrency:concurrency??"default",timings_ms:{preflight:null,project_prepare:null,engine_startup:null,frame_production:null,encode:null,render_command:renderCommandMs},encoder:{container:"mp4",video_codec:"h264",encoder:"libx264",pixel_format:"yuv420p",color_space:"bt709",intermediate_image_format:"png",crf:18,preset:"medium",audio_codec:hasAudio?"aac":null,audio_bitrate:hasAudio?"192k":null,sample_rate:hasAudio?48000:null,channel_layout:hasAudio?"mono":null,limitations:["Remotion CLI does not expose renderer-internal frame production and encode stage durations separately"]}}));
