# Distributed rendering

Pinned source: `532caf7aa24fef382cb103013f6414bb547a4129`. Significant claims below use immutable commit links. HyperFrames owns timing in HTML data attributes plus a seekable browser runtime; CineKernel evaluates it as a wrapped web compatibility renderer and a source of protocol/conformance ideas.

| Package / decision | Entrypoint or evidence | Control flow / finding |
|---|---|---|
| aws-lambda | [packages/aws-lambda/src](https://github.com/heygen-com/hyperframes/blob/532caf7aa24fef382cb103013f6414bb547a4129/packages/aws-lambda/src) | AWS execution |
| gcp-cloud-run | [packages/gcp-cloud-run/src](https://github.com/heygen-com/hyperframes/blob/532caf7aa24fef382cb103013f6414bb547a4129/packages/gcp-cloud-run/src) | GCP execution |
| producer | [packages/producer/src/distributed.ts](https://github.com/heygen-com/hyperframes/blob/532caf7aa24fef382cb103013f6414bb547a4129/packages/producer/src/distributed.ts) | distributed contract |

Errors and fallbacks are explicit in engine/producer services; caches require verified anchors; final success still depends on encoded-artifact verification. Recommendation confidence is medium pending full-profile probes.

```mermaid
flowchart LR
A[render plan] --> B[chunks] --> C[AWS]
B --> D[GCP]
C --> E[verified merge]
D --> E
```
