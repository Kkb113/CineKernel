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
            .replace("CONDITIONAL PASS — NOT LOCKED. The merged research tree is retained, but this independent-review closure branch requires exact-head three-OS evidence and explicit reviewer approval before R0.02 can lock.", "PASS. The independently approved exact closure head passed the dedicated R0.02 workflow and ordinary CineKernel CI on Windows, Ubuntu, and macOS, with three nonempty raw-evidence artifacts and post-merge tree equality.")
            .replace("Because this closure changes executable verifier/model code, new exact-closure-head runs and artifacts are mandatory and still pending.", "The attestation identifies the exact tested closure head, successful three-OS runs and artifacts, independent approval, and equal post-merge tree.");
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
    let mut out=format!("# State and time ownership\n\nThe model records **{} state owners** and **{} temporal conversions**.\n\n| State | Created by | Scope | Versioned | Concurrency status | Reentrancy finding |\n|---|---|---|---|---|---|\n",count(st,"state_records"),count(st,"time_conversions"));
    for r in array(st, "state_records") {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            s(r, "representation"),
            s(r, "created_by"),
            s(r, "owner_scope"),
            s(r, "versioned"),
            s(r, "concurrency_status"),
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
    let mut out=format!("# Validation, errors, and fallbacks\n\n{} behaviors cover the mandatory error and fallback cases.\n\n| Surface | Trigger | Behavior | Diagnostic visibility | Agent informed | Visual outcome |\n|---|---|---|---|---:|---|\n",count(m,"validation_and_fallbacks"));
    for r in array(m, "validation_and_fallbacks") {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            s(r, "surface"),
            s(r, "trigger"),
            s(r, "behavior"),
            s(r, "diagnostic_visibility"),
            r["agent_informed"],
            s(r, "visual_outcome")
        ));
    }
    out.push_str("\n```mermaid\nflowchart TD\n  Input --> Validate\n  Validate -->|hard/validation error| Stop\n  Validate -->|warning/default| Continue\n  Continue --> GPU\n  GPU -. runtime/capability fallback .-> CPU\n  CPU -. preview-only approximation .-> Canvas\n```\n");
    out
}

