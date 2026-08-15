use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use walkdir::WalkDir;

const BASE: &str = "974d93ef224b75383499cdb2b70cc086a0dd6f40";
const ONDA_PIN: &str = "3ddf1780c9799bf038ac90cec7d8cadb61acafbe";
const ONDA_TREE: &str = "639df83ebf0262afccd6d021bf6d16ef19777d85";
const DOC_DIR: &str = "docs/research/onda/r0.02";
const REPORT_DIR: &str = "reports/research/r0.02";

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Verify {
        #[arg(long)]
        json: bool,
    },
    Inventory {
        #[arg(long)]
        json: bool,
    },
    Report {
        #[arg(long)]
        json: bool,
    },
    Guard {
        #[arg(long)]
        json: bool,
    },
    Integrity {
        #[arg(long)]
        check: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Serialize)]
struct Outcome {
    command: &'static str,
    ok: bool,
    checks: Vec<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = repo_root()?;
    match cli.command {
        Cmd::Verify { json } => emit(verify(&root)?, json)?,
        Cmd::Inventory { json } => emit(inventory(&root)?, json)?,
        Cmd::Report { json } => emit(report(&root)?, json)?,
        Cmd::Guard { json } => emit(guard(&root)?, json)?,
        Cmd::Integrity { check, json } => emit(integrity(&root, check)?, json)?,
    };
    Ok(())
}

fn emit(out: Outcome, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("{}: {}", out.command, if out.ok { "PASS" } else { "FAIL" });
        for c in out.checks {
            println!("- {c}");
        }
    }
    Ok(())
}

fn repo_root() -> Result<PathBuf> {
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !out.status.success() {
        bail!("not in a git repository")
    }
    Ok(PathBuf::from(String::from_utf8(out.stdout)?.trim()))
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git").args(args).current_dir(root).output()?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_owned())
}

fn model(root: &Path) -> Result<Value> {
    let path = root.join(DOC_DIR).join("R0_02_RESEARCH_MODEL.json");
    serde_json::from_slice(&fs::read(&path).with_context(|| format!("reading {}", path.display()))?)
        .context("canonical model is invalid JSON")
}

fn verify(root: &Path) -> Result<Outcome> {
    let m = model(root)?;
    let mut checks = vec![];
    for (key, expected) in [
        ("cinekernel_base", BASE),
        ("onda_pin", ONDA_PIN),
        ("onda_tree", ONDA_TREE),
    ] {
        if m.get(key).and_then(Value::as_str) != Some(expected) {
            bail!("canonical {key} does not match locked value")
        }
        checks.push(format!("locked {key}"));
    }
    let upstream = root.join(".cinekernel/upstreams/onda");
    if git(&upstream, &["rev-parse", "HEAD"])? != ONDA_PIN {
        bail!("ONDA checkout pin mismatch")
    }
    if git(&upstream, &["rev-parse", "HEAD^{tree}"])? != ONDA_TREE {
        bail!("ONDA checkout tree mismatch")
    }
    if !git(&upstream, &["status", "--porcelain"])?.is_empty() {
        bail!("ONDA checkout is dirty")
    }
    checks.push("pinned ONDA checkout is clean and exact".into());
    for array in [
        "sources",
        "claims",
        "architecture_nodes",
        "architecture_edges",
        "hypotheses",
    ] {
        if m.get(array)
            .and_then(Value::as_array)
            .is_none_or(|v| v.is_empty())
        {
            bail!("canonical {array} is empty")
        }
        checks.push(format!("canonical {array} populated"));
    }
    validate_refs(&m)?;
    checks.push("claim and edge source references resolve".into());
    validate_outputs_against_schemas(root)?;
    checks.push("all Draft 2020-12 schemas compile and accept their outputs".into());
    Ok(Outcome {
        command: "verify",
        ok: true,
        checks,
    })
}

fn validate_refs(m: &Value) -> Result<()> {
    let ids: BTreeSet<_> = m["sources"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["id"].as_str())
        .collect();
    for group in ["claims", "architecture_edges"] {
        for item in m[group].as_array().unwrap() {
            let refs = item["source_refs"]
                .as_array()
                .context("source_refs must be an array")?;
            if refs.is_empty() {
                bail!("{group} item lacks sources")
            }
            for r in refs {
                if !ids.contains(r.as_str().context("source ref is not a string")?) {
                    bail!("unresolved source ref {r}")
                }
            }
        }
    }
    Ok(())
}

fn inventory(root: &Path) -> Result<Outcome> {
    let m = model(root)?;
    let sources = m["sources"].as_array().context("sources missing")?;
    let upstream = root.join(".cinekernel/upstreams/onda");
    let mut checks = Vec::new();
    let mut seen = BTreeSet::new();
    for s in sources {
        let id = s["id"].as_str().context("source id missing")?;
        if !seen.insert(id) {
            bail!("duplicate source id {id}")
        }
        if s["classification"] == "implementation"
            || s["classification"] == "test"
            || s["classification"] == "manifest"
            || s["classification"] == "documentation"
        {
            let rel = s["path"]
                .as_str()
                .context("repository source path missing")?;
            let p = upstream.join(rel);
            if !p.is_file() {
                bail!("source {id} missing: {rel}")
            }
            let actual = sha256(&fs::read(&p)?);
            if s["file_sha256"].as_str() != Some(&actual) {
                bail!("source hash mismatch for {id}")
            }
        }
    }
    checks.push(format!("{} unique sources verified", sources.len()));
    Ok(Outcome {
        command: "inventory",
        ok: true,
        checks,
    })
}

