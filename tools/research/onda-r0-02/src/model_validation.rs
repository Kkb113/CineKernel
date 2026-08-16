use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::collections::BTreeSet;

const NODE_KINDS: &[&str] = &[
    "AUTHORING_SURFACE",
    "VALIDATOR",
    "NORMALIZER",
    "REGISTRY",
    "RECONCILER",
    "CONTEXT",
    "TRANSIENT_TREE",
    "SEMANTIC_MODEL",
    "SCENE_MODEL",
    "ANIMATION_MODEL",
    "SERIALIZATION_BOUNDARY",
    "PREPASS",
    "PREVIEW_RUNTIME",
    "EXPORT_RUNTIME",
    "RENDERER_BOUNDARY",
    "ENCODER_BOUNDARY",
    "FALLBACK",
    "DIAGNOSTIC_SURFACE",
    "EXTERNAL_RUNTIME",
];
const EDGE_KINDS: &[&str] = &[
    "EMITS",
    "VALIDATES",
    "NORMALIZES",
    "LOWERS_TO",
    "EXPANDS_TO",
    "MATERIALIZES",
    "SERIALIZES",
    "DESERIALIZES",
    "CLONES",
    "MUTATES",
    "OWNS_STATE",
    "OWNS_TIME",
    "TARGETS",
    "INVOKES",
    "FALLS_BACK_TO",
    "APPROXIMATES_AS",
    "DROPS_OR_FLATTENS",
    "INSPECTS",
];

pub fn validate(model: &Value) -> Result<()> {
    graph(
        model["architecture_nodes"]
            .as_array()
            .context("nodes missing")?,
        model["architecture_edges"]
            .as_array()
            .context("edges missing")?,
    )?;
    semantics(
        model["semantic_preservation"]
            .as_array()
            .context("semantics missing")?,
    )?;
    fallbacks(
        model["validation_and_fallbacks"]
            .as_array()
            .context("fallbacks missing")?,
    )?;
    requirements(
        model["candidate_requirements"]
            .as_array()
            .context("requirements missing")?,
    )?;
    creativity(&model["creative_programmability"])?;
    novel_scenes(
        model["novel_scene_litmus"]
            .as_array()
            .context("novel scenes missing")?,
    )?;
    stable_order(
        model["sources"].as_array().context("sources missing")?,
        "source_id",
    )?;
    stable_order(model["architecture_nodes"].as_array().unwrap(), "id")?;
    stable_order(model["architecture_edges"].as_array().unwrap(), "id")?;
    stable_order(
        model["candidate_requirements"].as_array().unwrap(),
        "requirement_id",
    )?;
    Ok(())
}

pub fn graph(nodes: &[Value], edges: &[Value]) -> Result<()> {
    let mut node_ids = BTreeSet::new();
    for node in nodes {
        let id = req(node, "id")?;
        if !node_ids.insert(id) {
            bail!("duplicate graph node {id}")
        }
        if !NODE_KINDS.contains(&req(node, "kind")?) {
            bail!("invalid graph node kind")
        }
        for field in ["authority", "mutability"] {
            req(node, field)?;
        }
        if node["time_domains"].as_array().is_none_or(Vec::is_empty) {
            bail!("graph node missing time domain")
        }
        refs(node)?;
    }
    let mut edge_ids = BTreeSet::new();
    for edge in edges {
        let id = req(edge, "id")?;
        if !edge_ids.insert(id) {
            bail!("duplicate graph edge {id}")
        }
        if !EDGE_KINDS.contains(&req(edge, "kind")?) {
            bail!("invalid graph edge kind")
        }
        if !node_ids.contains(req(edge, "from")?) || !node_ids.contains(req(edge, "to")?) {
            bail!("dangling graph endpoint")
        }
        for field in [
            "data_form",
            "validation",
            "semantic_disposition",
            "error_behavior",
        ] {
            req(edge, field)?;
        }
        refs(edge)?;
    }
    Ok(())
}

