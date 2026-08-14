# ADR-0005: reference and accelerated renderers

Status: Accepted

## Context

Phase 0 requires reproducible evidence without committing future VideoIR or forcing one engine into every renderer role.

## Decision

Maintain a slower reference path and separately measured accelerated paths; correctness gates optimization.

## Consequences

The decision is reviewable, testable, and may be superseded only with new measured evidence. No Phase 0 result silently becomes a permanent format contract.