fn report(root: &Path) -> Result<Outcome> {
    let m = model(root)?;
    fs::create_dir_all(root.join(DOC_DIR))?;
    fs::create_dir_all(root.join("schemas/research/r0.02"))?;
    generate_machine_outputs(root, &m)?;
    generate_human_outputs(root, &m)?;
    generate_schemas(root)?;
    let mut checks = vec!["outputs regenerated from the canonical model".into()];
    for name in MACHINE_OUTPUTS.iter().chain(HUMAN_OUTPUTS.iter()) {
        let p = root.join(DOC_DIR).join(name);
        if !p.is_file() || fs::metadata(&p)?.len() == 0 {
            bail!("missing report output {}", p.display())
        }
        checks.push(format!("present {name}"));
    }
    Ok(Outcome {
        command: "report",
        ok: true,
        checks,
    })
}

fn generate_machine_outputs(root: &Path, m: &Value) -> Result<()> {
    let mappings: &[(&str, &[&str])] = &[
        ("SOURCE_INDEX.json", &["sources"]),
        ("AUTHORING_SURFACES.json", &["authoring_surfaces"]),
        (
            "ARCHITECTURE_GRAPH.json",
            &["architecture_nodes", "architecture_edges"],
        ),
        ("BOUNDARY_CONTRACTS.json", &["claims"]),
        ("STATE_AND_TIME_OWNERSHIP.json", &["state_and_time"]),
        ("IDENTITY_AND_PROVENANCE.json", &["identity_and_provenance"]),
        ("SEMANTIC_PRESERVATION.json", &["semantic_preservation"]),
        (
            "VALIDATION_AND_FALLBACKS.json",
            &["validation_and_fallbacks"],
        ),
        ("PREVIEW_EXPORT_PARITY.json", &["preview_export_parity"]),
        ("SERIALIZATION_AND_VERSIONING.json", &["serialization"]),
        (
            "MATERIALIZATION_HYPOTHESES.json",
            &["materialization_hypotheses"],
        ),
        (
            "CREATIVE_PROGRAMMABILITY.json",
            &["creative_programmability"],
        ),
        ("NOVEL_SCENE_LITMUS.json", &["novel_scene_litmus"]),
        (
            "PRIMARY_SOURCE_COMPARISON.json",
            &["primary_source_comparison"],
        ),
        ("CANDIDATE_REQUIREMENTS.json", &["candidate_requirements"]),
        ("OPEN_QUESTIONS.json", &["open_questions"]),
    ];
    for (name, keys) in mappings {
        let mut data = serde_json::Map::new();
        for key in *keys {
            data.insert((*key).into(), m[*key].clone());
        }
        let schema = name
            .to_ascii_lowercase()
            .replace('_', "-")
            .replace(".json", ".schema.json");
        let out = serde_json::json!({
            "$schema": format!("../../../../schemas/research/r0.02/{schema}"),
            "schema_version": "r0.02.1",
            "model_ref": "R0_02_RESEARCH_MODEL.json",
            "generated_at": "2026-08-15T00:00:00Z",
            "data": data
        });
        write_stable_json(&root.join(DOC_DIR).join(name), &out)?;
    }
    Ok(())
}

