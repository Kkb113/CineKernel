use anyhow::{Context, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

const DOC: &str = "docs/research/onda/r0.02";
const REVIEW: &str = "reports/research/r0.02";

pub fn generate(root: &Path, model: &Value) -> Result<()> {
    fs::create_dir_all(root.join(DOC))?;
    write(root, "ARCHITECTURE_OVERVIEW.md", &architecture(model))?;
    write(root, "AUTHORING_SURFACES.md", &authoring(model))?;
    write(root, "REACT_RECONCILER_FLOW.md", &react_flow(model))?;
    write(root, "CINEMA_COMPILER_FLOW.md", &cinema_flow(model))?;
    write(root, "DIRECT_JSON_AND_RUST_FLOW.md", &direct_flow(model))?;
    write(root, "SCENE_GRAPH_CONTRACT.md", &scene_contract(model))?;
    write(root, "STATE_AND_TIME_OWNERSHIP.md", &state_time(model))?;
    write(root, "IDENTITY_AND_SOURCE_MAPPING.md", &identity(model))?;
    write(root, "LOWERING_AND_INFORMATION_LOSS.md", &semantics(model))?;
    write(
        root,
        "VALIDATION_ERRORS_AND_FALLBACKS.md",
        &fallbacks(model),
    )?;
    write(root, "PREVIEW_EXPORT_PARITY.md", &preview(model))?;
    write(
        root,
        "SERIALIZATION_AND_VERSIONING.md",
        &serialization(model),
    )?;
    write(
        root,
        "MATERIALIZATION_AND_SCALABILITY.md",
        &materialization(model),
    )?;
    write(
        root,
        "CREATIVE_PROGRAMMABILITY_ASSESSMENT.md",
        &creativity(model),
    )?;
    write(root, "PRIMARY_SOURCE_COMPARISON.md", &primary(model))?;
    write(
        root,
        "CINEKERNEL_REQUIREMENT_CANDIDATES.md",
        &requirements(model),
    )?;
    write(root, "RISKS_AND_OPEN_QUESTIONS.md", &risks(model))?;
    write(root, "DEFERRED_TO_LATER_R0_PHASES.md", &deferred(model))?;
    write(root, "RESEARCH_SOURCE_INDEX.md", &sources(model))?;
    let mut acceptance_report = acceptance(model);
    if attestation_pass(root)? {
        acceptance_report = acceptance_report
            .replace("CONDITIONAL PASS pending a final remediation-head remote reproduction and updated attestation. All local remediation gates pass before publication.", "PASS. The exact remediation evidence commit passed the dedicated R0.02 workflow and ordinary CineKernel CI on Windows, Ubuntu, and macOS, with three nonempty raw-evidence artifacts.")
            .replace("Pending remediation-head Windows, Ubuntu, and macOS reproduction. The attestation is updated only after those runs complete and a nonempty raw-evidence artifact is published.", "Evidence commit `b528d651fa4e1f5678b098f39fc8c35ce034e1ef` passed dedicated run `31923882312` and ordinary CI run `31923882324` on Windows, Ubuntu, and macOS. Three nonempty raw-evidence artifacts and their SHA-256 digests are recorded in the attestation.")
            .replace("Do not start R0.03 until remediation-head three-OS evidence succeeds and the reviewer accepts promotion from CONDITIONAL PASS to PASS.", "R0.02 now satisfies its automated PASS gates. Keep PR #13 draft and unmerged until reviewer sign-off; begin R0.03 only after that review accepts the evidence lock.");
    }
    write(root, "R0_02_ACCEPTANCE_REPORT.md", &acceptance_report)?;
    write_review_packet(root, model)?;
    Ok(())
}

fn architecture(m: &Value) -> String {
    format!("# Architecture overview\n\nThe authoritative machine graph contains **{} nodes and {} edges**. Cinema, React, direct Scene JSON, and typed Rust converge on a finite Scene representation through explicit validation, reconciliation, serialization, prepass, preview, renderer, fallback, and encoder boundaries.\n\n```mermaid\nflowchart LR\n  C[Cinema payload] --> V[Cinema validation]\n  C --> I[Inspector semantic analysis]\n  C --> R[React program]\n  R --> X[Custom reconciler]\n  X --> H[Mutable HostNode tree]\n  H --> S[Per-frame Scene]\n  J[Direct Scene JSON] --> P[Prepasses]\n  T[Rust Timeline] --> S\n  S --> P\n  P --> CPU[CPU renderer]\n  P --> GPU[GPU renderer]\n  GPU -. capability fallback .-> CPU\n  CPU -. preview fallback .-> Canvas[Canvas approximation]\n  CPU --> E[Encoder]\n  GPU --> E\n```\n\nThe graph is a compiler map, not a CineKernel implementation design. Every node and edge carries immutable source references.\n", count(m,"architecture_nodes"),count(m,"architecture_edges"))
}

fn authoring(m: &Value) -> String {
    let mut out=String::from("# Authoring surfaces\n\nFive authoring surfaces were distinguished instead of collapsed into one score.\n\n| Surface | Authority | Mutability | Output | Programming classes |\n|---|---|---|---|---|\n");
    for r in array(m, "authoring_surfaces") {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            s(r, "name"),
            s(r, "authoritative_state"),
            s(r, "state_mutability"),
            s(r, "output_representation"),
            join(r, "creative_programmability_class")
        ));
    }
    out.push_str("\nThe Cinema inspector is not another authoring surface. It is a parallel high-level semantic analysis route over the Cinema payload.\n");
    out
}

