# ADR-0001: repository and language foundation

Status: Accepted

## Context

Phase 0 requires reproducible evidence without committing future VideoIR or forcing one engine into every renderer role.

## Decision

Use a Rust workspace for permanent cross-platform orchestration/native experiments and an exactly pinned pnpm workspace for browser baselines.

## Consequences

The decision is reviewable, testable, and may be superseded only with new measured evidence. No Phase 0 result silently becomes a permanent format contract.
