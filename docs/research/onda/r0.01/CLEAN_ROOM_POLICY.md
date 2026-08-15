# R0.01 clean-room policy

This policy governs R0.02–R0.08 and all later CineKernel implementation. It is an engineering information-flow boundary, not legal advice.

## Allowed research behavior

Subject to legal review, researchers may read public documentation; inspect architecture abstractly; cite immutable source locations; test external behavior; benchmark ONDA later as a black box; identify failure modes, concepts, risks and questions; study ONDA's open-source dependencies and standards independently; and write original CineKernel requirements.

## Prohibited implementation behavior

Do not copy ONDA source, tests, shaders, fixtures or schemas; vendor ONDA; translate functions line-by-line; rename identifiers and reuse implementation; ask an LLM to paraphrase ONDA implementation into CineKernel code; place ONDA source logic into implementation prompts; add ONDA crates/packages to CineKernel Core; expose ONDA scene types through VideoIR; or require ONDA at runtime.

## Information-flow rule

`ONDA source → research fact → abstract requirement/risk/question → independent primary-source research → CineKernel normative specification → original implementation`.

The direct flow `ONDA source → implementation prompt containing source logic → CineKernel code` is prohibited. Future prompts should cite standards, libraries and papers rather than ONDA implementation wherever possible. The `onda guard` command enforces dependency absence, untracked upstream input, exact-copy detection and Phase 0 immutability. The exact-copy guard is limited evidence, not proof against every form of derivation.
