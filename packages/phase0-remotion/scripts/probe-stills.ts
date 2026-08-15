import {bundle} from "@remotion/bundler";
import {openBrowser,renderStill,selectComposition,type ChromiumOptions,type HeadlessBrowser} from "@remotion/renderer";
import {readFileSync} from "node:fs";
import {resolve} from "node:path";
import {parseArgs} from "node:util";

type StillTask={case_id:string;duration_seconds:number;frame:number;output:string};
type Attempt={attempt:number;exit_code:number;stderr:string};

const {values}=parseArgs({args:process.argv.slice(2).filter(value=>value!=="--"),options:{"tasks-file":{type:"string"},fixtures:{type:"string"},"bundle-dir":{type:"string"}}});
if(!values["tasks-file"]||!values.fixtures||!values["bundle-dir"])throw new Error("--tasks-file, --fixtures, and --bundle-dir are required");
const tasks=JSON.parse(readFileSync(values["tasks-file"],"utf8")) as StillTask[];
const packageRoot=resolve(import.meta.dirname,"..");
const gl=process.env.CINEKERNEL_REMOTION_GL as ChromiumOptions["gl"];
const chromiumOptions:ChromiumOptions=gl?{gl}:{};
const browserExecutable=process.env.CINEKERNEL_BROWSER_EXECUTABLE;
const browserOptions={chromiumOptions,logLevel:"error" as const,...(browserExecutable?{browserExecutable}:{})};
const serveUrl=await bundle({entryPoint:resolve(packageRoot,"src/index.tsx"),publicDir:values.fixtures,rootDir:packageRoot,outDir:values["bundle-dir"],onProgress:()=>{}});
let browser:HeadlessBrowser|undefined;
const results=[];

const closeBrowser=async()=>{if(!browser)return;try{await browser.close({silent:true})}finally{browser=undefined}};
try{
  for(const task of tasks){
    const attempts:Attempt[]=[];let exitCode=1;
    for(let attempt=1;attempt<=3;attempt++){
      try{
        browser??=await openBrowser("chrome",browserOptions);
        const inputProps={caseId:task.case_id,width:1920,height:1080,durationSeconds:task.duration_seconds};
        const composition=await selectComposition({serveUrl,id:"CineKernelBenchmark",inputProps,puppeteerInstance:browser,chromiumOptions,logLevel:"error",timeoutInMilliseconds:120000,...(browserExecutable?{browserExecutable}:{})});
        await renderStill({serveUrl,composition,output:task.output,frame:task.frame,inputProps,puppeteerInstance:browser,chromiumOptions,logLevel:"error",timeoutInMilliseconds:120000,...(browserExecutable?{browserExecutable}:{})});
        attempts.push({attempt,exit_code:0,stderr:""});exitCode=0;break;
      }catch(error){
        attempts.push({attempt,exit_code:1,stderr:String(error instanceof Error?error.stack??error.message:error).slice(-4000)});
        await closeBrowser();
        if(attempt<3)await new Promise(done=>setTimeout(done,2000));
      }
    }
    results.push({...task,exit_code:exitCode,attempts});
  }
}finally{await closeBrowser()}

const payload={ok:results.every(result=>result.exit_code===0),browser_reuse:true,graphics_backend:gl??null,results};
console.log(JSON.stringify(payload));
if(!payload.ok)process.exitCode=1;
