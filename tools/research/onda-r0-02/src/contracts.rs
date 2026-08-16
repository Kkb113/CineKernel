use crate::{lock::UpstreamLock, MACHINE_OUTPUTS};
use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::{fs, path::Path};

pub fn generate(root: &Path, model: &Value, lock: &UpstreamLock) -> Result<()> {
    let dir = root.join("schemas/research/r0.02");
    fs::create_dir_all(&dir)?;
    write_json(
        &dir.join("research-model.schema.json"),
        &document_schema(model, Some(lock))?,
    )?;
    for name in MACHINE_OUTPUTS
        .iter()
        .filter(|name| **name != "R0_02_RESEARCH_MODEL.json")
    {
        let doc_path = root.join("docs/research/onda/r0.02").join(name);
        let doc: Value = serde_json::from_slice(&fs::read(&doc_path)?)?;
        let schema_name = schema_name(name);
        write_json(&dir.join(schema_name), &document_schema(&doc, Some(lock))?)?;
    }
    Ok(())
}

pub fn validate(root: &Path, model: &Value) -> Result<()> {
    validate_one(
        model,
        &read_json(&root.join("schemas/research/r0.02/research-model.schema.json"))?,
        "R0_02_RESEARCH_MODEL.json",
    )?;
    for name in MACHINE_OUTPUTS
        .iter()
        .filter(|name| **name != "R0_02_RESEARCH_MODEL.json")
    {
        let doc = read_json(&root.join("docs/research/onda/r0.02").join(name))?;
        let schema = read_json(&root.join("schemas/research/r0.02").join(schema_name(name)))?;
        validate_one(&doc, &schema, name)?;
    }
    Ok(())
}

pub fn document_schema(document: &Value, lock: Option<&UpstreamLock>) -> Result<Value> {
    let mut schema = strict_schema(document, None)?;
    let object = schema
        .as_object_mut()
        .context("document schema is not object")?;
    object.insert(
        "$schema".into(),
        json!("https://json-schema.org/draft/2020-12/schema"),
    );
    if let Some(lock) = lock {
        harden_sources(&mut schema, lock)?;
    }
    harden_graph(&mut schema);
    Ok(schema)
}

pub fn validate_one(document: &Value, schema: &Value, label: &str) -> Result<()> {
    let validator = jsonschema::validator_for(schema)
        .with_context(|| format!("compile strict schema for {label}"))?;
    if let Err(error) = validator.validate(document) {
        bail!("{label} strict schema validation failed: {error}")
    }
    Ok(())
}

fn strict_schema(value: &Value, field: Option<&str>) -> Result<Value> {
    Ok(match value {
        Value::Object(map) => {
            let mut properties = Map::new();
            let mut required = Vec::new();
            for (key, child) in map {
                properties.insert(key.clone(), strict_schema(child, Some(key))?);
                required.push(Value::String(key.clone()));
            }
            json!({"type":"object","additionalProperties":false,"required":required,"properties":properties})
        }
        Value::Array(items) => {
            let item_schema = if field == Some("sources") {
                json!({})
            } else if items.is_empty() {
                return Ok(json!({"type":"array","maxItems":0,"items":false}));
            } else {
                let first = items
                    .first()
                    .context("strict schema cannot infer an empty array")?;
                strict_schema(first, None)?
            };
            json!({"type":"array","minItems":1,"items":item_schema})
        }
        Value::String(_) => string_schema(field),
        Value::Bool(_) => json!({"type":"boolean"}),
        Value::Number(number) if number.is_u64() || number.is_i64() => {
            json!({"type":"integer"})
        }
        Value::Number(_) => json!({"type":"number"}),
        Value::Null => json!({"type":"null"}),
    })
}