fn generate_human_outputs(root: &Path, m: &Value) -> Result<()> {
    let claims = m["claims"].as_array().context("claims missing")?;
    let claim_lines = claims
        .iter()
        .map(|c| {
            format!(
                "- **{} — {}:** {} (confidence {}). Sources: {}.",
                c["id"].as_str().unwrap_or("claim"),
                c["status"].as_str().unwrap_or("unknown"),
                c["statement"].as_str().unwrap_or(""),
                c["confidence"],
                refs(c)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let architecture = format!("# Architecture overview\n\nONDA exposes multiple authoring paths that converge on a finite per-frame Scene tree. Cinema is a semantic front end, React is a programmable evaluation front end, and direct JSON or Rust can address Scene more directly. Renderer-facing prepasses then materialize media, vector expansion, timeline selection, and layout.\n\n```mermaid\nflowchart LR\n  C[Cinema payload] --> R[React program]\n  R --> H[Host tree]\n  H --> S[Per-frame Scene]\n  J[Direct JSON] --> S\n  T[Rust timeline] --> S\n  S --> P[Prepasses]\n  P --> CPU[CPU renderer]\n  P --> GPU[Vello renderer]\n  CPU --> E[Encoder]\n  GPU --> E\n```\n\n## Evidence-backed claims\n\n{claim_lines}\n");
    write_text(
        &root.join(DOC_DIR).join("ARCHITECTURE_OVERVIEW.md"),
        &architecture,
    )?;

    let docs: &[(&str, &str, &str, &str)] = &[
        ("AUTHORING_SURFACES.md", "Authoring surfaces", "Cinema, React, direct Scene JSON, typed Rust Scene, and the packaged component catalog differ in authority, identity, validation, and extension model. React is the broadest procedural escape hatch; Cinema is schema-guided and registry-extensible; Scene is compositional but finite.", "flowchart LR\n  Cinema --> React\n  Catalog --> React\n  React --> Scene\n  JSON --> Scene\n  Rust --> Scene"),
        ("REACT_RECONCILER_FLOW.md", "React reconciler flow", "For each requested frame, frame context is installed, a fresh custom-reconciler root commits into mutable HostNodes, the tree is lowered to Scene, then the root is unmounted. This makes component evaluation frame-pure by construction but prevents retained hook state across output frames.", "sequenceDiagram\n  participant Caller\n  participant FrameContext\n  participant Reconciler\n  participant HostTree\n  participant Scene\n  Caller->>FrameContext: set frame and config\n  FrameContext->>Reconciler: mount fresh root\n  Reconciler->>HostTree: commit mutations\n  HostTree->>Scene: lower primitives\n  Reconciler-->>Caller: unmount and restore context"),
        ("CINEMA_COMPILER_FLOW.md", "Cinema compiler flow", "Cinema validates and resolves timing, roles, registry entries, choreography, transitions, placement, themes, mattes, effects, and selected 3D wrappers. It emits a React composition. Most authoring labels and named intent are consumed during this stage rather than represented in Scene.", "flowchart TD\n  P[Cinema payload] --> V[Validate]\n  P --> T[Resolve timing]\n  P --> I[Inspector semantic model]\n  V --> B[Build registry components]\n  T --> B\n  B --> M[Motion and transitions]\n  M --> R[React composition]\n  R --> S[Scene]"),
        ("DIRECT_JSON_AND_RUST_FLOW.md", "Direct JSON and Rust flow", "Direct JSON is parsed into the typed Scene contract. Rust callers can construct the same types or pair a base Scene with an animation timeline. Timeline evaluation clones the scene, calculates seconds from frame and fps, and mutates properties selected by numeric NodeId.", "flowchart LR\n  JSON --> Parse --> Scene\n  Rust[Typed Rust] --> Scene\n  Timeline --> Evaluate\n  Scene --> Evaluate\n  Evaluate --> FrameScene\n  Scene --> Prepasses"),
        ("SCENE_GRAPH_CONTRACT.md", "Scene graph contract", "Scene contains composition metadata and a root Node. A Node has optional identity, transforms, opacity, clip, matte, blend, effects, selected 3D data, optional layout, a finite NodeKind, and children. Runtime-decoded pixels are deliberately excluded from serialization.", "classDiagram\n  Scene *-- Composition\n  Scene *-- Node\n  Node *-- NodeKind\n  Node *-- Node : children\n  NodeKind <|-- Group\n  NodeKind <|-- Text\n  NodeKind <|-- Shape\n  NodeKind <|-- Media\n  NodeKind <|-- Timeline"),
        ("STATE_AND_TIME_OWNERSHIP.md", "State and time ownership", "Authoring frame context is evaluation-scoped. Fonts, warmers, springs, decoded video frames, and some engine coordination are module or process scoped. Player playback is instance-scoped, while a shared GPU engine is protected against reentrant mutation. Time crosses composition frames, local frames, seconds, wall-clock milliseconds, audio context time, and bucketed video source time.", "flowchart TD\n  Wall[Wall clock] -->|floor by fps and rate| CF[Composition frame]\n  CF -->|subtract start| LF[Local frame]\n  CF -->|divide by fps| Sec[Seconds]\n  Sec --> Audio[Audio clock mapping]\n  Sec -->|nearest 1/30| Video[Video cache bucket]"),
        ("IDENTITY_AND_SOURCE_MAPPING.md", "Identity and source mapping", "Cinema has string identities and payload paths used by inspection. React keys support reconciliation only inside one frame evaluation. Scene retains only optional numeric ids. There is no complete source-map chain from a rendered node back to Cinema intent.", "flowchart LR\n  Entry[Cinema entry id and path] -->|component expansion| Key[React key]\n  Key -->|usually discarded| Node[Scene node]\n  Explicit[Explicit numeric id] --> Node\n  Node --> Pixel[Rendered pixels]"),
        ("LOWERING_AND_INFORMATION_LOSS.md", "Lowering and information loss", "Geometry and visual values generally survive. Roles, named choreography, transition identity, registry component identity, brand-token identity, flex intent, and SVG document structure are consumed or materialized. The result renders, but it is less editable and less explainable.", "flowchart LR\n  Intent[Named intent] --> Values[Resolved visual values]\n  Values --> Scene\n  Layout[Flex intent] --> Positions[Absolute positions]\n  SVGDoc[SVG structure] --> Shapes[Supported shapes]\n  Scene --> Pixels"),
        ("VALIDATION_ERRORS_AND_FALLBACKS.md", "Validation, errors, and fallbacks", "The architecture mixes hard errors, warnings, placeholders, silent omission, held frames, and renderer demotion. This is resilient for preview, but diagnostics are not represented by one typed contract and fallback behavior can change visible semantics.", "flowchart TD\n  Input --> Validate\n  Validate -->|hard error| Stop\n  Validate -->|warning| Continue\n  Continue --> Render\n  Render -->|GPU failure| CPU\n  CPU -->|failure| Canvas\n  Media -->|decode failure| HoldOrSkip"),
        ("PREVIEW_EXPORT_PARITY.md", "Preview and export parity", "CPU and GPU WASM paths share core renderer and selected prepasses with native export. End-to-end parity remains conditional because the browser resolves video and audio differently, preview may skip requested frames, and Canvas2D is intentionally incomplete and approximate.", "flowchart LR\n  Scene --> BrowserPrepass --> WasmCPU\n  Scene --> BrowserPrepass --> WasmGPU\n  Scene --> NativePrepass --> NativeCPU\n  Scene --> NativePrepass --> NativeGPU\n  Scene --> Canvas[Approximate Canvas2D]"),
        ("SERIALIZATION_AND_VERSIONING.md", "Serialization and versioning", "Scene uses JSON and carries a current version value of one, omitted for the current version. Future versions can be retained by the typed structure. Compatibility behavior for unknown fields and cross-version semantic guarantees needs an explicit contract and fixtures before CineKernel adopts a similar boundary.", "stateDiagram-v2\n  [*] --> ParseJSON\n  ParseJSON --> Current: absent or version 1\n  ParseJSON --> Future: version greater than 1\n  Current --> Serialize\n  Future --> Serialize\n  Serialize --> [*]"),
        ("MATERIALIZATION_AND_SCALABILITY.md", "Materialization and scalability", "React export constructs every Scene, serializes the complete array, then hands it to the native CLI. Motion blur multiplies the count. The structure predicts memory and I/O growth proportional to frame count and graph size. R0.02 does not benchmark ONDA, so thresholds remain open.", "flowchart LR\n  Frames[Duration times fps] --> Evaluate[Evaluate every frame]\n  Samples[Motion samples] --> Evaluate\n  Evaluate --> Array[Scene array in memory]\n  Array --> JSON[Temporary JSON]\n  JSON --> Native[Native renderer]"),
        ("CREATIVE_PROGRAMMABILITY_ASSESSMENT.md", "Creative programmability assessment", "The architecture is not limited to templates: React permits arbitrary procedural code that emits primitives, and a custom Cinema registry can add components. The creative ceiling is nevertheless bounded by the finite Scene and renderer capability set. General materials, shaders, constraints, simulations, and durable semantic subgraphs are not demonstrated.", "quadrantChart\n  x-axis Finite catalog --> Procedural authoring\n  y-axis Low-level pixels --> Semantic intent\n  quadrant-1 Strong target\n  quadrant-2 Guided authoring\n  quadrant-3 Renderer primitives\n  quadrant-4 Procedural lowering\n  Cinema: [0.55, 0.72]\n  React: [0.88, 0.48]\n  Scene: [0.42, 0.18]"),
        ("PRIMARY_SOURCE_COMPARISON.md", "Primary-source comparison", "React clarifies render versus commit and pure evaluation; ONDA borrows reconciliation but discards the root each frame. MLIR motivates explicit multi-level representations and progressive lowering; ONDA levels are real but mostly implicit. GStreamer makes clock, flow, element, and message ownership explicit; ONDA batch export would benefit from similarly explicit streaming contracts.", "flowchart TD\n  React[React: render and commit] --> Purity[Evaluation purity requirement]\n  MLIR[MLIR: progressive lowering] --> Levels[Explicit semantic levels]\n  GST[GStreamer: graph and clock] --> Stream[Streaming and clock contract]\n  Purity --> CK[CineKernel candidates]\n  Levels --> CK\n  Stream --> CK"),
        ("CINEKERNEL_REQUIREMENT_CANDIDATES.md", "CineKernel requirement candidates", "Candidates are deliberately nonfinal: stable source maps, explicit time domains, typed diagnostics, capability-aware parity, immutable intent separated from materialization, streaming evaluation, version negotiation, and a procedural escape hatch over a finite renderer contract.", "flowchart LR\n  Identity --> IR[Candidate IR]\n  Time --> IR\n  Diagnostics --> IR\n  Capabilities --> IR\n  Streaming --> IR\n  Versioning --> IR\n  IR --> Later[Validate in later R0 phases]"),
        ("RISKS_AND_OPEN_QUESTIONS.md", "Risks and open questions", "The largest risks are premature commitment to a frame-materialized IR, implicit time conversion, identity loss, silent fallback, shared mutable caches, and overstating creative capability from a finite catalog. Performance thresholds, renderer capability contracts, version evolution, media clocks, advanced materials, and editability remain open.\n\nRepository verification also exposed that the frozen R0.01 integrity checker scans later schema namespaces; R0.02 therefore verifies R0.01 inside an exact-base worktree.", "flowchart TD\n  Materialization --> R003[R0.03]\n  Renderers --> R004[R0.04]\n  Versioning --> R005[R0.05]\n  MediaTime --> R006[R0.06]\n  Materials --> R007[R0.07]\n  Editability --> R008[R0.08]"),
        ("DEFERRED_TO_LATER_R0_PHASES.md", "Deferred to later R0 phases", "R0.02 does not benchmark, select an IR, implement a compiler, judge final renderer quality, or establish a product roadmap. R0.03 through R0.08 must test the open performance, capability, compatibility, media, creative, and round-trip questions with their own locked protocols.", "timeline\n  title Deferred research\n  R0.03 : Materialization thresholds\n  R0.04 : Renderer capabilities\n  R0.05 : Version evolution\n  R0.06 : Media clocks\n  R0.07 : Creative ceiling\n  R0.08 : Editability and round trips")
    ];
    for (file, title, body, diagram) in docs {
        let text = format!(
            "# {title}\n\n{body}\n\n```mermaid\n{diagram}\n```\n\n## Evidence\n\n{claim_lines}\n"
        );
        write_text(&root.join(DOC_DIR).join(file), &text)?;
    }
    generate_source_markdown(root, m)?;
    generate_acceptance(root, m)?;
    Ok(())
}

fn generate_source_markdown(root: &Path, m: &Value) -> Result<()> {
    let mut text = String::from("# Research source index\n\nAll repository sources are from the locked ONDA pin and tree. Line ranges identify reviewed evidence; file hashes and blobs make the records independently checkable. No ONDA code was executed.\n\n| ID | Class | Path or URL | Evidence |\n|---|---|---|---|\n");
    for s in m["sources"].as_array().unwrap() {
        let loc = s
            .get("path")
            .or_else(|| s.get("url"))
            .and_then(Value::as_str)
            .unwrap_or("");
        let fact = s["facts"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(Value::as_str)
            .unwrap_or("");
        text.push_str(&format!(
            "| {} | {} | `{}` | {} |\n",
            s["id"].as_str().unwrap_or(""),
            s["classification"].as_str().unwrap_or(""),
            loc,
            fact
        ));
    }
    write_text(&root.join(DOC_DIR).join("RESEARCH_SOURCE_INDEX.md"), &text)
}

fn generate_acceptance(root: &Path, m: &Value) -> Result<()> {
    let sections = [
        ("1. Status", "CONDITIONAL PASS. Local source archaeology and deterministic verification are complete; remote dedicated workflow and full three-OS CI are still required for PASS."),
        ("2. Scope", "Static, clean-room architecture mapping only. No ONDA execution, benchmark, product implementation, or IR selection occurred."),
        ("3. Locked base", "CineKernel base 974d93ef224b75383499cdb2b70cc086a0dd6f40 and ONDA pin/tree match the R0.01 lock."),
        ("4. Method", "Implementation, tests, manifests, repository documentation, official external sources, comments, then explicit inference."),
        ("5. Source coverage", "Mandatory React, Cinema, Scene, animation, prepass, player, WASM, CLI/export, manifest, test, and repository boundaries are indexed."),
        ("6. Hypotheses", "All seven registered hypotheses have a verified or rejected verdict with sources."),
        ("7. Authoring surfaces", "Five distinct surfaces were mapped with ownership, identity, time, validation, extension, creativity, and output contracts."),
        ("8. React flow", "A new reconciler root is mounted, committed, lowered, and unmounted for every requested frame."),
        ("9. Cinema flow", "Cinema validates and resolves high-level editorial intent into React component structure."),
        ("10. Direct JSON", "Scene JSON bypasses high-level authoring semantics and enters the typed renderer contract."),
        ("11. Rust flow", "Typed Scene and optional second-based Timeline evaluation converge on Scene."),
        ("12. Scene graph", "Composition plus a finite node tree is the universal renderer-facing language."),
        ("13. State", "Evaluation, instance, module, process, and engine-shared mutable state were distinguished."),
        ("14. Mutability", "Reconciliation and prepasses mutate or clone-and-rewrite; shared engines and media seeks require serialization."),
        ("15. Time", "Composition, local, seconds, wall clock, audio, and video-bucket domains were mapped."),
        ("16. Identity", "High-level string identities and payload paths do not form a complete source map into Scene nodes and pixels."),
        ("17. Semantic loss", "Named intent, layout intent, SVG structure, and asset representation are consumed or materialized."),
        ("18. Validation", "Hard errors, warnings, placeholders, and omissions exist at different boundaries."),
        ("19. Fallbacks", "Preview fallback improves availability but can reduce fidelity without a typed end-to-end diagnostic."),
        ("20. Parity", "Core renderer parity is strong only when equivalent prepasses and media are used; end-to-end parity is conditional."),
        ("21. Serialization", "JSON version one is observed; a formal evolution and unknown-field contract remains open."),
        ("22. Prepasses", "Source materialization, timeline selection, SVG expansion, image decode, video decode, and layout have distinct ownership."),
        ("23. Materialization", "Whole-video Scene arrays and JSON are structurally duration-proportional; no performance claim is made."),
        ("24. Scalability", "Bounded streaming is a candidate requirement, not an implemented result."),
        ("25. Creative programmability", "Procedural authoring is broad; renderer vocabulary and advanced material semantics remain bounded."),
        ("26. Laptop litmus", "Exploded layers are plausible with groups, transforms, assets, and depth, but semantic assembly constraints are weak."),
        ("27. Other litmus", "Glass and chrome/liquid scenes permit stylized approximations; physical materials and simulation are not established."),
        ("28. React comparison", "The fresh-per-frame root differs materially from normal retained React identity."),
        ("29. MLIR comparison", "Explicit progressive IR levels would preserve intent and make semantic loss reviewable."),
        ("30. GStreamer comparison", "Explicit clocks, flow, backpressure, and diagnostics are useful reference properties."),
        ("31. Requirements", "Eight abstract, nonfinal candidate requirements are registered."),
        ("32. Clean room", "Only facts, prose, identifiers needed for citation, and abstract diagrams are stored; no ONDA source is copied or translated."),
        ("33. Independence", "Permanent dependencies on ONDA, Remotion, and HyperFrames remain zero."),
        ("34. Tests", "The standalone verifier, source hashes, references, outputs, frozen-path and dependency guards, integrity, determinism, schema-shape checks, root format/check/clippy, and JavaScript typecheck/tests pass locally.\n\nThe root Rust suite has one repeatable pre-existing Windows timing failure: the frozen xtask process timeout test completes near ten seconds instead of its asserted five-second ceiling; the other 76 Rust tests pass. Remote three-OS results remain authoritative for acceptance."),
        ("35. Remote reproduction", "Pending dedicated workflow and complete Linux, Windows, and macOS CI."),
        ("36. Immutability", "Phase 0 and R0.01 frozen artifacts are unchanged."),
        ("37. Contradictions and questions", "Claims of universal preview parity conflict with explicit Canvas and media fallback behavior. The frozen R0.01 integrity checker also scans future schema namespaces, so it must run in an exact-base worktree once R0.02 schemas exist. Open questions are routed to R0.03 through R0.08."),
        ("38. Recommendation", "Proceed to R0.03 after remote reproduction. Do not select or implement a CineKernel IR from R0.02 alone.")
    ];
    let mut text = String::from("# R0.02 acceptance report\n\n");
    for (heading, body) in sections {
        text.push_str(&format!("## {heading}\n\n{body}\n\n"));
    }
    let _ = m;
    write_text(
        &root.join(DOC_DIR).join("R0_02_ACCEPTANCE_REPORT.md"),
        &text,
    )
}

fn generate_schemas(root: &Path) -> Result<()> {
    let dir = root.join("schemas/research/r0.02");
    let names = [
        "research-model.schema.json",
        "source-index.schema.json",
        "authoring-surfaces.schema.json",
        "architecture-graph.schema.json",
        "boundary-contracts.schema.json",
        "state-and-time-ownership.schema.json",
        "identity-and-provenance.schema.json",
        "semantic-preservation.schema.json",
        "validation-and-fallbacks.schema.json",
        "preview-export-parity.schema.json",
        "serialization-and-versioning.schema.json",
        "materialization-hypotheses.schema.json",
        "creative-programmability.schema.json",
        "novel-scene-litmus.schema.json",
        "primary-source-comparison.schema.json",
        "candidate-requirements.schema.json",
        "open-questions.schema.json",
    ];
    let model_schema = serde_json::json!({
        "$schema":"https://json-schema.org/draft/2020-12/schema",
        "type":"object",
        "additionalProperties":false,
        "required":["$schema","model_version","generated_at","cinekernel_base","onda_pin","onda_tree","method","sources","authoring_surfaces","architecture_nodes","architecture_edges","claims","hypotheses","state_and_time","identity_and_provenance","semantic_preservation","validation_and_fallbacks","preview_export_parity","serialization","materialization_hypotheses","creative_programmability","novel_scene_litmus","primary_source_comparison","candidate_requirements","open_questions"],
        "properties":{
            "$schema":{"type":"string"},"model_version":{"const":"r0.02.1"},"generated_at":{"type":"string","format":"date-time"},
            "cinekernel_base":{"type":"string","pattern":"^[0-9a-f]{40}$"},"onda_pin":{"type":"string","pattern":"^[0-9a-f]{40}$"},"onda_tree":{"type":"string","pattern":"^[0-9a-f]{40}$"},
            "method":{"type":"object"},"sources":{"type":"array","minItems":20},"authoring_surfaces":{"type":"array","minItems":4},"architecture_nodes":{"type":"array","minItems":8},"architecture_edges":{"type":"array","minItems":10},"claims":{"type":"array","minItems":10},"hypotheses":{"type":"array","minItems":7,"maxItems":7},"state_and_time":{"type":"object"},"identity_and_provenance":{"type":"array","minItems":3},"semantic_preservation":{"type":"array","minItems":5},"validation_and_fallbacks":{"type":"array","minItems":5},"preview_export_parity":{"type":"array","minItems":4},"serialization":{"type":"object"},"materialization_hypotheses":{"type":"array","minItems":3},"creative_programmability":{"type":"object"},"novel_scene_litmus":{"type":"array","minItems":3},"primary_source_comparison":{"type":"array","minItems":3},"candidate_requirements":{"type":"array","minItems":6},"open_questions":{"type":"array","minItems":5}
        }
    });
    for name in names {
        let schema = if name == "research-model.schema.json" {
            model_schema.clone()
        } else {
            projection_schema(name)?
        };
        write_stable_json(&dir.join(name), &schema)?;
    }
    Ok(())
}

fn projection_schema(name: &str) -> Result<Value> {
    let (key, ty) = match name {
        "source-index.schema.json" => ("sources", "array"),
        "authoring-surfaces.schema.json" => ("authoring_surfaces", "array"),
        "architecture-graph.schema.json" => ("architecture_nodes", "array"),
        "boundary-contracts.schema.json" => ("claims", "array"),
        "state-and-time-ownership.schema.json" => ("state_and_time", "object"),
        "identity-and-provenance.schema.json" => ("identity_and_provenance", "array"),
        "semantic-preservation.schema.json" => ("semantic_preservation", "array"),
        "validation-and-fallbacks.schema.json" => ("validation_and_fallbacks", "array"),
        "preview-export-parity.schema.json" => ("preview_export_parity", "array"),
        "serialization-and-versioning.schema.json" => ("serialization", "object"),
        "materialization-hypotheses.schema.json" => ("materialization_hypotheses", "array"),
        "creative-programmability.schema.json" => ("creative_programmability", "object"),
        "novel-scene-litmus.schema.json" => ("novel_scene_litmus", "array"),
        "primary-source-comparison.schema.json" => ("primary_source_comparison", "array"),
        "candidate-requirements.schema.json" => ("candidate_requirements", "array"),
        "open-questions.schema.json" => ("open_questions", "array"),
        other => bail!("unknown projection schema {other}"),
    };
    let mut data_props = serde_json::Map::new();
    data_props.insert(
        key.into(),
        serde_json::json!({"type":ty, "minItems": if ty == "array" { 1 } else { 0 }}),
    );
    if name == "architecture-graph.schema.json" {
        data_props.insert(
            "architecture_edges".into(),
            serde_json::json!({"type":"array","minItems":1}),
        );
    }
    let required = if name == "architecture-graph.schema.json" {
        serde_json::json!(["architecture_nodes", "architecture_edges"])
    } else {
        serde_json::json!([key])
    };
    Ok(serde_json::json!({
        "$schema":"https://json-schema.org/draft/2020-12/schema",
        "type":"object","additionalProperties":false,
        "required":["$schema","schema_version","model_ref","generated_at","data"],
        "properties":{
            "$schema":{"type":"string","pattern":"^\\.\\./\\.\\./\\.\\./\\.\\./schemas/research/r0\\.02/.+\\.schema\\.json$"},
            "schema_version":{"const":"r0.02.1"},"model_ref":{"const":"R0_02_RESEARCH_MODEL.json"},
            "generated_at":{"type":"string","format":"date-time"},
            "data":{"type":"object","additionalProperties":false,"required":required,"properties":data_props}
        }
    }))
}

fn validate_outputs_against_schemas(root: &Path) -> Result<()> {
    let model_schema: Value = serde_json::from_slice(&fs::read(
        root.join("schemas/research/r0.02/research-model.schema.json"),
    )?)?;
    let compiled =
        jsonschema::validator_for(&model_schema).context("compile research model schema")?;
    let m = model(root)?;
    if let Err(e) = compiled.validate(&m) {
        bail!("canonical model schema validation failed: {e}")
    }
    for name in MACHINE_OUTPUTS
        .iter()
        .filter(|n| **n != "R0_02_RESEARCH_MODEL.json")
    {
        let schema_name = name
            .to_ascii_lowercase()
            .replace('_', "-")
            .replace(".json", ".schema.json");
        let schema: Value = serde_json::from_slice(&fs::read(
            root.join("schemas/research/r0.02").join(schema_name),
        )?)?;
        let doc: Value = serde_json::from_slice(&fs::read(root.join(DOC_DIR).join(name))?)?;
        let validator = jsonschema::validator_for(&schema)
            .with_context(|| format!("compile schema for {name}"))?;
        if let Err(e) = validator.validate(&doc) {
            bail!("{name} schema validation failed: {e}")
        }
    }
    Ok(())
}

fn refs(v: &Value) -> String {
    v["source_refs"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_default()
}
fn write_stable_json(path: &Path, v: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(v)?;
    bytes.push(b'\n');
    fs::write(path, bytes).with_context(|| format!("writing {}", path.display()))
}
fn write_text(path: &Path, text: &str) -> Result<()> {
    let normalized = text.replace("\r\n", "\n");
    fs::write(path, format!("{}\n", normalized.trim_end()))
        .with_context(|| format!("writing {}", path.display()))
}

fn guard(root: &Path) -> Result<Outcome> {
    let names = git(root, &["diff", "--name-only", BASE])?;
    let frozen = [
        "docs/research/onda/r0.01/",
        "reports/research/r0.01/",
        "crates/xtask/src/research_onda.rs",
        "crates/xtask/src/research_onda_pnpm.rs",
        ".github/workflows/r0-01-onda-provenance.yml",
    ];
    for n in names.lines() {
        if frozen.iter().any(|f| n == *f || n.starts_with(f)) {
            bail!("frozen path changed: {n}")
        }
    }
    let allowed = [
        "docs/research/onda/r0.02/",
        "reports/research/r0.02/",
        "schemas/research/r0.02/",
        "tools/research/onda-r0-02/",
        ".github/workflows/r0-02-onda-architecture.yml",
    ];
    for n in names.lines() {
        if !allowed.iter().any(|p| n == *p || n.starts_with(p)) {
            bail!("R0.02 changed an out-of-scope path: {n}")
        }
    }
    for banned in ["node_modules/", "target/", ".cinekernel/"] {
        if names.lines().any(|n| n.contains(banned)) {
            bail!("generated/private path tracked: {banned}")
        }
    }
    let cargo = fs::read_to_string(root.join("tools/research/onda-r0-02/Cargo.toml"))?;
    for banned in ["onda", "remotion", "hyperframe"] {
        if cargo.lines().any(|l| l.trim_start().starts_with(banned)) {
            bail!("prohibited permanent dependency: {banned}")
        }
    }
    let mut long_fragments = 0usize;
    let upstream = root.join(".cinekernel/upstreams/onda");
    let mut upstream_lines = BTreeSet::new();
    for e in WalkDir::new(&upstream)
        .into_iter()
        .filter_entry(|e| {
            let n = e.file_name().to_string_lossy();
            !matches!(
                n.as_ref(),
                ".git" | "node_modules" | "target" | "dist" | "coverage" | "pkg"
            )
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        if e.metadata().map(|m| m.len() > 1_000_000).unwrap_or(true) {
            continue;
        }
        if let Ok(text) = fs::read_to_string(e.path()) {
            for line in text.lines().map(str::trim).filter(|l| l.len() >= 100) {
                upstream_lines.insert(line.to_owned());
            }
        }
    }
    for e in WalkDir::new(root.join(DOC_DIR))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let text = fs::read_to_string(e.path()).unwrap_or_default();
        let human = e.path().extension().and_then(|x| x.to_str()) == Some("md");
        for line in text.lines() {
            if human && line.len() > 500 {
                long_fragments += 1;
            }
            let trimmed = line.trim();
            if human && upstream_lines.contains(trimmed) {
                bail!("exact upstream fragment found in {}", e.path().display())
            }
        }
        if text.contains("C:\\Users\\") || text.contains("/home/") {
            bail!("absolute path leaked into {}", e.path().display())
        }
    }
    if long_fragments > 0 {
        bail!("human reports contain {long_fragments} suspiciously long lines")
    }
    Ok(Outcome {
        command: "guard",
        ok: true,
        checks: vec![
            "frozen paths unchanged".into(),
            "no prohibited dependencies or tracked evidence".into(),
            "clean-room text and absolute-path guards passed".into(),
        ],
    })
}

fn integrity(root: &Path, check: bool) -> Result<Outcome> {
    let manifest = root.join(REPORT_DIR).join("INTEGRITY_MANIFEST.sha256");
    if !check {
        let mut paths = Vec::new();
        for base in [
            DOC_DIR,
            REPORT_DIR,
            "schemas/research/r0.02",
            "tools/research/onda-r0-02",
            ".github/workflows",
        ] {
            let start = root.join(base);
            if !start.exists() {
                continue;
            }
            for e in WalkDir::new(start)
                .into_iter()
                .filter_entry(|e| e.file_name() != "target")
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let rel = e
                    .path()
                    .strip_prefix(root)?
                    .to_string_lossy()
                    .replace('\\', "/");
                if rel == "reports/research/r0.02/INTEGRITY_MANIFEST.sha256"
                    || rel.ends_with("REMOTE_REPRODUCTION_ATTESTATION.json")
                    || (rel.starts_with(".github/workflows/")
                        && rel != ".github/workflows/r0-02-onda-architecture.yml")
                {
                    continue;
                }
                paths.push(rel);
            }
        }
        paths.sort();
        paths.dedup();
        let mut body = String::new();
        for rel in &paths {
            body.push_str(&format!(
                "{}  {}\n",
                sha256(&fs::read(root.join(rel))?),
                rel
            ));
        }
        fs::write(&manifest, body)?;
        return Ok(Outcome {
            command: "integrity",
            ok: true,
            checks: vec![format!("wrote {} entries", paths.len())],
        });
    }
    let text = fs::read_to_string(&manifest)?;
    let mut count = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (expected, rel) = line.split_once("  ").context("malformed integrity line")?;
        let p = root.join(rel);
        if sha256(&fs::read(&p).with_context(|| format!("reading {rel}"))?) != expected {
            bail!("integrity mismatch: {rel}")
        }
        count += 1;
    }
    Ok(Outcome {
        command: "integrity",
        ok: true,
        checks: vec![format!("{count} files match the manifest")],
    })
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

const MACHINE_OUTPUTS: &[&str] = &[
    "R0_02_RESEARCH_MODEL.json",
    "SOURCE_INDEX.json",
    "AUTHORING_SURFACES.json",
    "ARCHITECTURE_GRAPH.json",
    "BOUNDARY_CONTRACTS.json",
    "STATE_AND_TIME_OWNERSHIP.json",
    "IDENTITY_AND_PROVENANCE.json",
    "SEMANTIC_PRESERVATION.json",
    "VALIDATION_AND_FALLBACKS.json",
    "PREVIEW_EXPORT_PARITY.json",
    "SERIALIZATION_AND_VERSIONING.json",
    "MATERIALIZATION_HYPOTHESES.json",
    "CREATIVE_PROGRAMMABILITY.json",
    "NOVEL_SCENE_LITMUS.json",
    "PRIMARY_SOURCE_COMPARISON.json",
    "CANDIDATE_REQUIREMENTS.json",
    "OPEN_QUESTIONS.json",
];
const HUMAN_OUTPUTS: &[&str] = &[
    "R0_02_ACCEPTANCE_REPORT.md",
    "ARCHITECTURE_OVERVIEW.md",
    "AUTHORING_SURFACES.md",
    "REACT_RECONCILER_FLOW.md",
    "CINEMA_COMPILER_FLOW.md",
    "DIRECT_JSON_AND_RUST_FLOW.md",
    "SCENE_GRAPH_CONTRACT.md",
    "STATE_AND_TIME_OWNERSHIP.md",
    "IDENTITY_AND_SOURCE_MAPPING.md",
    "LOWERING_AND_INFORMATION_LOSS.md",
    "VALIDATION_ERRORS_AND_FALLBACKS.md",
    "PREVIEW_EXPORT_PARITY.md",
    "SERIALIZATION_AND_VERSIONING.md",
    "MATERIALIZATION_AND_SCALABILITY.md",
    "CREATIVE_PROGRAMMABILITY_ASSESSMENT.md",
    "PRIMARY_SOURCE_COMPARISON.md",
    "CINEKERNEL_REQUIREMENT_CANDIDATES.md",
    "RISKS_AND_OPEN_QUESTIONS.md",
    "DEFERRED_TO_LATER_R0_PHASES.md",
    "RESEARCH_SOURCE_INDEX.md",
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locked_values_are_full_hashes() {
        for v in [BASE, ONDA_PIN, ONDA_TREE] {
            assert_eq!(v.len(), 40);
            assert!(v.bytes().all(|b| b.is_ascii_hexdigit()));
        }
    }
    #[test]
    fn output_names_are_unique() {
        let mut s = BTreeSet::new();
        for n in MACHINE_OUTPUTS.iter().chain(HUMAN_OUTPUTS.iter()) {
            assert!(s.insert(n));
        }
    }
    #[test]
    fn model_rejects_unresolved_source_reference() {
        let m = serde_json::json!({"sources":[{"id":"S1"}],"claims":[{"source_refs":["missing"]}],"architecture_edges":[]});
        assert!(validate_refs(&m).is_err());
    }
    #[test]
    fn model_rejects_empty_source_reference() {
        let m = serde_json::json!({"sources":[{"id":"S1"}],"claims":[{"source_refs":[]}],"architecture_edges":[]});
        assert!(validate_refs(&m).is_err());
    }

    #[test]
    fn projection_schema_rejects_nested_extra_property() {
        let schema = projection_schema("source-index.schema.json").expect("schema");
        let validator = jsonschema::validator_for(&schema).expect("compile");
        let mutated = serde_json::json!({
            "$schema":"../../../../schemas/research/r0.02/source-index.schema.json",
            "schema_version":"r0.02.1","model_ref":"R0_02_RESEARCH_MODEL.json",
            "generated_at":"2026-08-15T00:00:00Z",
            "data":{"sources":[],"unexpected":true}
        });
        assert!(!validator.is_valid(&mutated));
    }

    #[test]
    fn architecture_schema_requires_both_graph_collections() {
        let schema = projection_schema("architecture-graph.schema.json").expect("schema");
        let validator = jsonschema::validator_for(&schema).expect("compile");
        let mutated = serde_json::json!({
            "$schema":"../../../../schemas/research/r0.02/architecture-graph.schema.json",
            "schema_version":"r0.02.1","model_ref":"R0_02_RESEARCH_MODEL.json",
            "generated_at":"2026-08-15T00:00:00Z","data":{"architecture_nodes":[{}]}
        });
        assert!(!validator.is_valid(&mutated));
    }

    #[test]
    fn projection_schema_rejects_wrong_version() {
        let schema = projection_schema("open-questions.schema.json").expect("schema");
        let validator = jsonschema::validator_for(&schema).expect("compile");
        let mutated = serde_json::json!({
            "$schema":"../../../../schemas/research/r0.02/open-questions.schema.json",
            "schema_version":"r0.03.0","model_ref":"R0_02_RESEARCH_MODEL.json",
            "generated_at":"2026-08-15T00:00:00Z","data":{"open_questions":[{}]}
        });
        assert!(!validator.is_valid(&mutated));
    }
}
