# Identity and source mapping

10 identity transitions were classified.

| Concept | Source → target | Disposition | Traceable at final Scene |
|---|---|---|---|
| Cinema scene ID | Cinema scene → React composition | USED_ONLY_DURING_LOWERING | false |
| Cinema track ID | Cinema track → React grouping | USED_ONLY_DURING_LOWERING | false |
| Cinema entry ID | Cinema entry → React key | USED_ONLY_AS_REACT_KEY | false |
| Cinema morph key | Cinema entry → transition matching | USED_ONLY_DURING_LOWERING | false |
| React key | React element → HostNode/Scene | DROPPED | false |
| React host node | HostNode tree → Scene Node | REMAPPED | false |
| numeric Scene NodeId | Scene → renderer/timeline | PRESERVED | true |
| Timeline target NodeId | Timeline → Scene mutation target | PRESERVED | true |
| renderer element identity | Scene Node → render pass | USED_ONLY_DURING_LOWERING | false |
| preview selection identity | Cinema inspector → rendered node | NOT_REPRESENTABLE | false |

No complete intent-to-pixel source map was found.