fn string_schema(field: Option<&str>) -> Value {
    let key = field.unwrap_or_default();
    if matches!(key, "file_sha256") {
        return json!({"type":"string","pattern":"^[0-9a-f]{64}$"});
    }
    if matches!(
        key,
        "pinned_commit" | "pinned_tree" | "cinekernel_base" | "onda_pin" | "onda_tree"
    ) {
        return json!({"type":"string","pattern":"^[0-9a-f]{40}$"});
    }
    if key == "git_blob" {
        return json!({"type":"string","pattern":"^[0-9a-f]{40,64}$"});
    }
    if matches!(key, "generated_at" | "accessed_at_utc") {
        return json!({"type":"string","format":"date-time"});
    }
    if matches!(key, "repository" | "document_url") {
        return json!({"type":"string","format":"uri","pattern":"^https://"});
    }
    if key == "authority" {
        return json!({"type":"string","enum":["AUTHORITATIVE","DERIVED","TRANSIENT","CACHE","PRESENTATION_ONLY","EXTERNAL_RUNTIME","UNKNOWN"]});
    }
    if matches!(key, "mutability" | "state_mutability") {
        return json!({"type":"string","enum":["IMMUTABLE_DATA","MUTABLE_TREE","PER_FRAME_REBUILT","CLONED_AND_MUTATED","GLOBAL_MUTABLE","INSTANCE_MUTABLE","CACHE_MUTABLE","EXTERNAL_RUNTIME","UNKNOWN"]});
    }
    if key == "disposition" {
        return json!({"type":"string","enum":["PRESERVED","REMAPPED","USED_ONLY_DURING_LOWERING","USED_ONLY_AS_REACT_KEY","CONVERTED_TO_NUMERIC_ID","DROPPED","NOT_REPRESENTABLE","UNKNOWN","NORMALIZED","EXPANDED","MATERIALIZED","APPROXIMATED","FALLBACK_SUBSTITUTED","UNRESOLVED"]});
    }
    if matches!(key, "source_domain" | "target_domain") {
        return json!({"type":"string","enum":["FRAME_INDEX_INTEGER","FRAME_INDEX_FLOAT","COMPOSITION_FRAME_LOCAL","SEQUENCE_FRAME_LOCAL","SECONDS_NUMBER","FPS_FLOAT","SOURCE_MEDIA_SECONDS","AUDIO_CONTEXT_SECONDS","RAF_WALL_TIME","ENCODED_TIMESTAMP","TIMELINE_CLIP_FRAME","NONE","UNKNOWN"]});
    }
    if key == "behavior" {
        return json!({"type":"string","enum":["HARD_ERROR","VALIDATION_ERROR","WARNING","INFO_DIAGNOSTIC","SILENT_IGNORE","DEFAULT_SUBSTITUTION","VISUAL_PLACEHOLDER","AUTOMATIC_BACKEND_FALLBACK","APPROXIMATION","MALFORMED_VALUE_DROPPED","ASYNC_RETRY_OR_REPAINT","UNSUPPORTED","UNKNOWN"]});
    }
    if key == "confidence" {
        return json!({"type":"string","enum":["HIGH","MEDIUM","LOW"]});
    }
    if key.ends_with("_id") || key == "id" || key == "from" || key == "to" {
        return json!({"type":"string","pattern":"^[A-Z][A-Z0-9.-]*(?:-[A-Z0-9.-]+)+$"});
    }
    json!({"type":"string","minLength":1})
}

fn harden_sources(schema: &mut Value, lock: &UpstreamLock) -> Result<()> {
    visit_sources(schema, lock);
    Ok(())
}

fn harden_graph(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(properties) = map.get_mut("properties").and_then(Value::as_object_mut) {
                if let Some(nodes) = properties.get_mut("architecture_nodes") {
                    nodes["items"]["properties"]["kind"] = json!({"type":"string","enum":["AUTHORING_SURFACE","VALIDATOR","NORMALIZER","REGISTRY","RECONCILER","CONTEXT","TRANSIENT_TREE","SEMANTIC_MODEL","SCENE_MODEL","ANIMATION_MODEL","SERIALIZATION_BOUNDARY","PREPASS","PREVIEW_RUNTIME","EXPORT_RUNTIME","RENDERER_BOUNDARY","ENCODER_BOUNDARY","FALLBACK","DIAGNOSTIC_SURFACE","EXTERNAL_RUNTIME"]});
                    nodes["items"]["properties"]["time_domains"]["items"] = json!({"type":"string","enum":["FRAME_INDEX_INTEGER","FRAME_INDEX_FLOAT","COMPOSITION_FRAME_LOCAL","SEQUENCE_FRAME_LOCAL","SECONDS_NUMBER","FPS_FLOAT","SOURCE_MEDIA_SECONDS","AUDIO_CONTEXT_SECONDS","RAF_WALL_TIME","ENCODED_TIMESTAMP","TIMELINE_CLIP_FRAME","NONE","UNKNOWN"]});
                }
                if let Some(edges) = properties.get_mut("architecture_edges") {
                    edges["items"]["properties"]["kind"] = json!({"type":"string","enum":["EMITS","VALIDATES","NORMALIZES","LOWERS_TO","EXPANDS_TO","MATERIALIZES","SERIALIZES","DESERIALIZES","CLONES","MUTATES","OWNS_STATE","OWNS_TIME","TARGETS","INVOKES","FALLS_BACK_TO","APPROXIMATES_AS","DROPS_OR_FLATTENS","INSPECTS"]});
                }
            }
            for child in map.values_mut() {
                harden_graph(child);
            }
        }
        Value::Array(items) => {
            for child in items {
                harden_graph(child);
            }
        }
        _ => {}
    }
}

fn visit_sources(value: &mut Value, lock: &UpstreamLock) {
    match value {
        Value::Object(map) => {
            if let Some(properties) = map.get_mut("properties").and_then(Value::as_object_mut) {
                if let Some(source_array) = properties.get_mut("sources") {
                    source_array["items"] = source_record_schema(lock);
                    source_array["minItems"] = json!(50);
                }
            }
            for child in map.values_mut() {
                visit_sources(child, lock);
            }
        }
        Value::Array(items) => {
            for child in items {
                visit_sources(child, lock);
            }
        }
        _ => {}
    }
}