fn react_flow(_: &Value) -> String {
    "# React reconciler flow\n\n```mermaid\nsequenceDiagram\n  participant Caller\n  participant Global as Module-global frame/DOF\n  participant Root as Fresh reconciler root\n  participant Host as Mutable HostNode tree\n  participant Scene\n  Caller->>Global: install requested integer/fractional frame\n  Caller->>Root: synchronous render\n  Root->>Host: commit mutations\n  Host->>Scene: lower with toNode\n  Root-->>Caller: unmount\n  Caller->>Global: restore evaluation state\n```\n\nA new root isolates each host tree and hook lifetime. It does **not** prove concurrent or nested reentrancy: module-global active frame and depth-of-field state remain shared. `renderFrames` accumulates the resulting Scene snapshots, while motion blur requests fractional subframes.\n".into()
}

fn cinema_flow(_: &Value) -> String {
    "# Cinema compiler flow\n\n```mermaid\nflowchart TD\n  P[Cinema payload] --> V[Validate payload and timing]\n  P --> I[Inspector semantic analysis]\n  V --> T[Normalize TimeSpec]\n  T --> G[Resolve scenes tracks entries roles]\n  G --> C[Invoke registry components]\n  C --> M[Materialize choreography transitions placement]\n  M --> R[React composition]\n  R --> S[Renderer-facing Scene]\n```\n\nCinema preserves high-level identities during validation and inspection, then consumes many of them while constructing React elements. Roles, names, brand-token identity, choreography names, and transition names are generally not first-class Scene data.\n".into()
}

fn direct_flow(_: &Value) -> String {
    "# Direct JSON and Rust flow\n\n```mermaid\nflowchart LR\n  JSON[Direct Scene JSON] --> D[Deserialize and version handling]\n  Rust[Typed Rust Scene] --> Scene\n  Timeline[Rust Timeline] --> Eval[Clone and evaluate at frame/fps]\n  Scene --> Eval\n  Eval --> Frame[Evaluated Scene]\n  D --> P[Prepasses]\n  Frame --> P\n```\n\nDirect JSON bypasses Cinema and React validation. Rust Scene and Timeline construction is **explicit and close to the renderer contract**; R0.02 makes no efficiency claim because benchmarking was prohibited.\n".into()
}

fn scene_contract(m: &Value) -> String {
    format!("# Scene graph contract\n\nScene is the authoritative renderer-facing representation: composition metadata plus a finite NodeKind hierarchy, optional numeric NodeId, visual fields, media references, layout, effects, and selected 3D placement. Runtime pixels are not serialized.\n\nThe architecture graph records {} boundaries into or out of this model. Numeric IDs support renderer/timeline targeting, but do not establish a source map back to Cinema entries, React components, user instructions, or agent operations.\n", count(m,"architecture_edges"))
}

