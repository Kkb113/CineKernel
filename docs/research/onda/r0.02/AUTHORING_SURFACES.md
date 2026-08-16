# Authoring surfaces

Five authoring surfaces were distinguished instead of collapsed into one score.

| Surface | Authority | Mutability | Output | Programming classes |
|---|---|---|---|---|
| Cinema payload | serialized payload | IMMUTABLE_DATA | React element program | DECLARATIVE_SCENE_PROGRAM, EXTENSIBLE_COMPONENT_SYSTEM, FINITE_PATTERN_CATALOG |
| React composition | host-language program | PER_FRAME_REBUILT | one Scene snapshot | PROCEDURAL_HOST_LANGUAGE, GENERAL_PRIMITIVE_SYSTEM, EXTERNAL_CODE_ESCAPE_HATCH |
| Direct Scene JSON | serialized Scene document | IMMUTABLE_DATA | prepass-ready Scene | DECLARATIVE_SCENE_PROGRAM, GENERAL_PRIMITIVE_SYSTEM |
| Rust Scene and Timeline | typed Rust values | CLONED_AND_MUTATED | evaluated Scene | PROCEDURAL_HOST_LANGUAGE, DECLARATIVE_SCENE_PROGRAM |
| Component package and registry | registry definition | IMMUTABLE_DATA | React primitive subtree | EXTENSIBLE_COMPONENT_SYSTEM, FINITE_REGISTRY, EXTERNAL_CODE_ESCAPE_HATCH |

The Cinema inspector is not another authoring surface. It is a parallel high-level semantic analysis route over the Cinema payload.
