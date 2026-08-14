import assert from "node:assert/strict";
import {existsSync,readFileSync,readdirSync,statSync} from "node:fs";
import {dirname,resolve} from "node:path";

const root=resolve(import.meta.dirname,"../../..");
const lock=JSON.parse(readFileSync(resolve(root,"benchmarks/upstreams.lock.json"),"utf8")) as {remotion:{commit:string};hyperframes:{commit:string}};
const researchRoot=resolve(root,"docs/research");
const walk=(directory:string):string[]=>readdirSync(directory).flatMap(name=>{const path=resolve(directory,name);return statSync(path).isDirectory()?walk(path):[path]});
const documents=walk(researchRoot).filter(path=>path.endsWith(".md"));
const failures:string[]=[];
for(const document of documents){
  const text=readFileSync(document,"utf8");
  for(const match of text.matchAll(/https:\/\/github\.com\/(remotion-dev\/remotion|heygen-com\/hyperframes)\/blob\/([0-9a-f]{40})\//g)){
    const expected=match[1]!.startsWith("remotion-dev")?lock.remotion.commit:lock.hyperframes.commit;
    if(match[2]!==expected)failures.push(`${document}: immutable link uses ${match[2]}, expected ${expected}`);
  }
  for(const match of text.matchAll(/\[[^\]]+\]\((?!https?:|#)([^)]+\.md)(?:#[^)]+)?\)/g)){
    const target=resolve(dirname(document),match[1]!);
    if(!existsSync(target))failures.push(`${document}: local document link does not exist: ${match[1]}`);
  }
}
const inventoryPath=resolve(root,"docs/source-lineage/upstream-inventory.yaml");
const inventory=readFileSync(inventoryPath,"utf8");
const blocks=inventory.split(/\n  - /).slice(1);
const required=["local_path","classification","permission_basis","original_license","local_contracts_added","conformance_tests","upstream_sync_policy","owner"];
for(const block of blocks){
  for(const key of required)if(!new RegExp(`(?:^|\\n)    ${key}:`).test(`\n    ${block}`))failures.push(`${inventoryPath}: entry missing ${key}`);
  const local=/local_path:\s*([^\n]+)/.exec(block)?.[1]?.trim();
  if(local&&!existsSync(resolve(root,local)))failures.push(`${inventoryPath}: local_path does not exist: ${local}`);
}
assert.equal(failures.length,0,failures.join("\n"));
console.log(JSON.stringify({ok:true,documents_checked:documents.length,lineage_entries_checked:blocks.length,remotion_commit:lock.remotion.commit,hyperframes_commit:lock.hyperframes.commit}));