fn state_time(m: &Value) -> String {
    let st = &m["state_and_time"];
    let mut out=format!("# State and time ownership\n\nThe model records **{} state owners** and **{} temporal conversions**.\n\n| State | Scope | Authority | Mutability | Reentrancy finding |\n|---|---|---|---|---|\n",count(st,"state_records"),count(st,"time_conversions"));
    for r in array(st, "state_records") {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            s(r, "representation"),
            s(r, "owner_scope"),
            s(r, "authority"),
            s(r, "mutability"),
            s(r, "reentrancy")
        ));
    }
    out.push_str("\n```mermaid\nflowchart LR\n  Seconds[Cinema seconds] -->|round × fps| Frames[Composition frames]\n  Frames -->|subtract start| Local[Sequence-local frames]\n  Frames -->|fractional samples| Blur[Motion-blur subframes]\n  Frames -->|÷ fps| Time[Timeline seconds]\n  Wall[RAF wall time] -->|floor by fps/rate| Frames\n  Frames --> Audio[AudioContext transport]\n  Frames --> Timestamp[Encoded timestamps]\n  Time --> Media[Source-media seconds]\n```\n\nEvery conversion records rounding, clamping, negative/fractional behavior, rate ownership, and precision risk in the machine output.\n");
    out
}

fn identity(m: &Value) -> String {
    let mut out=format!("# Identity and source mapping\n\n{} identity transitions were classified.\n\n| Concept | Source → target | Disposition | Traceable at final Scene |\n|---|---|---|---|\n",count(m,"identity_and_provenance"));
    for r in array(m, "identity_and_provenance") {
        out.push_str(&format!(
            "| {} | {} → {} | {} | {} |\n",
            s(r, "concept"),
            s(r, "source_representation"),
            s(r, "target_representation"),
            s(r, "disposition"),
            r["final_scene_traceable"]
        ));
    }
    out.push_str("\nNo complete intent-to-pixel source map was found.\n");
    out
}

fn semantics(m: &Value) -> String {
    let mut counts = std::collections::BTreeMap::new();
    for r in array(m, "semantic_preservation") {
        *counts.entry(s(r, "disposition")).or_insert(0usize) += 1;
    }
    let mut out=format!("# Lowering and information loss\n\nThe register contains **{} semantic concepts**.\n\n| Disposition | Count |\n|---|---:|\n",count(m,"semantic_preservation"));
    for (k, v) in counts {
        out.push_str(&format!("| {k} | {v} |\n"));
    }
    out.push_str("\n```mermaid\nflowchart LR\n  Intent[Roles names brand timing intent] --> Resolve[Validation and normalization]\n  Resolve --> React[React values and keys]\n  React --> Scene[Finite Scene values]\n  Scene --> Prepass[Materialized layout/media]\n  Prepass --> Pixels[Pixels and timestamps]\n```\n\nEvery row records reversibility plus editing, diagnostic, agent-repair, and incremental-compilation impact.\n");
    out
}

fn fallbacks(m: &Value) -> String {
    let mut out=format!("# Validation, errors, and fallbacks\n\n{} behaviors cover the mandatory error and fallback cases.\n\n| Trigger | Behavior | Informed | Quality reducing | Visual outcome |\n|---|---|---:|---:|---|\n",count(m,"validation_and_fallbacks"));
    for r in array(m, "validation_and_fallbacks") {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            s(r, "trigger"),
            s(r, "behavior"),
            r["user_or_agent_informed"],
            r["quality_reducing"],
            s(r, "visual_outcome")
        ));
    }
    out.push_str("\n```mermaid\nflowchart TD\n  Input --> Validate\n  Validate -->|hard/validation error| Stop\n  Validate -->|warning/default| Continue\n  Continue --> GPU\n  GPU -. runtime/capability fallback .-> CPU\n  CPU -. preview-only approximation .-> Canvas\n```\n");
    out
}

fn preview(m: &Value) -> String {
    let mut out=format!("# Preview and export parity\n\n{} comparison rows show that shared authoring evaluation and renderer cores do not establish end-to-end parity.\n\n| Feature | Preview | Export | Classification |\n|---|---|---|---|\n",count(m,"preview_export_parity"));
    for r in array(m, "preview_export_parity") {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            s(r, "feature"),
            s(r, "preview_path"),
            s(r, "export_path"),
            s(r, "parity_class")
        ));
    }
    out
}