fn source_record_schema(lock: &UpstreamLock) -> Value {
    let facts = json!({"type":"array","minItems":1,"items":{"type":"string","minLength":1}});
    let local = json!({
        "type":"object","additionalProperties":false,
        "required":["source_id","repository","pinned_commit","pinned_tree","path","git_blob","symbol_or_section","start_line","end_line","file_sha256","classification","facts_supported"],
        "properties":{
            "source_id":{"type":"string","pattern":"^S-[A-Z0-9-]+$"},
            "repository":{"const":lock.repository},
            "pinned_commit":{"const":lock.pinned_commit},
            "pinned_tree":{"const":lock.pinned_tree},
            "path":{"type":"string","minLength":1,"not":{"pattern":"(?:^|/)(?:main|master)(?:/|$)"}},
            "git_blob":{"type":"string","pattern":"^[0-9a-f]{40,64}$"},
            "symbol_or_section":{"type":"string","minLength":1},
            "start_line":{"type":"integer","minimum":1},
            "end_line":{"type":"integer","minimum":1},
            "file_sha256":{"type":"string","pattern":"^[0-9a-f]{64}$"},
            "classification":{"enum":["UPSTREAM_SOURCE","UPSTREAM_TEST","UPSTREAM_MANIFEST","UPSTREAM_DOCUMENTATION"]},
            "facts_supported":facts
        }
    });
    let external = json!({
        "type":"object","additionalProperties":false,
        "required":["source_id","classification","publisher","document_title","document_url","accessed_at_utc","section","facts_supported"],
        "properties":{
            "source_id":{"type":"string","pattern":"^E-[A-Z0-9-]+$"},
            "classification":{"enum":["PRIMARY_STANDARD","PRIMARY_IMPLEMENTATION_DOC"]},
            "publisher":{"type":"string","minLength":1},
            "document_title":{"type":"string","minLength":1},
            "document_url":{"type":"string","format":"uri","pattern":"^https://"},
            "accessed_at_utc":{"type":"string","format":"date-time"},
            "section":{"type":"string","minLength":1},
            "facts_supported":facts
        }
    });
    json!({"oneOf":[local,external]})
}

fn schema_name(name: &str) -> String {
    name.to_ascii_lowercase()
        .replace('_', "-")
        .replace(".json", ".schema.json")
}

fn read_json(path: &Path) -> Result<Value> {
    serde_json::from_slice(&fs::read(path)?).with_context(|| format!("parse {}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lock() -> UpstreamLock {
        UpstreamLock {
            repository: "https://github.com/onda-engine/onda-engine.git".into(),
            pinned_commit: "a".repeat(40),
            pinned_tree: "b".repeat(40),
        }
    }

    fn source() -> Value {
        json!({"sources":[{"source_id":"S-X","repository":"https://github.com/onda-engine/onda-engine.git","pinned_commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","pinned_tree":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","path":"x.ts","git_blob":"cccccccccccccccccccccccccccccccccccccccc","symbol_or_section":"WHOLE_FILE","start_line":1,"end_line":1,"file_sha256":"dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd","classification":"UPSTREAM_SOURCE","facts_supported":["fact"]}]})
    }

    #[test]
    fn nested_extra_property_is_rejected() {
        let doc = source();
        let schema = document_schema(&doc, Some(&lock())).unwrap();
        let mut bad = doc;
        bad["sources"][0]["extra"] = json!(true);
        assert!(validate_one(&bad, &schema, "mutation").is_err());
    }
    #[test]
    fn source_wrong_repository_is_rejected() {
        let doc = source();
        let schema = document_schema(&doc, Some(&lock())).unwrap();
        let mut bad = doc;
        bad["sources"][0]["repository"] = json!("https://github.com/onda-video/onda");
        assert!(validate_one(&bad, &schema, "mutation").is_err());
    }
    #[test]
    fn source_wrong_blob_is_rejected() {
        let doc = source();
        let schema = document_schema(&doc, Some(&lock())).unwrap();
        let mut bad = doc;
        bad["sources"][0]["git_blob"] = json!("short");
        assert!(validate_one(&bad, &schema, "mutation").is_err());
    }
    #[test]
    fn source_invalid_line_is_rejected() {
        let doc = source();
        let schema = document_schema(&doc, Some(&lock())).unwrap();
        let mut bad = doc;
        bad["sources"][0]["start_line"] = json!(0);
        assert!(validate_one(&bad, &schema, "mutation").is_err());
    }
    #[test]
    fn source_missing_symbol_is_rejected() {
        let doc = source();
        let schema = document_schema(&doc, Some(&lock())).unwrap();
        let mut bad = doc;
        bad["sources"][0]
            .as_object_mut()
            .unwrap()
            .remove("symbol_or_section");
        assert!(validate_one(&bad, &schema, "mutation").is_err());
    }
}
