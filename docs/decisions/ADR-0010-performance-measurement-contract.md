# ADR-0010: performance measurement contract

Status: Accepted

## Context

Phase 0 requires reproducible evidence without committing future VideoIR or forcing one engine into every renderer role.

## Decision

Separate prepare/compile/init/frame/encode/verify timings only when measured; never infer missing phases.

## Consequences

The decision is reviewable, testable, and may be superseded only with new measured evidence. No Phase 0 result silently becomes a permanent format contract.
