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
const CREATIVE_STATES: &[&str] = &[
    "SUPPORTED",
    "PARTIALLY_SUPPORTED",
    "PARTIALLY_SUPPORTED_BEFORE_LOWERING",
    "SUPPORTED_THROUGH_HOST_LANGUAGE",
    "SUPPORTED_THROUGH_CUSTOM_REGISTRY",
    "REQUIRES_LOWER_LEVEL_SCENE_ACCESS",
    "FINITE_CATALOG_LIMIT",
    "NOT_NATIVE",
    "NOT_REPRESENTABLE_AT_THIS_LAYER",
    "UNKNOWN",
];
const R0_REGISTRY: &[(&str, &str)] = &[
    ("R0.03", "Native GPU, CPU, WASM, and encoding architecture"),
    (
        "R0.04",
        "Typography, layout, effects, color, and 3D architecture",
    ),
    (
        "R0.05",
        "Agent component catalog and cinematic composition model",
    ),
    (
        "R0.06",
        "CLI, installation, preview, embedding, and developer experience",
    ),
    ("R0.07", "Independent benchmark and failure analysis"),
    (
        "R0.08",
        "Adoption, rejection, clean-room, and roadmap-delta matrix",
    ),
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
    claims(model["claims"].as_array().context("claims missing")?)?;
    creativity(&model["creative_programmability"])?;
    novel_scenes(
        model["novel_scene_litmus"]
            .as_array()
            .context("novel scenes missing")?,
    )?;
    roadmap(
        model["deferred_topics"]
            .as_array()
            .context("deferred topics missing")?,
        model["open_questions"]
            .as_array()
            .context("open questions missing")?,
        model["candidate_requirements"].as_array().unwrap(),
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
    let mut impact_signatures = BTreeSet::new();
    for row in rows {
        if !req(row, "requirement_id")?.starts_with("CK-R002-REQ-") {
            bail!("candidate requirement ID is invalid")
        }
        if req(row, "status")? != "CANDIDATE_ONLY" {
            bail!("R0.02 requirement must remain CANDIDATE_ONLY")
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
        for program in row["affected_cinekernel_programs"].as_array().unwrap() {
            let program = program.as_str().context("program ID is not a string")?;
            let number = program
                .strip_prefix('P')
                .and_then(|value| value.parse::<u8>().ok())
                .filter(|value| (1..=28).contains(value));
            if program.len() != 3 || number.is_none() {
                bail!("requirement uses an unlocked CineKernel program identifier")
            }
        }
        let signature = format!(
            "{}|{}|{}|{}",
            req(row, "quality_impact")?,
            req(row, "trust_impact")?,
            req(row, "performance_impact")?,
            req(row, "creative_programming_impact")?
        );
        if !impact_signatures.insert(signature) {
            bail!("candidate requirements reuse a generic impact block")
        }
        match req(row, "requirement_id")? {
            "CK-R002-REQ-001"
                if !has(row, "onda_observation_refs", "C-004")
                    || !has(row, "affected_cinekernel_programs", "P25") =>
            {
                bail!("cross-stage identity traceability is mismapped")
            }
            "CK-R002-REQ-002"
                if !has(row, "onda_source_refs", "S-CINEMA-TIME")
                    || !has(row, "onda_source_refs", "S-AUDIO")
                    || !has(row, "affected_cinekernel_programs", "P19") =>
            {
                bail!("time-domain traceability is mismapped")
            }
            "CK-R002-REQ-004"
                if !has(row, "onda_observation_refs", "C-007")
                    || !has(row, "onda_observation_refs", "C-008")
                    || !has(row, "onda_source_refs", "S-CANVAS") =>
            {
                bail!("fallback traceability is mismapped")
            }
            "CK-R002-REQ-005"
                if !has(row, "onda_observation_refs", "C-009")
                    || !has(row, "affected_cinekernel_programs", "P26") =>
            {
                bail!("bounded-generation traceability is mismapped")
            }
            _ => {}
        }
    }
    Ok(())
}

fn has(row: &Value, field: &str, expected: &str) -> bool {
    row[field]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item == expected))
}