pub fn semantics(rows: &[Value]) -> Result<()> {
    for row in rows {
        for field in [
            "source_representation",
            "target_representation",
            "disposition",
            "editing_impact",
            "diagnostic_impact",
            "agent_repair_impact",
            "incremental_compilation_impact",
        ] {
            req(row, field)?;
        }
        if req(row, "disposition")? == "UNRESOLVED"
            && row.get("confidence").and_then(Value::as_str) == Some("HIGH")
        {
            bail!("unresolved semantic row cannot be high confidence")
        }
        refs(row)?;
    }
    Ok(())
}

pub fn fallbacks(rows: &[Value]) -> Result<()> {
    for row in rows {
        for field in [
            "trigger",
            "behavior",
            "visual_outcome",
            "semantic_impact",
            "visual_impact",
            "timing_impact",
            "determinism_impact",
            "preview_export_difference",
            "repairability",
        ] {
            req(row, field)?;
        }
        if row["quality_reducing"].as_bool() == Some(true) && req(row, "visual_impact")?.is_empty()
        {
            bail!("quality reducing fallback lacks impact")
        }
        if row["user_or_agent_informed"].as_bool() == Some(false)
            && matches!(req(row, "behavior")?, "WARNING" | "INFO_DIAGNOSTIC")
        {
            bail!("silent fallback mislabeled as diagnostic")
        }
        refs(row)?;
    }
    Ok(())
}

pub fn requirements(rows: &[Value]) -> Result<()> {
    for row in rows {
        if !req(row, "requirement_id")?.starts_with("CK-R002-REQ-") {
            bail!("candidate requirement ID is invalid")
        }
        if req(row, "status")? == "FINAL" {
            bail!("R0.02 requirement incorrectly marked final")
        }
        for field in [
            "abstract_requirement",
            "problem_addressed",
            "quality_impact",
            "trust_impact",
            "performance_impact",
            "creative_programming_impact",
            "prohibited_reuse_note",
        ] {
            req(row, field)?;
        }
        for field in [
            "onda_observation_refs",
            "onda_source_refs",
            "independent_primary_source_refs",
            "affected_cinekernel_programs",
            "required_follow_up_research",
        ] {
            if row[field].as_array().is_none_or(Vec::is_empty) {
                bail!("requirement missing {field}")
            }
        }
    }
    Ok(())
}

pub fn creativity(value: &Value) -> Result<()> {
    if value.get("numeric_score").is_some() {
        bail!("creative programmability must not use a numeric score")
    }
    if !req(value, "scoring_policy")?.contains("No numeric") {
        bail!("creative scoring policy must reject simplified numeric scoring")
    }
    let rows = value["surface_assessments"]
        .as_array()
        .context("surface assessments missing")?;
    for row in rows {
        for field in [
            "general_primitive_access",
            "procedural_logic",
            "custom_component_extension",
            "registry_dependence",
            "named_pattern_dependence",
            "can_descend_to_primitives",
            "inspectability",
            "post_generation_editability",
            "source_mapping",
            "novel_scene_expressibility",
        ] {
            if row.get(field).and_then(Value::as_bool).is_none() {
                bail!("creative assessment missing boolean {field}")
            }
        }
        if row["general_primitive_access"].as_bool() == Some(true)
            && req(row, "escape_hatch_evidence")?.contains("No general")
        {
            bail!("open-ended classification lacks escape-hatch evidence")
        }
        refs(row)?;
    }
    Ok(())
}

pub fn novel_scenes(rows: &[Value]) -> Result<()> {
    for row in rows {
        if row["required_primitives"]
            .as_array()
            .is_none_or(Vec::is_empty)
        {
            bail!("novel scene lacks required primitives")
        }
        if row["surface_comparison"]
            .as_array()
            .is_none_or(Vec::is_empty)
        {
            bail!("novel scene lacks per-surface comparison")
        }
        let decomposition = row["capability_decomposition"]
            .as_array()
            .context("novel scene decomposition missing")?;
        if decomposition.is_empty()
            || decomposition
                .iter()
                .any(|item| req(item, "required_primitive").is_err())
        {
            bail!("novel scene capability row lacks required primitive")
        }
        refs(row)?;
    }
    Ok(())
}