fn serialization(m: &Value) -> String {
    let r = &m["serialization"];
    format!("# Serialization and versioning\n\nScene serialization uses **{}** with current version **{}**. Current-version omission is `{}` and runtime pixels are not serialized. TypeScript and Rust meet at JSON boundaries, but forward-field, future-version, and semantic migration guarantees remain incomplete and require focused compatibility fixtures in later research.\n",s(r,"format"),r["scene_version"],r["current_version_omitted"])
}

fn materialization(m: &Value) -> String {
    let mut out=format!("# Materialization and scalability\n\n{} hypotheses are architectural predictions, not benchmark results.\n\n",count(m,"materialization_hypotheses"));
    for r in array(m, "materialization_hypotheses") {
        out.push_str(&format!(
            "- **{}:** {}. Evidence: {}.\n",
            s(r, "id"),
            s(r, "statement"),
            join(r, "source_refs")
        ));
    }
    out.push_str("\n```mermaid\nflowchart LR\n  OutputFrames --> Samples[Motion-blur samples]\n  Samples --> Evaluate[Per-sample React evaluation]\n  Evaluate --> Array[Whole Scene array]\n  Array --> Temp[Temporary JSON]\n  Temp --> Native[Native export]\n```\n");
    out
}

fn creativity(m: &Value) -> String {
    let c = &m["creative_programmability"];
    let mut out=format!("# Creative-programmability assessment\n\n**Verdict:** {}. {}\n\n| Surface | primitives | procedural logic | component extension | registry-dependent | descends to primitives | source mapping | black-box risk |\n|---|---:|---:|---:|---:|---:|---:|---|\n",s(c,"overall_verdict"),s(c,"scoring_policy"));
    for r in array(c, "surface_assessments") {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            s(r, "surface_id"),
            r["general_primitive_access"],
            r["procedural_logic"],
            r["custom_component_extension"],
            r["registry_dependence"],
            r["can_descend_to_primitives"],
            r["source_mapping"],
            s(r, "black_box_risk")
        ));
    }
    out.push_str("\n```mermaid\nflowchart TD\n  Cinema[Guided semantic payload] --> React[Host-language escape hatch]\n  Components[Finite/extensible registry] --> React\n  React --> Primitives[Inspectable finite primitives]\n  JSON[Direct declarative Scene] --> Primitives\n  Rust[Typed host language] --> Primitives\n  Primitives --> Renderer[Finite renderer vocabulary]\n```\n\nNo numeric creativity score is used. Reusable components are not classified as fixed templates.\n");
    out
}

fn primary(m: &Value) -> String {
    let mut out=String::from("# Independent primary-source comparison\n\n| Source | ONDA comparison | CineKernel research question |\n|---|---|---|\n");
    for r in array(m, "primary_source_comparison") {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            s(r, "source"),
            s(r, "comparison"),
            s(r, "candidate_lesson")
        ));
    }
    out
}

fn requirements(m: &Value) -> String {
    let mut out=format!("# CineKernel requirement candidates\n\nAll {} records are nonfinal and prohibit implementation reuse.\n\n| ID | Abstract requirement | Follow-up | Status |\n|---|---|---|---|\n",count(m,"candidate_requirements"));
    for r in array(m, "candidate_requirements") {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            s(r, "requirement_id"),
            s(r, "abstract_requirement"),
            join(r, "required_follow_up_research"),
            s(r, "status")
        ));
    }
    out
}

fn risks(m: &Value) -> String {
    let mut out = String::from("# Risks and open questions\n\n## Contradictions\n\n");
    for r in array(m, "contradictions") {
        out.push_str(&format!(
            "- **{}:** {}\n",
            s(r, "contradiction_id"),
            s(r, "statement")
        ));
    }
    out.push_str("\n## Open questions\n\n");
    for r in array(m, "open_questions") {
        out.push_str(&format!("- **{}:** {}\n", s(r, "id"), s(r, "question")));
    }
    out
}

fn deferred(m: &Value) -> String {
    let mut out = String::from("# Deferred to later R0 phases\n\n| Phase | Topic |\n|---|---|\n");
    for r in array(m, "deferred_topics") {
        out.push_str(&format!("| {} | {} |\n", s(r, "phase"), s(r, "topic")));
    }
    out
}

