# Source-lineage policy

CineKernel classifies every external relationship as `original`, `adopted`,
`derived`, `reimplemented`, `wrapped`, `rejected`, or `third-party`. A copied or
materially derived file is not reviewable until `upstream-inventory.yaml` records
its repository, immutable commit, upstream paths, permission basis, original
license, modifications, local contracts, conformance tests, sync policy, and
owner. Generated sparse checkouts are research inputs, not vendored source.

Phase 0 prefers wrappers and independent reimplementation. Original CineKernel
code is Apache-2.0. Upstream copyright and attribution remain intact.