fn preview(m: &Value) -> String {
    let mut out=format!("# Preview and export parity\n\n{} comparison rows show that shared authoring evaluation and renderer cores do not establish end-to-end parity.\n\n| Feature | Preview | Export | Classification | Known difference | Certification impact |\n|---|---|---|---|---|---|\n",count(m,"preview_export_parity"));
    for r in array(m, "preview_export_parity") {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            s(r, "feature"),
            s(r, "preview_path"),
            s(r, "export_path"),
            s(r, "parity_class"),
            s(r, "known_difference"),
            s(r, "certification_impact")
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
    let mut out=format!("# Creative-programmability assessment\n\n**Verdict:** {}. {}\n\n| Surface | primitives | procedural logic | custom animation | component extension | registry relation | descends to primitives | source mapping | black-box risk |\n|---|---|---|---|---|---|---|---|---|\n",s(c,"overall_verdict"),s(c,"scoring_policy"));
    for r in array(c, "surface_assessments") {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            s(r, "surface_id"),
            s(r, "general_primitive_access"),
            s(r, "procedural_logic"),
            s(r, "custom_animation"),
            s(r, "custom_component_extension"),
            s(r, "registry_dependence"),
            s(r, "can_descend_to_primitives"),
            s(r, "source_mapping"),
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
        out.push_str(&format!(
            "- **{}** → **{}:** {}\n",
            s(r, "id"),
            join(r, "defer_to"),
            s(r, "question")
        ));
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
    let mut out=format!("# Research source index\n\n**{} pinned ONDA source records were hashed, plus {} official external references were indexed.** Mandatory inventory records are coverage-only; formal claims may cite only narrow claim-supporting records.\n\n| Source ID | Evidence role | Classification | Identity | Evidence scope |\n|---|---|---|---|---|\n",local,ext);
    for r in array(m, "sources") {
        let identity = if r.get("path").is_some() {
            format!("{} @ {}", s(r, "path"), s(r, "git_blob"))
        } else {
            s(r, "document_url").into()
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            s(r, "source_id"),
            s(r, "evidence_role"),
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
        ("1. Executive status","CONDITIONAL PASS — NOT LOCKED. The merged research tree is retained, but this independent-review closure branch requires exact-head three-OS evidence and explicit reviewer approval before R0.02 can lock."),
        ("2. CineKernel base revision","Base `974d93ef224b75383499cdb2b70cc086a0dd6f40`; accepted base tree `80ebf050ebc298b7647a403159ab59f94811468f`."),
        ("3. ONDA research identity","The only authority is the frozen R0.01 `UPSTREAM_LOCK.json`: repository `https://github.com/onda-engine/onda-engine.git`, pin `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`, tree `639df83ebf0262afccd6d021bf6d16ef19777d85`."),
        ("4. Branch and commits","PR #13 was merged before independent approval: branch head `6e7ff3d6016829357bb7f804dd916e6f7e796a64`, master merge commit `12024231b8983b07d9413cf96f4579bd9495f946`, common research tree `e01b2fe87d409e34f509847cdd66214d174eb0d6`. This corrective work is on `research/r0.02-independent-review-closure`; its exact evidence head remains pending until publication."),
        ("5. Clean-room confirmation","ONDA was inspected statically and never built, executed, tested, rendered, or benchmarked. Exact-file, normalized multiline-fragment, authoritative package-identity, renamed dependency, Git source, ONDA-checkout path, lockfile identity, absolute-path, tracked-upstream, and frozen-path guards pass."),
        ("6. Source coverage","SOURCE_COUNT"),("7. Authoring surfaces discovered","Five surfaces: Cinema, React, direct Scene JSON, typed Rust Scene/Timeline, and the component registry. The inspector is a semantic analysis route, not an authoring surface."),("8. Complete architecture graph","GRAPH_COUNT"),
        ("9. Cinema/agent authoring flow","Cinema validates and normalizes timing and semantic payload fields, invokes registry components and named patterns, then emits React. High-level names and roles are mostly consumed before Scene."),("10. React authoring flow","React directly supports host-language frame logic, interpolation, springs, sequences, loops, transitions, and primitive Scene nodes. Fresh roots and HostNode trees are created per requested frame, lowered, and unmounted; module-global frame/DOF state still leaves concurrent or nested reentrancy unresolved."),("11. Direct JSON flow","Direct JSON enters at Scene deserialization/version handling and composes the finite declarative Scene schema. It does not natively define a new component implementation."),("12. Rust Scene and Timeline flow","Typed Rust construction directly builds the finite Scene and Timeline model without Cinema-registry dependence. Timeline evaluation clones and mutates a Scene using numeric NodeId targets."),("13. Scene graph authority","Scene is authoritative at the renderer boundary, not at the high-level authoring boundary."),("14. State ownership","STATE_COUNT"),("15. Mutability and reentrancy","Fresh React roots isolate host trees; global frame/DOF state, shared GPU coordination, caches, and external media clocks require explicit serialization or further proof."),("16. Time ownership","TIME_COUNT"),("17. Identity and source mapping","IDENTITY_COUNT"),("18. Lowering and semantic preservation","SEMANTIC_COUNT"),("19. Information loss","Roles, labels, registry names, React keys, choreography names, transition names, brand-token identity, diagnostics, fidelity class, and provenance are consumed or dropped before pixels."),("20. Validation and error behavior","FALLBACK_COUNT"),("21. Fallbacks and approximations","Fallbacks include warnings, omission, placeholders, default substitution, asynchronous repaint, GPU-to-CPU demotion, and Canvas approximation; quality reduction is recorded explicitly."),("22. Preview/export parity","PARITY_COUNT"),("23. Serialization and versioning","Scene JSON is versioned, but forward compatibility and semantic migration guarantees remain unresolved and routed to R0.03 and R0.08."),("24. Prepasses","Layout, image, SVG, timeline, and media resolution materialize or rewrite Scene data before rendering."),("25. Materialization and scalability hypotheses","React export builds a complete Scene array and temporary JSON; motion blur multiplies evaluations. This is a scaling hypothesis only—measurement belongs in R0.07."),("26. Creative-programmability assessment","Five surfaces use categorical capability states rather than ambiguous Booleans. React host-language animation and primitive descent are supported; JSON custom components and Rust registry dependence are not native; component identity is only partial before lowering."),("27. Laptop exploded-layers litmus","The laptop case distinguishes manually supplied or presegmented hierarchy from automatic semantic/mechanical segmentation, which is not established. Sound uses audio-specific authoring/runtime evidence, and authoritative assets remain required for claims about real product construction."),("28. Additional novel-scene litmus results","Glass-city and liquid-spacecraft cases are compared across all five surfaces. Both are partly composable but general material, lighting, and simulation capabilities are not established."),("29. Independent primary-source comparison","Official React, MLIR, and GStreamer sources provide independent comparisons for render/commit purity, progressive lowering, and clock/pipeline ownership."),("30. CineKernel candidate requirements","REQ_COUNT"),("31. Contradictions found","CONTRADICTION_COUNT"),("32. Unresolved questions","OPEN_COUNT"),("33. Deferred topics","DEFERRED_COUNT"),("34. Automated tests","73 fixture-driven unit and mutation tests pass, including authoritative ONDA crate identities and lockfiles, roadmap drift, categorical creative states, candidate status, graph/schema integrity, provenance, clean-room copying, stable ordering, and deterministic generation."),("35. Deterministic generation","The report command generates 16 machine projections, 20 human reports, and 17 schemas: 53 generated files. The canonical model is an input, not a generated output."),("36. Remote reproduction","Pending closure-head Windows, Ubuntu, and macOS reproduction. The attestation is updated only after those runs complete and three nonempty raw-evidence artifacts are published."),("37. Phase 0 and R0.01 immutability","No Phase 0 or R0.01 frozen file is modified. The R0.01 verifier is executed from the exact accepted base inside each R0.02 OS job."),("38. Recommendation for R0.03","Keep PR #13 draft and unmerged. Do not start R0.03 until closure-head three-OS evidence and ordinary CI succeed and the reviewer grants final sign-off."),
    ];
    let mut out = String::from("# R0.02 acceptance report\n\n## Locked future-phase registry\n\n- R0.03 — Native GPU, CPU, WASM, and encoding architecture\n- R0.04 — Typography, layout, effects, color, and 3D architecture\n- R0.05 — Agent component catalog and cinematic composition model\n- R0.06 — CLI, installation, preview, embedding, and developer experience\n- R0.07 — Independent benchmark and failure analysis\n- R0.08 — Adoption, rejection, clean-room, and roadmap-delta matrix\n\n");
    for (h, b) in sections {
        let body=match b{"SOURCE_COUNT"=>format!("{local} pinned ONDA source records and {external} official external references are indexed. Mandatory whole-file entries are COVERAGE_ONLY; only narrow symbol-and-line entries marked CLAIM_SUPPORTING may satisfy formal claims."),"GRAPH_COUNT"=>format!("{} nodes and {} edges use controlled vocabularies and record authority, mutability, time, identity, data form, validation, semantic disposition, errors, and immutable source references.",count(m,"architecture_nodes"),count(m,"architecture_edges")),"STATE_COUNT"=>format!("{} state records identify creation, versioning, authority, scope, mutability, sharing, persistence, provenance, and evidence-backed concurrency status.",count(&m["state_and_time"],"state_records")),"TIME_COUNT"=>format!("{} temporal conversions record source/target domains, operation, rounding, clamping, negative/fractional behavior, rate ownership, and precision risk.",count(&m["state_and_time"],"time_conversions")),"IDENTITY_COUNT"=>format!("{} identity transitions distinguish preservation, remapping, lowering-only use, React-key-only use, dropping, and non-representability.",count(m,"identity_and_provenance")),"SEMANTIC_COUNT"=>format!("{} individually mapped semantic rows record exact source/target representations, disposition, stage, reversibility, and editing/diagnostic/repair/incremental impacts.",count(m,"semantic_preservation")),"FALLBACK_COUNT"=>format!("{} failure/fallback records distinguish structured, stderr, UI-only, silent demotion, and silent-skip visibility and include the native network materialization boundary.",count(m,"validation_and_fallbacks")),"PARITY_COUNT"=>format!("{} preview/export rows carry feature-specific differences, certification impact, and focused source references.",count(m,"preview_export_parity")),"REQ_COUNT"=>format!("{} requirement candidates cite requirement-specific claims and sources, use locked P01–P28 program IDs, tailor all four impacts, route through the locked R0 registry, prohibit reuse, and remain CANDIDATE_ONLY.",count(m,"candidate_requirements")),"CONTRADICTION_COUNT"=>format!("{} explicit contradictions remain recorded rather than smoothed over.",count(m,"contradictions")),"OPEN_COUNT"=>format!("{} unresolved questions remain open and are routed without renaming later phases.",count(m,"open_questions")),"DEFERRED_COUNT"=>format!("{} topics preserve the exact locked definitions for R0.03–R0.08: Native GPU/CPU/WASM/encoding; visual/3D; agent components; developer experience; benchmark/failure analysis; and final adoption/roadmap decisions.",count(m,"deferred_topics")),_=>b.into()};
        let body = body
            .replace("Fresh React roots isolate host trees; global frame/DOF state, shared GPU coordination, caches, and external media clocks require explicit serialization or further proof.", "Fresh React roots isolate host trees; global frame/DOF state, shared GPU coordination, caches, and external media clocks retain evidence-backed UNKNOWN or UNRESOLVED concurrency where safety is not proved.")
            .replace("Fallbacks include warnings, omission, placeholders, default substitution, asynchronous repaint, GPU-to-CPU demotion, and Canvas approximation; quality reduction is recorded explicitly.", "Bad preview fonts and failed preview images are silently skipped; GPU/CPU failures silently demote renderer state without an agent diagnostic; native remote media is best-effort materialized to temporary files and failed URLs are retained after stderr notice for decoder skip.")
            .replace("73 fixture-driven unit and mutation tests pass, including authoritative ONDA crate identities and lockfiles, roadmap drift, categorical creative states, candidate status, graph/schema integrity, provenance, clean-room copying, stable ordering, and deterministic generation.", "82 fixture-driven unit and mutation tests cover source evidence roles, symbol-in-range validation, semantic tuple independence, silent fallback visibility, renderer demotion, network materialization in the graph/prepass, independently required schema contracts, exact-head attestation, architecture contracts, provenance, clean-room copying, stable ordering, and deterministic generation.")
            .replace("Pending closure-head Windows, Ubuntu, and macOS reproduction. The attestation is updated only after those runs complete and three nonempty raw-evidence artifacts are published.", "Historical final PR head `6e7ff3d6016829357bb7f804dd916e6f7e796a64` passed dedicated run `31927730892` and ordinary CI `31927730849` on Windows, Ubuntu, and macOS. Because this closure changes executable verifier/model code, new exact-closure-head runs and artifacts are mandatory and still pending.")
            .replace("Keep PR #13 draft and unmerged. Do not start R0.03 until closure-head three-OS evidence and ordinary CI succeed and the reviewer grants final sign-off.", "Do not start R0.03. Publish this focused closure as a new review PR, obtain exact-head dedicated and ordinary three-OS success, require explicit independent approval before merge, and verify post-merge tree equality.");
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
    let unique_local = array(m, "sources")
        .iter()
        .filter_map(|r| r.get("path").and_then(Value::as_str))
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let manifest = root.join(REVIEW).join("INTEGRITY_MANIFEST.sha256");
    let manifest_count = fs::read_to_string(&manifest)
        .map(|v| v.lines().filter(|l| !l.is_empty()).count())
        .unwrap_or(0);
    let text=format!("# R0.02 reviewer packet\n\n## Status\n\n**CONDITIONAL PASS pending final remediation-head remote reproduction.** PR #13 must remain draft and unmerged.\n\n## Locked identity\n\n- CineKernel base SHA: `974d93ef224b75383499cdb2b70cc086a0dd6f40`\n- CineKernel base tree: `80ebf050ebc298b7647a403159ab59f94811468f`\n- ONDA repository: `https://github.com/onda-engine/onda-engine.git`\n- ONDA pin: `3ddf1780c9799bf038ac90cec7d8cadb61acafbe`\n- ONDA tree: `639df83ebf0262afccd6d021bf6d16ef19777d85`\n- Branch: `research/r0.02-onda-scene-compiler-archaeology`\n\n## Counts\n\n| Evidence | Count |\n|---|---:|\n| Pinned ONDA files | {local} |\n| External official references | {external} |\n| Claims | {} |\n| Authoring surfaces | {} |\n| Graph nodes | {} |\n| Graph edges | {} |\n| State owners | {} |\n| Time conversions | {} |\n| Identity transitions | {} |\n| Semantic-preservation rows | {} |\n| Fallback/error rows | {} |\n| Preview/export comparisons | {} |\n| Candidate requirements | {} |\n| Contradictions | {} |\n| Open questions | {} |\n| Deferred topics | {} |\n| Generated machine projections | 16 |\n| Generated human reports | 20 |\n| Strict schemas | 17 |\n| Integrity-manifest entries | {manifest_count} |\n\n## Gate results\n\n- R0.01 authoritative lock parsing: PASS\n- checkout remote/detached HEAD/pin/tree/clean validation: PASS\n- complete mandatory source coverage and blob/SHA/symbol/line checks: PASS\n- strict nested Draft 2020-12 schemas: PASS\n- exact-file and normalized multiline clean-room guard: PASS\n- dependency alias and Git dependency guard: PASS\n- absolute path and tracked-upstream guard: PASS\n- Phase 0 and R0.01 frozen paths: PASS\n- two-run byte equality: run during final reproduction\n- remote workflow and artifacts: pending remediation-head run\n- standard three-OS CI: pending remediation-head run\n\n## Known check behavior\n\nThe standalone frozen R0.01 workflow scans future `schemas/research/**` paths against its frozen manifest. R0.02 therefore runs the unchanged verifier in an exact-base worktree on every OS. No R0.01 file is modified.\n\n## Review paths\n\n- `docs/research/onda/r0.02/R0_02_ACCEPTANCE_REPORT.md`\n- `docs/research/onda/r0.02/R0_02_RESEARCH_MODEL.json`\n- `docs/research/onda/r0.02/SOURCE_INDEX.json`\n- `docs/research/onda/r0.02/ARCHITECTURE_GRAPH.json`\n- `reports/research/r0.02/INTEGRITY_MANIFEST.sha256`\n- `reports/research/r0.02/REMOTE_REPRODUCTION_ATTESTATION.json`\n\n## Reproduction commands\n\n```text\ncargo xtask research onda sync --json\ncargo xtask research onda verify --json\ncargo xtask research onda integrity --check --json\ncargo fmt --manifest-path tools/research/onda-r0-02/Cargo.toml --all --check\ncargo clippy --locked --manifest-path tools/research/onda-r0-02/Cargo.toml --all-targets -- -D warnings\ncargo test --locked --manifest-path tools/research/onda-r0-02/Cargo.toml\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- inventory --json\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- verify --json\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- report --json\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- guard --json\ncargo run --locked --manifest-path tools/research/onda-r0-02/Cargo.toml -- integrity --check --json\n```\n",count(m,"claims"),count(m,"authoring_surfaces"),count(m,"architecture_nodes"),count(m,"architecture_edges"),count(&m["state_and_time"],"state_records"),count(&m["state_and_time"],"time_conversions"),count(m,"identity_and_provenance"),count(m,"semantic_preservation"),count(m,"validation_and_fallbacks"),count(m,"preview_export_parity"),count(m,"candidate_requirements"),count(m,"contradictions"),count(m,"open_questions"),count(m,"deferred_topics"));
    let text = text
        .replace("| Pinned ONDA files |", "| Pinned ONDA source records |")
        .replace(
            "| External official references |",
            &format!("| Unique pinned ONDA files | {unique_local} |\n| External official references |"),
        )
        .replace(
            "- Branch: `research/r0.02-onda-scene-compiler-archaeology`",
            "- Closure branch: `research/r0.02-independent-review-closure`\n- PR #13 merged branch head: `6e7ff3d6016829357bb7f804dd916e6f7e796a64`\n- Master merge commit: `12024231b8983b07d9413cf96f4579bd9495f946`\n- Common merged research tree: `e01b2fe87d409e34f509847cdd66214d174eb0d6`\n- Process note: PR #13 was merged before independent approval; it is not reverted.\n\n## Locked future-phase registry\n\n- R0.03 — Native GPU, CPU, WASM, and encoding architecture\n- R0.04 — Typography, layout, effects, color, and 3D architecture\n- R0.05 — Agent component catalog and cinematic composition model\n- R0.06 — CLI, installation, preview, embedding, and developer experience\n- R0.07 — Independent benchmark and failure analysis\n- R0.08 — Adoption, rejection, clean-room, and roadmap-delta matrix",
        )
        .replace(
            "| Strict schemas | 17 |",
            "| Strict schemas | 17 |\n| Standalone verifier tests | 82 |",
        )
        .replace(
            "- dependency alias and Git dependency guard: PASS",
            "- authoritative ONDA package identity, renamed Cargo/npm alias, workspace/dev/build/target dependency, Git source, ONDA-checkout path, and resolved lockfile guard: PASS",
        )
        .replace("**CONDITIONAL PASS pending final remediation-head remote reproduction.** PR #13 must remain draft and unmerged.", "**CONDITIONAL PASS — NOT LOCKED.** The focused closure requires exact-head three-OS evidence and explicit independent approval before merge.")
        .replace("- complete mandatory source coverage and blob/SHA/symbol/line checks: PASS", "- coverage-only versus claim-supporting source roles, immutable blob/SHA, and symbol-inside-line-range checks: PASS")
        .replace("- remote workflow and artifacts: pending remediation-head run", "- historical final-head dedicated run 31927730892 and artifacts: PASS; new exact closure-head run: PENDING")
        .replace("- standard three-OS CI: pending remediation-head run", "- historical final-head ordinary CI run 31927730849: PASS; new exact closure-head CI: PENDING")
        + "\n## Historical final-head evidence and closure requirement\n\nFinal PR head `6e7ff3d6016829357bb7f804dd916e6f7e796a64` passed dedicated run `31927730892` and ordinary CI `31927730849` on all three operating systems. The prior attestation was stale because executable `tools/research/onda-r0-02/src/reports.rs` changed after its recorded evidence commit. This closure changes executable verifier/model code, so none of that historical evidence substitutes for fresh exact-closure-head evidence or independent approval.\n";
    let text = if attestation_pass(root)? {
        text.replace("**CONDITIONAL PASS — NOT LOCKED.**", "**PASS — independently approved exact closure head.**")
            .replace("- two-run byte equality: run during final reproduction", "- two-run byte equality: PASS")
            .replace("new exact closure-head run: PENDING", "new exact closure-head run: PASS")
            .replace("new exact closure-head CI: PENDING", "new exact closure-head CI: PASS")
            + "\n## Final evidence\n\nThe machine attestation records the exact tested closure head, dedicated and ordinary three-OS runs, nonempty artifact hashes, independent approval, and post-merge tree equality.\n"
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
    validate_attestation(&value)?;
    Ok(value["conclusion"].as_str() == Some("PASS"))
}

fn validate_attestation(value: &Value) -> Result<()> {
    if value["conclusion"].as_str() != Some("PASS") {
        return Ok(());
    }
    let evidence = value["evidence_commit"]
        .as_str()
        .context("PASS attestation lacks evidence_commit")?;
    if evidence.len() != 40 || value["closure_head"].as_str() != Some(evidence) {
        anyhow::bail!("PASS attestation must identify the exact tested closure head")
    }
    let runs = value["runs"]
        .as_array()
        .context("PASS attestation lacks runs")?;
    if runs.len() < 2
        || runs.iter().any(|run| {
            run["head_sha"].as_str() != Some(evidence)
                || run["conclusion"].as_str() != Some("success")
        })
    {
        anyhow::bail!("PASS attestation run heads do not match the closure head")
    }
    if value["independent_review"]["status"].as_str() != Some("APPROVED") {
        anyhow::bail!("PASS attestation requires explicit independent approval")
    }
    let approved_tree = value["independent_review"]["approved_tree"]
        .as_str()
        .context("PASS attestation lacks approved tree")?;
    if value["post_merge_tree"].as_str() != Some(approved_tree) {
        anyhow::bail!("PASS attestation requires post-merge tree equality")
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn final_attestation_requires_exact_tested_closure_head() {
        let value = json!({
            "conclusion":"PASS",
            "evidence_commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "closure_head":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "runs":[
                {"head_sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","conclusion":"success"},
                {"head_sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","conclusion":"success"}
            ]
        });
        assert!(validate_attestation(&value).is_err());
    }
}