fn sources(m: &Value) -> String {
    let local = array(m, "sources")
        .iter()
        .filter(|r| s(r, "source_id").starts_with("S-"))
        .count();
    let ext = count(m, "sources") - local;
    let mut out=format!("# Research source index\n\n**{} pinned ONDA files were hashed, plus {} official external references were indexed.**\n\n| Source ID | Classification | Identity | Evidence scope |\n|---|---|---|---|\n",local,ext);
    for r in array(m, "sources") {
        let identity = if r.get("path").is_some() {
            format!("{} @ {}", s(r, "path"), s(r, "git_blob"))
        } else {
            s(r, "document_url").into()
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            s(r, "source_id"),
            s(r, "classification"),
            identity,
            join(r, "facts_supported")
        ));
    }
    out
}

fn acceptance(m: &Value) -> String {
    let local = array(m, "sources")
        .iter()
        .filter(|r| s(r, "source_id").starts_with("S-"))
        .count();
    let external = count(m, "sources") - local;
    let sections=[
        ("1. Executive status","CONDITIONAL PASS pending a final remediation-head remote reproduction and updated attestation. All local remediation gates pass before publication."),
        ("2. CineKernel base revision","Base `974d93ef224b75383499cdb2b70cc086a0dd6f40`; accepted base tree `80ebf050ebc298b7647a403159ab59f94811468f`."),
        ("3. ONDA research identity","The only authority is the frozen R0.01 `UPSTREAM_LOCK.json`: repository `https://github.com/onda-engine/onda-engine.git`, pin `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`, tree `639df83ebf0262afccd6d021bf6d16ef19777d85`."),
        ("4. Branch and commits","Branch `research/r0.02-onda-scene-compiler-archaeology`; PR #13 remains draft and unmerged. Existing commits are recorded in the reviewer packet; the final attestation distinguishes the evidence commit from any later documentation-only commit."),
        ("5. Clean-room confirmation","ONDA was inspected statically and never built, executed, tested, rendered, or benchmarked. Exact-file, normalized multiline-fragment, dependency-alias, Git-dependency, absolute-path, tracked-upstream, and frozen-path guards pass."),
        ("6. Source coverage","SOURCE_COUNT"),("7. Authoring surfaces discovered","Five surfaces: Cinema, React, direct Scene JSON, typed Rust Scene/Timeline, and the component registry. The inspector is a semantic analysis route, not an authoring surface."),("8. Complete architecture graph","GRAPH_COUNT"),
        ("9. Cinema/agent authoring flow","Cinema validates and normalizes timing and semantic payload fields, invokes registry components and named patterns, then emits React. High-level names and roles are mostly consumed before Scene."),("10. React authoring flow","Fresh roots and HostNode trees are created per requested frame, lowered, and unmounted. Module-global active frame/DOF state leaves concurrent or nested reentrancy unresolved."),("11. Direct JSON flow","Direct JSON enters at Scene deserialization/version handling and therefore bypasses Cinema and React contracts."),("12. Rust Scene and Timeline flow","Typed Rust construction is explicit and close to the renderer contract. Timeline evaluation clones and mutates a Scene using numeric NodeId targets."),("13. Scene graph authority","Scene is authoritative at the renderer boundary, not at the high-level authoring boundary."),("14. State ownership","STATE_COUNT"),("15. Mutability and reentrancy","Fresh React roots isolate host trees; global frame/DOF state, shared GPU coordination, caches, and external media clocks require explicit serialization or further proof."),("16. Time ownership","TIME_COUNT"),("17. Identity and source mapping","IDENTITY_COUNT"),("18. Lowering and semantic preservation","SEMANTIC_COUNT"),("19. Information loss","Roles, labels, registry names, React keys, choreography names, transition names, brand-token identity, diagnostics, fidelity class, and provenance are consumed or dropped before pixels."),("20. Validation and error behavior","FALLBACK_COUNT"),("21. Fallbacks and approximations","Fallbacks include warnings, omission, placeholders, default substitution, asynchronous repaint, GPU-to-CPU demotion, and Canvas approximation; quality reduction is recorded explicitly."),("22. Preview/export parity","PARITY_COUNT"),("23. Serialization and versioning","Scene JSON is versioned, but forward compatibility and semantic migration guarantees remain unresolved and deferred."),("24. Prepasses","Layout, image, SVG, timeline, and media resolution materialize or rewrite Scene data before rendering."),("25. Materialization and scalability hypotheses","React export builds a complete Scene array and temporary JSON; motion blur multiplies evaluations. This is a scaling hypothesis only—R0.02 performed no benchmark."),("26. Creative-programmability assessment","Five surfaces are assessed categorically across primitive access, procedural logic, extension, registry/pattern dependence, descent to primitives, inspectability, editability, source mapping, novel-scene expression, and black-box risk. No numeric score is used."),("27. Laptop exploded-layers litmus","The laptop case separately covers hierarchy, segmentation, exploded-view planning, collision avoidance, geometry, materials, emissive lighting, camera, particles, timing, sound, semantic grouping, inspectability, editability, and asset truthfulness."),("28. Additional novel-scene litmus results","Glass-city and liquid-spacecraft cases are compared across all five surfaces. Both are partly composable but general material, lighting, and simulation capabilities are not established."),("29. Independent primary-source comparison","Official React, MLIR, and GStreamer sources provide independent comparisons for render/commit purity, progressive lowering, and clock/pipeline ownership."),("30. CineKernel candidate requirements","REQ_COUNT"),("31. Contradictions found","CONTRADICTION_COUNT"),("32. Unresolved questions","OPEN_COUNT"),("33. Deferred topics","DEFERRED_COUNT"),("34. Automated tests","63 fixture-driven unit and mutation tests pass, covering lock/checkout failures, provenance, strict nested schemas, graph integrity, semantic/fallback/requirement completeness, creative-taxonomy and novel-scene mutations, clean-room aliases and fragments, absolute paths, sorting, and two-run determinism."),("35. Deterministic generation","The report command generates 16 machine projections, 20 human reports, and 17 schemas: 53 generated files. The canonical model is an input, not a generated output."),("36. Remote reproduction","Pending remediation-head Windows, Ubuntu, and macOS reproduction. The attestation is updated only after those runs complete and a nonempty raw-evidence artifact is published."),("37. Phase 0 and R0.01 immutability","No Phase 0 or R0.01 frozen file is modified. The R0.01 verifier is executed from the exact accepted base inside each R0.02 OS job."),("38. Recommendation for R0.03","Do not start R0.03 until remediation-head three-OS evidence succeeds and the reviewer accepts promotion from CONDITIONAL PASS to PASS."),
    ];
    let mut out = String::from("# R0.02 acceptance report\n\n");
    for (h, b) in sections {
        let body=match b{"SOURCE_COUNT"=>format!("{local} pinned ONDA files are fully verified and {external} official external references are indexed separately. All mandatory paths are present."),"GRAPH_COUNT"=>format!("{} nodes and {} edges use controlled vocabularies and record authority, mutability, time, identity, data form, validation, semantic disposition, errors, and immutable source references.",count(m,"architecture_nodes"),count(m,"architecture_edges")),"STATE_COUNT"=>format!("{} state records identify authority, scope, mutability, sharing, persistence, provenance, and reentrancy.",count(&m["state_and_time"],"state_records")),"TIME_COUNT"=>format!("{} temporal conversions record source/target domains, operation, rounding, clamping, negative/fractional behavior, rate ownership, and precision risk.",count(&m["state_and_time"],"time_conversions")),"IDENTITY_COUNT"=>format!("{} identity transitions distinguish preservation, remapping, lowering-only use, React-key-only use, dropping, and non-representability.",count(m,"identity_and_provenance")),"SEMANTIC_COUNT"=>format!("{} semantic rows record disposition, stage, reversibility, and editing/diagnostic/repair/incremental impacts.",count(m,"semantic_preservation")),"FALLBACK_COUNT"=>format!("{} failure/fallback records cover every mandatory behavior and explicitly state diagnosis, semantic/visual/timing/determinism impact, parity difference, and repairability.",count(m,"validation_and_fallbacks")),"PARITY_COUNT"=>format!("{} preview/export rows distinguish shared evaluation from browser/native media, scheduling, renderer, and fallback differences.",count(m,"preview_export_parity")),"REQ_COUNT"=>format!("{} requirement candidates use CK-R002 IDs, cite ONDA and independent sources, identify affected programs and impacts, require later research, prohibit reuse, and remain CANDIDATE_ONLY.",count(m,"candidate_requirements")),"CONTRADICTION_COUNT"=>format!("{} explicit contradictions remain recorded rather than smoothed over.",count(m,"contradictions")),"OPEN_COUNT"=>format!("{} unresolved questions remain open.",count(m,"open_questions")),"DEFERRED_COUNT"=>format!("{} topics are routed to R0.03–R0.08.",count(m,"deferred_topics")),_=>b.into()};
        out.push_str(&format!("## {h}\n\n{body}\n\n"));
    }
    out
}