pub fn claims(rows: &[Value]) -> Result<()> {
    for row in rows {
        if !req(row, "claim_id")?.starts_with("C-") {
            bail!("claim ID is invalid")
        }
        if !matches!(
            req(row, "status")?,
            "VERIFIED_AT_PIN"
                | "INFERRED_FROM_MULTIPLE_SOURCES"
                | "CONTRADICTED"
                | "CANDIDATE_ONLY"
                | "UNRESOLVED"
        ) {
            bail!("claim status is invalid")
        }
        if req(row, "status")? == "UNRESOLVED" && req(row, "confidence")? == "HIGH" {
            bail!("unresolved claim cannot have high confidence")
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
            "custom_geometry",
            "custom_animation",
            "custom_component_extension",
            "registry_dependence",
            "named_pattern_dependence",
            "can_descend_to_primitives",
            "inspectability",
            "post_generation_editability",
            "source_mapping",
            "novel_scene_expressibility",
        ] {
            if !CREATIVE_STATES.contains(&req(row, field)?) {
                bail!("creative assessment has invalid categorical state for {field}")
            }
        }
        if req(row, "general_primitive_access")? == "SUPPORTED"
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
        if row.get("litmus_id").and_then(Value::as_str) == Some("LITMUS-LAPTOP") {
            let segmentation = decomposition
                .iter()
                .find(|item| item["dimension"] == "segmentation")
                .context("laptop segmentation row missing")?;
            if req(segmentation, "status")? != "PARTIAL_OR_NOT_ESTABLISHED"
                || !req(segmentation, "evidence_scope")?
                    .contains("automatic semantic or mechanical segmentation")
            {
                bail!("laptop segmentation overclaims automatic asset understanding")
            }
            let sound = decomposition
                .iter()
                .find(|item| item["dimension"] == "sound")
                .context("laptop sound row missing")?;
            if !sound["source_refs"]
                .as_array()
                .is_some_and(|refs| refs.iter().any(|item| item == "S-AUDIO"))
            {
                bail!("laptop sound row lacks audio-specific evidence")
            }
        }
        refs(row)?;
    }
    Ok(())
}

pub fn roadmap(deferred: &[Value], questions: &[Value], requirements: &[Value]) -> Result<()> {
    if deferred.len() != R0_REGISTRY.len() {
        bail!("locked R0 registry has the wrong number of phases")
    }
    for (row, (phase, topic)) in deferred.iter().zip(R0_REGISTRY) {
        if req(row, "phase")? != *phase || req(row, "topic")? != *topic {
            bail!("locked R0 registry drifted at {phase}")
        }
    }
    for routes in questions.iter().map(|row| &row["defer_to"]).chain(
        requirements
            .iter()
            .map(|row| &row["required_follow_up_research"]),
    ) {
        let routes = routes.as_array().context("R0 routing must be an array")?;
        if routes.is_empty()
            || routes.iter().any(|route| {
                route
                    .as_str()
                    .is_none_or(|phase| !R0_REGISTRY.iter().any(|entry| entry.0 == phase))
            })
        {
            bail!("future-work routing references an unlocked R0 phase")
        }
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
        json!({"requirement_id":"CK-R002-REQ-001","status":"CANDIDATE_ONLY","abstract_requirement":"x","problem_addressed":"x","quality_impact":"x","trust_impact":"x","performance_impact":"x","creative_programming_impact":"x","prohibited_reuse_note":"x","onda_observation_refs":["C-001"],"onda_source_refs":["S-X"],"independent_primary_source_refs":["E-X"],"affected_cinekernel_programs":["P01"],"required_follow_up_research":["R0.03"]})
    }
    fn creative() -> Value {
        json!({"overall_verdict":"MULTI_LAYER","scoring_policy":"No numeric creativity score is assigned.","surface_assessments":[{"general_primitive_access":"SUPPORTED","procedural_logic":"SUPPORTED_THROUGH_HOST_LANGUAGE","custom_geometry":"FINITE_CATALOG_LIMIT","custom_animation":"SUPPORTED_THROUGH_HOST_LANGUAGE","custom_component_extension":"SUPPORTED_THROUGH_HOST_LANGUAGE","registry_dependence":"NOT_NATIVE","named_pattern_dependence":"NOT_NATIVE","can_descend_to_primitives":"SUPPORTED","inspectability":"SUPPORTED","post_generation_editability":"SUPPORTED_THROUGH_HOST_LANGUAGE","source_mapping":"PARTIALLY_SUPPORTED","novel_scene_expressibility":"SUPPORTED_THROUGH_HOST_LANGUAGE","escape_hatch_evidence":"Host-language code","source_refs":["S-X"]}]})
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
    fn creativity_rejects_boolean_compression() {
        let mut row = creative();
        row["surface_assessments"][0]["custom_animation"] = json!(true);
        assert!(creativity(&row).is_err())
    }
    #[test]
    fn claim_candidate_status_is_explicit() {
        assert!(claims(&[
            json!({"claim_id":"C-011","status":"CANDIDATE_ONLY","confidence":"HIGH"})
        ])
        .is_ok());
        assert!(
            claims(&[json!({"claim_id":"C-011","status":"UNRESOLVED","confidence":"HIGH"})])
                .is_err()
        );
    }
    #[test]
    fn roadmap_rejects_definition_drift() {
        let mut deferred: Vec<Value> = R0_REGISTRY.iter().enumerate().map(|(i, (phase, topic))| json!({"topic_id":format!("DEF-{:03}",i+1),"phase":phase,"topic":topic})).collect();
        deferred[1]["topic"] = json!("time and identity architecture");
        assert!(roadmap(
            &deferred,
            &[json!({"defer_to":["R0.03"]})],
            &[requirement()]
        )
        .is_err());
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
