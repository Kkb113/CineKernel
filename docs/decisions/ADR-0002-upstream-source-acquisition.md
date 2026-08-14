# ADR-0002: upstream source acquisition

Status: Accepted

## Context

Phase 0 requires reproducible evidence without committing future VideoIR or forcing one engine into every renderer role.

## Decision

Use generated sparse detached checkouts at manifest-locked SHAs; no submodules or vendored monorepos.

## Consequences

The decision is reviewable, testable, and may be superseded only with new measured evidence. No Phase 0 result silently becomes a permanent format contract.