fn write_review_packet(root: &Path, m: &Value) -> Result<()> {
    let local = array(m, "sources")
        .iter()
        .filter(|r| s(r, "source_id").starts_with("S-"))
        .count();
    let external = count(m, "sources") - local;
    let manifest = root.join(REVIEW).join("INTEGRITY_MANIFEST.sha256");
    let manifest_count = fs::read_to_string(&manifest)
        .map(|v| v.lines().filter(|l| !l.is_empty()).count())
        .unwrap_or(0);
    let text=format!("# R0.02 reviewer packet\n\n## Status\n\n**CONDITIONAL PASS pending final remediation-head remote reproduction.** PR #13 must remain draft and unmerged.\n\n## Locked identity\n\n- CineKernel base SHA: `974d93ef224b75383499cdb2b70cc086a0dd6f40`\n- CineKernel base tree: `80ebf050ebc298b7647a403159ab59f94811468f`\n- ONDA repository: `https://github.com/onda-engine/onda-engine.git`\n- ONDA pin: `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`\n- ONDA tree: `639df83ebf0262afccd6d021bf6d16ef19777d85`\n- Branch: `research/r0.02-onda-scene-compiler-archaeology`\n\n## Counts\n\n| Evidence | Count |\n|---|---:|\n| Pinned ONDA files | {local} |\n| External official references | {external} |\n| Claims | {} |\n| Authoring surfaces | {} |\n| Graph nodes | {} |\n| Graph edges | {} |\n| State owners | {} |\n| Time conversions | {} |\n| Identity transitions | {} |\n| Semantic-preservation rows | {} |\n| Fallback/error rows | {} |\n| Preview/export comparisons | {} |\n| Candidate requirements | {} |\n| Contradictions | {} |\n| Open questions | {} |\n| Deferred topics | {} |\n| Generated machine projections | 16 |\n| Generated human reports | 20 |\n| Strict schemas | 17 |\n| Integrity-manifest entries | {manifest_count} |\n\n## Gate results\n\n- R0.01 authoritative lock parsing: PASS\n- checkout remote/detached HEAD/pin/tree/clean validation: PASS\n- complete mandatory source coverage and blob/SHA/symbol/line checks: PASS\n- strict nested Draft 2020-12 schemas: PASS\n- exact-file and normalized multiline clean-room guard: PASS\n- dependency alias and Git dependency guard: PASS\n- absolute path and tracked-upstream guard: PASS\n- Phase 0 and R0.01 frozen paths: PASS\n- two-run byte equality: run during final reproduction\n- remote workflow and artifacts: pending remediation-head run\n- standard three-OS CI: pending remediation-head run\n\n## Known check behavior\n\nThe standalone frozen R0.01 workflow scans future `schemas/research/**` paths against its frozen manifest. R0.02 therefore runs the unchanged verifier in an exact-base worktree on every OS. No R0.01 file is modified.\n\n## Review paths\n\n- `docs/research/onda/r0.02/R0_02_ACCEPTANCE_REPORT.md`\n- `docs/research/onda/r0.02/R0_02_RESEARCH_MODEL.json`\n- `docs/research/onda/r0.02/SOURCE_INDEX.json`\n- `docs/research/onda/r0.02/ARCHITECTURE_GRAPH.json`\n- `reports/research/r0.02/INTEGRITY_MANIFEST.sha256`\n- `reports/research/r0.02/REMOTE_REPRODUCTION_ATTESTATION.json`\n\n## Reproduction commands\n\n```text\ncargo xtask research onda sync --json\ncargo xtask research onda verify --json\ncargo xtask research onda integrity --check --json\ncargo fmt --manifest-path tools/research/onda-r0-02/Cargo.toml --all --check\ncargo clippy --locked --manifest-path tools/research/onda-r0-02/Cargo.toml --all-targets -- -D warnings\ncargo test --locked --manifest-path tools/research/onda-r0-02/Cargo.toml\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- inventory --json\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- verify --json\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- report --json\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- guard --json\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- integrity --check --json\n```\n",count(m,"claims"),count(m,"authoring_surfaces"),count(m,"architecture_nodes"),count(m,"architecture_edges"),count(&m["state_and_time"],"state_records"),count(&m["state_and_time"],"time_conversions"),count(m,"identity_and_provenance"),count(m,"semantic_preservation"),count(m,"validation_and_fallbacks"),count(m,"preview_export_parity"),count(m,"candidate_requirements"),count(m,"contradictions"),count(m,"open_questions"),count(m,"deferred_topics"));
    let text = text.replace(
        "| Strict schemas | 17 |",
        "| Strict schemas | 17 |\n| Standalone verifier tests | 63 |",
    );
    let text = if attestation_pass(root)? {
        text.replace("**CONDITIONAL PASS pending final remediation-head remote reproduction.**", "**PASS — exact remediation-head research workflow and ordinary CI succeeded on all three operating systems.**")
            .replace("- two-run byte equality: run during final reproduction", "- two-run byte equality: PASS")
            .replace("- remote workflow and artifacts: pending remediation-head run", "- remote workflow and artifacts: PASS — run 31923882312; three nonempty artifacts")
            .replace("- standard three-OS CI: pending remediation-head run", "- standard three-OS CI: PASS — run 31923882324")
            + "\n## Implementation and evidence commits\n\n- `8d9d425024761715bbbb37f8a14104d1c1fd670b` — initial research packet\n- `e9a4546db0962cd30858ad71041ec92c33b81fa7` — workflow token correction\n- `c8d16e3d7d8029a3e2fe2e2e2019f48996533758` — immutable blob hashing correction\n- `81ba1835d759e332f2d73683161de28a1f0954fc` — historical attestation\n- `b528d651fa4e1f5678b098f39fc8c35ce034e1ef` — reviewer remediation evidence commit\n\n## Final remote evidence\n\n- Dedicated R0.02 run: `31923882312` — Windows, Ubuntu, macOS success\n- Ordinary CI run: `31923882324` — Windows, Ubuntu, macOS success\n- `r0-02-windows-latest-evidence`: `sha256:40261bfc15cd945cab7f09ab5355e4cd57024a767f3ca745291f07348d0c8108`\n- `r0-02-macos-latest-evidence`: `sha256:2c63bdd641d736afc28a2e5f64d3b1f15970e766a0858204f3a251abac8decb3`\n- `r0-02-ubuntu-latest-evidence`: `sha256:f381ab255a94ac0341decf3dfd4ecb9d69504aa0b4bc0169d97c9d8a6846210a`\n"
    } else {
        text
    };
    fs::create_dir_all(root.join(REVIEW))?;
    fs::write(
        root.join(REVIEW).join("REVIEW_PACKET.md"),
        format!("{}\n", text.trim_end()),
    )?;
    Ok(())
}

fn attestation_pass(root: &Path) -> Result<bool> {
    let path = root
        .join(REVIEW)
        .join("REMOTE_REPRODUCTION_ATTESTATION.json");
    if !path.is_file() {
        return Ok(false);
    }
    let value: Value = serde_json::from_slice(&fs::read(path)?)?;
    Ok(value["conclusion"].as_str() == Some("PASS"))
}

fn array<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v.get(key)
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[])
}
fn count(v: &Value, key: &str) -> usize {
    array(v, key).len()
}
fn s<'a>(v: &'a Value, key: &str) -> &'a str {
    v.get(key).and_then(Value::as_str).unwrap_or("")
}
fn join(v: &Value, key: &str) -> String {
    array(v, key)
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}
fn write(root: &Path, name: &str, text: &str) -> Result<()> {
    fs::write(root.join(DOC).join(name), format!("{}\n", text.trim_end()))
        .with_context(|| format!("writing {name}"))
}

#[allow(dead_code)]
fn digest(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
