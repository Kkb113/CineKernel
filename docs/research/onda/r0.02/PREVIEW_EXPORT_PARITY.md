# Preview and export parity

7 comparison rows show that shared authoring evaluation and renderer cores do not establish end-to-end parity.

| Feature | Preview | Export | Classification |
|---|---|---|---|
| React frame evaluation | player-requested frame evaluation | batch renderFrames evaluation | SHARED_WITH_DIFFERENT_SCHEDULING |
| CPU renderer core | WASM CPU boundary | native CPU boundary | CONDITIONAL_PARITY |
| GPU renderer core | WASM Vello boundary | native GPU boundary | CONDITIONAL_PARITY |
| video decode | browser media element/cache | native media prepass | DIFFERENT_EXECUTION |
| audio scheduling | AudioContext transport | native audio/encoder path | DIFFERENT_EXECUTION |
| Canvas2D | approximate preview fallback | not equivalent certification path | KNOWN_APPROXIMATION |
| frame scheduling | RAF may skip requested frames | ordered complete export frame sequence | DIFFERENT_EXECUTION |