pub fn stable_order(rows: &[Value], field: &str) -> Result<()> {
    let actual: Vec<_> = rows.iter().map(|r| req(r, field)).collect::<Result<_>>()?;
    let mut sorted = actual.clone();
    sorted.sort();
    if actual != sorted {
        bail!("{field} records are not stably sorted")
    }
    Ok(())
}
fn req<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .with_context(|| format!("missing {key}"))
}
fn refs(v: &Value) -> Result<()> {
    if v["source_refs"].as_array().is_none_or(Vec::is_empty) {
        bail!("missing source refs")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn node(id: &str) -> Value {
        json!({"id":id,"kind":"SCENE_MODEL","authority":"AUTHORITATIVE","mutability":"IMMUTABLE_DATA","time_domains":["NONE"],"source_refs":["S-X"]})
    }
    fn edge(id: &str, from: &str, to: &str) -> Value {
        json!({"id":id,"from":from,"to":to,"kind":"LOWERS_TO","data_form":"x","validation":"x","semantic_disposition":"x","error_behavior":"x","source_refs":["S-X"]})
    }
    fn semantic() -> Value {
        json!({"source_representation":"a","target_representation":"b","disposition":"DROPPED","editing_impact":"x","diagnostic_impact":"x","agent_repair_impact":"x","incremental_compilation_impact":"x","confidence":"MEDIUM","source_refs":["S-X"]})
    }
    fn fallback() -> Value {
        json!({"trigger":"x","behavior":"APPROXIMATION","visual_outcome":"x","semantic_impact":"x","visual_impact":"x","timing_impact":"x","determinism_impact":"x","preview_export_difference":"x","repairability":"x","quality_reducing":true,"user_or_agent_informed":true,"source_refs":["S-X"]})
    }
    fn requirement() -> Value {
        json!({"requirement_id":"CK-R002-REQ-001","status":"CANDIDATE_ONLY","abstract_requirement":"x","problem_addressed":"x","quality_impact":"x","trust_impact":"x","performance_impact":"x","creative_programming_impact":"x","prohibited_reuse_note":"x","onda_observation_refs":["C-1"],"onda_source_refs":["S-X"],"independent_primary_source_refs":["E-X"],"affected_cinekernel_programs":["compiler"],"required_follow_up_research":["R0.03"]})
    }
    fn creative() -> Value {
        json!({"overall_verdict":"MULTI_LAYER","scoring_policy":"No numeric creativity score is assigned.","surface_assessments":[{"general_primitive_access":true,"procedural_logic":true,"custom_component_extension":true,"registry_dependence":false,"named_pattern_dependence":false,"can_descend_to_primitives":true,"inspectability":true,"post_generation_editability":true,"source_mapping":false,"novel_scene_expressibility":true,"escape_hatch_evidence":"Host-language code","source_refs":["S-X"]}]})
    }
    fn novel() -> Value {
        json!({"required_primitives":["geometry"],"surface_comparison":[{"surface_id":"AS-X"}],"capability_decomposition":[{"required_primitive":"geometry"}],"source_refs":["S-X"]})
    }
    #[test]
    fn graph_rejects_duplicate_node() {
        assert!(graph(&[node("N-X"), node("N-X")], &[]).is_err())
    }
    #[test]
    fn graph_rejects_duplicate_edge() {
        assert!(graph(
            &[node("N-X")],
            &[edge("AE-X", "N-X", "N-X"), edge("AE-X", "N-X", "N-X")]
        )
        .is_err())
    }
    #[test]
    fn graph_rejects_dangling_endpoint() {
        assert!(graph(&[node("N-X")], &[edge("AE-X", "N-X", "N-MISSING")]).is_err())
    }
    #[test]
    fn graph_rejects_invalid_node_kind() {
        let mut n = node("N-X");
        n["kind"] = json!("bad");
        assert!(graph(&[n], &[]).is_err())
    }
    #[test]
    fn graph_rejects_invalid_edge_kind() {
        let mut e = edge("AE-X", "N-X", "N-X");
        e["kind"] = json!("bad");
        assert!(graph(&[node("N-X")], &[e]).is_err())
    }
    #[test]
    fn graph_rejects_missing_authority() {
        let mut n = node("N-X");
        n.as_object_mut().unwrap().remove("authority");
        assert!(graph(&[n], &[]).is_err())
    }
    #[test]
    fn graph_rejects_missing_mutability() {
        let mut n = node("N-X");
        n.as_object_mut().unwrap().remove("mutability");
        assert!(graph(&[n], &[]).is_err())
    }
    #[test]
    fn graph_rejects_missing_time_domain() {
        let mut n = node("N-X");
        n["time_domains"] = json!([]);
        assert!(graph(&[n], &[]).is_err())
    }
    #[test]
    fn semantic_rejects_missing_source() {
        let mut r = semantic();
        r.as_object_mut().unwrap().remove("source_representation");
        assert!(semantics(&[r]).is_err())
    }
    #[test]
    fn semantic_rejects_missing_target() {
        let mut r = semantic();
        r.as_object_mut().unwrap().remove("target_representation");
        assert!(semantics(&[r]).is_err())
    }
    #[test]
    fn semantic_rejects_missing_disposition() {
        let mut r = semantic();
        r.as_object_mut().unwrap().remove("disposition");
        assert!(semantics(&[r]).is_err())
    }
    #[test]
    fn semantic_rejects_loss_without_impact() {
        let mut r = semantic();
        r["editing_impact"] = json!("");
        assert!(semantics(&[r]).is_err())
    }
    #[test]
    fn semantic_rejects_verified_unresolved() {
        let mut r = semantic();
        r["disposition"] = json!("UNRESOLVED");
        r["confidence"] = json!("HIGH");
        assert!(semantics(&[r]).is_err())
    }
    #[test]
    fn fallback_rejects_missing_trigger() {
        let mut r = fallback();
        r.as_object_mut().unwrap().remove("trigger");
        assert!(fallbacks(&[r]).is_err())
    }
    #[test]
    fn fallback_rejects_missing_outcome() {
        let mut r = fallback();
        r.as_object_mut().unwrap().remove("visual_outcome");
        assert!(fallbacks(&[r]).is_err())
    }
    #[test]
    fn fallback_rejects_quality_loss_without_impact() {
        let mut r = fallback();
        r["visual_impact"] = json!("");
        assert!(fallbacks(&[r]).is_err())
    }
    #[test]
    fn fallback_rejects_silent_warning() {
        let mut r = fallback();
        r["behavior"] = json!("WARNING");
        r["user_or_agent_informed"] = json!(false);
        assert!(fallbacks(&[r]).is_err())
    }
    #[test]
    fn requirement_rejects_missing_onda_evidence() {
        let mut r = requirement();
        r["onda_source_refs"] = json!([]);
        assert!(requirements(&[r]).is_err())
    }
    #[test]
    fn requirement_rejects_missing_primary_source() {
        let mut r = requirement();
        r["independent_primary_source_refs"] = json!([]);
        assert!(requirements(&[r]).is_err())
    }
    #[test]
    fn requirement_rejects_final_status() {
        let mut r = requirement();
        r["status"] = json!("FINAL");
        assert!(requirements(&[r]).is_err())
    }
    #[test]
    fn requirement_rejects_missing_program() {
        let mut r = requirement();
        r["affected_cinekernel_programs"] = json!([]);
        assert!(requirements(&[r]).is_err())
    }
    #[test]
    fn stable_order_rejects_unsorted() {
        assert!(stable_order(&[json!({"id":"B"}), json!({"id":"A"})], "id").is_err())
    }
    #[test]
    fn creativity_rejects_numeric_score() {
        let mut row = creative();
        row["numeric_score"] = json!(0.8);
        assert!(creativity(&row).is_err())
    }
    #[test]
    fn creativity_requires_escape_hatch_evidence() {
        let mut row = creative();
        row["surface_assessments"][0]["escape_hatch_evidence"] = json!("No general escape hatch");
        assert!(creativity(&row).is_err())
    }
    #[test]
    fn novel_scene_requires_primitive() {
        let mut row = novel();
        row["required_primitives"] = json!([]);
        assert!(novel_scenes(&[row]).is_err())
    }
    #[test]
    fn novel_scene_capability_requires_primitive() {
        let mut row = novel();
        row["capability_decomposition"][0]
            .as_object_mut()
            .unwrap()
            .remove("required_primitive");
        assert!(novel_scenes(&[row]).is_err())
    }
}
