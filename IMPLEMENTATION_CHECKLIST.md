# HDT Graph Full Implementation Checklist

This checklist enumerates the remaining deliverables and acceptance criteria required for a complete HDT implementation. Each line is intended to be checked off only when the criteria are fully met.

## 1) HDT invariants (spec + enforcement)
- [ ] Write the internal spec at `crates/rustdynconn-core/src/hdt/invariants.md` defining edge levels, tree/non-tree placement, and replacement search ordering.
- [ ] Add a single helper module that defines:
  - [ ] `edge_active_at_level(edge_level, i)`
  - [ ] `tree_edge_in_forest_at_level(tree_edge_level, i)`
  - [ ] `max_level(n)` and the growth rule
- [ ] Assert invariants in `debug_invariants` mode for every update and query path.

## 2) Euler Tour Tree (ETT) dynamic forest
- [ ] Implement ETT with splay trees and a node pool allocator (index-based handles).
- [ ] Provide primitives: `rotate`, `splay`, `join`, `split`, `expose`.
- [ ] Maintain per-node metadata: subtree size (and any lazy tags if used).
- [ ] Support component iteration (choose one option and complete it):
  - [ ] Option A: intrusive per-component vertex list with correct splice/split on link/cut.
  - [ ] Option B: component iterator over Euler tour with dedup/stamp, bounded to smaller side.
- [ ] `iter_component_vertices(u)` yields each vertex exactly once.
- [ ] Randomized forest tests validate `connected`, `size`, and component iteration against a baseline.

## 3) HDT level hierarchy + replacement logic
- [ ] Implement `HdtConnectivity` with `levels`, `edges`, `n`, and `L = floor(log2(max(1,n)))`.
- [ ] Each `LevelState` includes `ett` and `adj_non_tree`.
- [ ] `add_edge(u, v)`:
  - [ ] Link tree edges when disconnected at level 0 (per invariant).
  - [ ] Otherwise insert as non-tree at the correct level.
- [ ] `remove_edge(u, v)`:
  - [ ] Remove non-tree edges from adjacency + map.
  - [ ] For tree edges: cut from all relevant forests, run replacement search, and relink if found.
- [ ] `find_replacement(cut_level, u, v)`:
  - [ ] Scan only the smaller component.
  - [ ] Promote non-tree edges when they do not reconnect the cut.
  - [ ] Update adjacency sets and `EdgeRec.level` on promotion.
- [ ] Debug promotion count asserts `O(log n)` per edge (bounded by `L`).

## 4) Node growth + dynamic levels
- [ ] Node insertion extends ETT arrays and adjacency arrays across all levels.
- [ ] When `L` grows: append a new `LevelState` and populate tree edges per invariant.
- [ ] Existing non-tree edges remain at recorded levels (or are repositioned consistently).
- [ ] Invariants continue to hold after growth.

## 5) Python bindings
- [ ] PyO3 `DynamicGraph` class with Python node mapping (forward + reverse).
- [ ] Release GIL for heavy operations (`add_edge`, `remove_edge`, batch updates, large queries).
- [ ] Ergonomic API: `add_edge`, `remove_edge`, `has_edge`, `connected`, `nodes`, `edges`, `to_networkx`.
- [ ] `from_networkx` constructor or `update_from_edges` support.
- [ ] Behavior stable with mixed hashable node types (no ref leaks).

## 6) Correctness suite (Rust + Python)
- [ ] Rust ETT unit tests.
- [ ] HDT targeted tests for replacement on tree edge deletion.
- [ ] Property-based tests vs oracle after every operation.
- [ ] Python deterministic tests.
- [ ] Python random stress tests vs NetworkX.

## 7) Performance validation
- [ ] Rust Criterion benchmarks: `connected`, mixed workloads.
- [ ] Python microbench vs NetworkX.
- [ ] CI performance smoke checks (non-flaky guardrail).

## 8) Packaging, wheels, and CI
- [ ] `pyproject.toml` configured for maturin.
- [ ] CI matrix covers rustfmt/clippy, cargo test, and pytest.
- [ ] Wheels build on Linux/macOS/Windows + sdist.
- [ ] Release workflow uses Trusted Publishing.

## 9) Documentation
- [ ] README updated with correct complexity, graph model, examples, and determinism notes.
- [ ] Python docstrings for all exported API functions.
- [ ] README examples covered by tests.

## 10) Final “done” checks
- [ ] Invariants are written down and enforced in debug mode.
- [ ] ETT supports link/cut/connected/size + component enumeration correctly.
- [ ] Tree deletions trigger replacement search + promotions correctly.
- [ ] Random sequences match oracle after every step.
- [ ] Wheels build/install across OS + Python versions without a Rust toolchain.
- [ ] Benchmarks show expected qualitative behavior.
- [ ] Python API works with arbitrary hashable nodes + NetworkX snapshots.

## Required CI/test markers (do not remove)
These markers are validated in CI to ensure required jobs and test categories remain present.
- [ ] CI-MARKER: lint-rust
- [ ] CI-MARKER: test-rust
- [ ] CI-MARKER: test-python
- [ ] CI-MARKER: wheels-build
- [ ] CI-MARKER: wheels-smoke
- [ ] CI-MARKER: release
- [ ] TEST-MARKER: rust-ett-unit
- [ ] TEST-MARKER: rust-hdt-targeted
- [ ] TEST-MARKER: rust-property
- [ ] TEST-MARKER: python-deterministic
- [ ] TEST-MARKER: python-random
- [ ] TEST-MARKER: perf-smoke
