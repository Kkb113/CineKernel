import {spawnSync} from "node:child_process";
import {mkdirSync,writeFileSync} from "node:fs";
import {dirname,resolve} from "node:path";
import {fileURLToPath} from "node:url";

type IsolationResult={
  engine:string;
  mechanism:string;
  command:string;
  exit_code:number|null;
  passed:boolean;
  stdout_tail:string;
  stderr_tail:string;
};

const root=resolve(dirname(fileURLToPath(import.meta.url)),"../..");
const outputRoot=resolve(root,".cinekernel/probes/network-isolation");
const reportRoot=resolve(root,"reports/phase0");
mkdirSync(outputRoot,{recursive:true});
mkdirSync(reportRoot,{recursive:true});

if(process.platform!=="linux")throw new Error("Probe G requires Linux");
const cargoLookup=spawnSync("sh",["-c","command -v cargo"],{cwd:root,encoding:"utf8"});
const cargo=(process.env.CARGO??cargoLookup.stdout.trim())||"cargo";
const runnerPath=process.env.PATH;
if(!runnerPath)throw new Error("Probe G requires the runner PATH");
const results:IsolationResult[]=[];

for(const engine of ["remotion","hyperframes"]){
  const command=["-E","env",`PATH=${runnerPath}`,"unshare","--net","--","sh","-c",'ip link set lo up && exec "$@"',"cinekernel-network-isolation",cargo,"xtask","phase0","run","--engine",engine,"--case","media-frame-sampling","--profile","smoke","--timeout-seconds","900","--json"];
  const execution=spawnSync("sudo",command,{cwd:root,encoding:"utf8",maxBuffer:256*1024*1024,env:{...process.env}});
  results.push({
    engine,
    mechanism:"sudo unshare --net with loopback-only namespace",
    command:`sudo -E env PATH=<runner-path> unshare --net -- sh -c 'ip link set lo up; exec ${cargo} xtask phase0 run --engine ${engine} ...'`,
    exit_code:execution.status,
    passed:execution.status===0,
    stdout_tail:(execution.stdout??"").slice(-8000),
    stderr_tail:(execution.stderr??"").slice(-8000),
  });
}

const passed=results.every(result=>result.passed);
const payload={
  schema_version:"phase0.network-isolation.v1",
  generated_at_utc:new Date().toISOString(),
  host:process.platform,
  mechanism:"Linux network namespace with loopback only",
  unexpected_external_network_available:false,
  passed,
  results,
};
writeFileSync(resolve(outputRoot,"probe-g.json"),`${JSON.stringify(payload,null,2)}\n`);
writeFileSync(resolve(reportRoot,"NETWORK_ISOLATION_PROBE.json"),`${JSON.stringify(payload,null,2)}\n`);
writeFileSync(resolve(reportRoot,"NETWORK_ISOLATION_PROBE.md"),`# Phase 0.1 Probe G — render-time network isolation\n\nStatus: **${passed?"PASS":"FAIL"}**\n\nMechanism: Linux network namespace with loopback only.\n\n| Engine | Exit | Status |\n|---|---:|---|\n${results.map(result=>`| ${result.engine} | ${result.exit_code} | ${result.passed?"PASS":"FAIL"} |`).join("\n")}\n`);
console.log(JSON.stringify(payload));
if(!passed)process.exitCode=1;
