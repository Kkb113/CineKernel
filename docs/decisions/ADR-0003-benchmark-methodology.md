# ADR-0003: benchmark methodology

Status: Accepted

## Context

Phase 0 requires reproducible evidence without committing future VideoIR or forcing one engine into every renderer role.

## Decision

Measure identical deterministic local fixtures, retain failures, report null for unavailable phases, and verify decoded artifacts.

## Consequences

The decision is reviewable, testable, and may be superseded only with new measured evidence. No Phase 0 result silently becomes a permanent format contract.
