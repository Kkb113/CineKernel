# Preview and export parity

7 comparison rows show that shared authoring evaluation and renderer cores do not establish end-to-end parity.

| Feature | Preview | Export | Classification | Known difference | Certification impact |
|---|---|---|---|---|---|
| React frame evaluation | player-requested frame evaluation | batch renderFrames evaluation | SHARED_WITH_DIFFERENT_SCHEDULING | Preview requests current targets and may coalesce work; export evaluates and retains the complete ordered frame set. | Certify value equivalence separately from scheduling/completeness. |
| CPU renderer core | WASM CPU boundary | native CPU boundary | CONDITIONAL_PARITY | Both consume Scene, but WASM and native use different host, font, filesystem, and media prepass environments. | Require fixture parity with identical fonts and materialized media. |
| GPU renderer core | WASM Vello boundary | native GPU boundary | CONDITIONAL_PARITY | The browser Vello engine is asynchronous WebGPU while native export selects and drives a native renderer path. | Backend name alone is insufficient; compare exact-capability pixels. |
| video decode | browser media element/cache | native media prepass | DIFFERENT_EXECUTION | Preview seeks HTML media and extracts data URIs or overlays; native render fetches to files and decodes before rendering. | Video parity requires source-time, CORS, decode, and overlay exclusions. |
| audio scheduling | AudioContext transport | native audio/encoder path | DIFFERENT_EXECUTION | Preview anchors composition time to AudioContext and schedules ahead; export materializes audio for the encoder timeline. | Audio certification must compare clock mapping and encoded mix, not preview audibility. |
| Canvas2D | approximate preview fallback | not equivalent certification path | KNOWN_APPROXIMATION | Canvas2D implements only a subset and may omit or simplify nodes/effects. | Any Canvas-selected run is excluded from renderer equivalence certification. |
| frame scheduling | RAF may skip requested frames | ordered complete export frame sequence | DIFFERENT_EXECUTION | The Player can skip intermediate targets while catching up; export must emit every requested frame in order. | Temporal certification requires the export sequence, while preview is responsiveness evidence only. |
