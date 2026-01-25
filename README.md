# hdtgraph-rs
Holm–de Lichtenberg–Thorup (HDT) deterministic fully dynamic connectivity, based on a hierarchy of spanning forests represented with Euler Tour Trees (ETTs). Rust-first core with a Python API that feels natural next to NetworkX, plus disciplined packaging and CI.

## Project identity

Repo name: `hdtgraph-rs`

PyPI name: `hdtgraph-rs`

The goal here is a Rust-first core with a Python API that feels natural next to NetworkX, plus disciplined packaging and CI.

## Scope and guarantees

What we ship in v1:

* Undirected, simple graph (no parallel edges) with dynamic edge insert/delete
* Queries:

  * `connected(u, v) -> bool`
  * `component(u) -> int` (stable only until next update)
  * `components() -> Iterator[set[node]]` (optional, may be linear-time)
* Updates:

  * `add_edge(u, v)`
  * `remove_edge(u, v)`
  * `add_node(u)` (optional; can be implicit on first use)
  * `remove_node(u)` (optional; can be implemented as removing all incident edges)

Complexity target (amortized):

* Update: `O(log^2 n)`
* Query: `O(log n)` (or `O(log n / log log n)` in the original analysis depending on representation details) ([ACM Digital Library][2])

Algorithm: Holm–de Lichtenberg–Thorup (HDT) deterministic fully dynamic connectivity, based on a hierarchy of spanning forests represented with Euler Tour Trees (ETTs). ([ACM Digital Library][2])

## High-level algorithm recap

HDT maintains:

* Levels `0..L`, where `L = floor(log2(n))`
* For each level `i`, a subgraph consisting of edges with level ≥ `i`
* A spanning forest `F_i` for the level-`i` subgraph
* Nesting invariant: `F_{i+1} ⊆ F_i` (higher levels are sparser)

Edges are classified:

* Tree edge: present in the forest at its level and therefore in all forests down to level 0
* Non-tree edge: stored in adjacency sets at its level, used as replacement candidates after deletions

Key amortization idea:

* When a tree edge is deleted, you try to find a replacement edge connecting the two new components.
* If you fail at a level, you “push” certain non-tree edges up a level (making future searches cheaper), and you only search the smaller side component (geometric charging). ([CMU School of Computer Science][3])

## Core data structures

### Node identity and Python compatibility

Python users will pass arbitrary hashable objects (ints, strings, tuples).

In Python:

* Maintain a bijection:

  * `py_node -> node_id: u32`
  * `node_id -> py_node` (for iteration and export)
* Use `hash(py_node)` only for dictionary keys; never as the node id.

In Rust:

* All algorithmic structures operate on compact `u32` node ids.

This keeps:

* Fast contiguous arrays in Rust
* “NetworkX-like” ergonomics in Python

### Edge identity

Use canonical `(min(u,v), max(u,v))` for undirected edges.

Rust representation:

```text
struct EdgeRec {
  u: u32,
  v: u32,
  level: u8,      // 0..L
  is_tree: bool,  // tree edge at its current level
}
```

A global `HashMap<(u32,u32), EdgeRec>` is the source of truth for edge existence and metadata.

### Level forests via Euler Tour Trees (ETT)

We need dynamic forest operations:

* `link(u, v)`  (add tree edge)
* `cut(u, v)`   (remove tree edge)
* `connected(u, v)` / `find_root(u)` / `component_size(u)`

Implement ETT using a deterministic self-adjusting BST (splay tree). This matches the “heavy pointer manipulation” motivation and avoids randomized balancing. ETT background appears in standard lecture notes and is used as the dynamic tree primitive. ([courses.csail.mit.edu][4])

ETT representation details:

* Convert each undirected tree edge `{u,v}` into two directed “arc” nodes `(u→v)` and `(v→u)` plus one vertex “occurrence” node per vertex.
* Maintain an Euler tour sequence for each tree as a splay tree over occurrences.
* Support `split` and `concat` on sequences.
* Each vertex maintains a handle to one occurrence node in its current tree (per level) for root finding.

Minimum API per level `i`:

```text
struct EttForest {
  // internal node pool for splay nodes
  fn connected(&self, u: u32, v: u32) -> bool
  fn root(&self, u: u32) -> u32
  fn size(&self, u: u32) -> usize
  fn link(&mut self, u: u32, v: u32)
  fn cut(&mut self, u: u32, v: u32)
  fn iter_vertices_in_component(&self, u: u32) -> VertexIter  // needed for “scan smaller side”
}
```

That last iterator is the practical pain point. Implement it by storing, in each splay node, aggregated info that lets you iterate the Euler tour and extract unique vertices. Two workable approaches:

1. Maintain a per-component “vertex list” side structure that is updated on `link/cut` using ETT split/concat boundaries.
2. Expose an iterator over the Euler tour sequence and deduplicate vertices with a visitation stamp array (fast in practice, still amortized acceptable if you scan only the smaller component).

For v1, do (2), and gate it behind “scan smaller side only”.

### Non-tree edge adjacency by level

For each level `i`, store adjacency of non-tree edges:

* `adj[i][u] = HashSet<v>` (neighbors via non-tree edges at level i)

Constraints:

* Must support delete by exact edge quickly
* Must support iterating all incident non-tree edges of a set of vertices efficiently

Implementation choice:

* Use `hashbrown::HashSet<u32>` per `(level, u)` in Rust.
* Store only the neighbor `v`; the canonical edge key lives in the global edge map.

Memory layout:

* `Vec<Vec<HashSet<u32>>>` would be too fragmented.
* Prefer:

  * `Vec<HashMap<u32, HashSet<u32>>>` per level (only allocate for vertices that have non-tree edges at that level), or
  * A single `HashMap<(u8,u32), HashSet<u32>>`

Pick:

* `levels: Vec<LevelState>`
* `struct LevelState { ett: EttForest, adj_non_tree: Vec<HashSet<u32>> }`

  * `adj_non_tree` is `Vec<HashSet<u32>>` indexed by `u32` node id (resize as nodes are added)
  * This is fast and simpler than sparse maps; resizing is linear in nodes but only on new node insertions.

## HDT operations (spec-level pseudocode)

Let `L = floor(log2(n_max_seen))`. When `n` grows, allow `L` to grow and append new levels (details below).

### `connected(u, v)`

Return `levels[0].ett.connected(u, v)`.

Reason: level 0 forest spans the whole graph connectivity by invariant.

### `add_edge(u, v)`

1. Canonicalize `(a,b) = (min(u,v), max(u,v))`
2. If edge exists, no-op or raise (choose no-op by default)
3. Start at level 0:

   * If `levels[0].ett.connected(u, v)` is false:

     * Insert edge as tree edge at level 0:

       * `edge.level = 0`, `edge.is_tree = true`
       * For all `i in 0..=L`: `levels[i].ett.link(u, v)` until the first level where u,v are already connected in that level’s forest (in the standard HDT nesting, a tree edge at level `ℓ` belongs to all forests `0..ℓ`; keep that invariant explicit and consistent in code)
     * Done
   * Else:

     * Insert edge as non-tree at level 0:

       * `edge.level = 0`, `edge.is_tree = false`
       * Add to `adj_non_tree[0][u].insert(v)` and `adj_non_tree[0][v].insert(u)`
     * Done

Implementation note: Keep one consistent convention:

* Convention A (common in descriptions): an edge at level `ℓ` belongs to subgraph `G_ℓ` and all lower subgraphs `G_0..G_ℓ`.
* Then forest `F_i` is spanning forest of `G_i`, so tree edges at level ≥ i appear in `F_i`.

Pick one, encode it as helpers:

* `edge_active_at_level(edge_level, i) -> bool`
* `edge_in_forest_at_level(tree_edge_level, i) -> bool`

### `remove_edge(u, v)`

1. Look up edge record; if missing, no-op or raise (choose no-op)
2. Let `ℓ = edge.level`
3. If non-tree:

   * Remove from `adj_non_tree[ℓ][u]` and `[v]`
   * Delete from global edge map
   * Done
4. If tree edge:

   * For each level `i in 0..=ℓ` where this tree edge is represented in the forest under your chosen convention:

     * `levels[i].ett.cut(u, v)`
   * Now the deletion split the component at level `ℓ` (and thus at level 0) into two parts. We need a replacement edge, trying from level `ℓ` downward (or upward depending on convention; keep it aligned with the paper’s replacement search logic). The standard approach: search for a replacement among non-tree edges at levels `0..ℓ`, promoting edges upward when scanned and found unusable. ([courses.csail.mit.edu][4])
   * Call `find_replacement(u, v, ℓ)`:

     * If found edge `(x, y)`:

       * Convert it to a tree edge at some level `k <= ℓ` per the algorithm rules
       * Remove it from non-tree adjacency at its current level
       * Link it into forests `0..=k` (per convention)
     * If not found:

       * Component stays split; connectivity reflects that

### `find_replacement(u, v, ℓ)` (core HDT logic)

Let `Cu` be the component containing `u` in `F_ℓ`, `Cv` containing `v`.

1. Identify the smaller component `S` and larger `B` using `levels[ℓ].ett.size(u)` and `.size(v)`
2. For `i` from `ℓ` down to `0`:

   * Iterate vertices `w` in `S` using `levels[i].ett.iter_vertices_in_component(w0)`
   * For each `w`, scan all incident non-tree neighbors `t` in `levels[i].adj_non_tree[w]`

     * If `levels[i].ett.connected(w, t)` is false, then `(w,t)` is a candidate replacement edge at level `i`

       * Return it immediately (deterministic “first found” is fine)
     * Else:

       * This edge is internal to the component at level `i`; per HDT amortization, raise its level (push it “up”):

         * Remove `(w,t)` from `adj_non_tree[i]`
         * Insert it into `adj_non_tree[i+1]` if `i+1 <= ℓ` else keep at top
         * Update `EdgeRec.level += 1`
   * After scanning all vertices in `S` at level `i` without finding a replacement, proceed to `i-1`
3. If no replacement found, return None

This is the part you must implement with care so that:

* Each non-tree edge gets promoted only `O(log n)` times total
* You only scan the smaller side at each deletion
  Those two facts are the amortization engine. ([CMU School of Computer Science][3])

Practical guardrails:

* While iterating `adj_non_tree[w]`, you mutate it (removals/promotions). Use a temporary Vec of neighbors to avoid iterator invalidation.
* When you find a replacement `(x,y)`, you must stop scanning and then:

  * Remove it from adjacency at its current level
  * Mark it tree
  * Link into forests according to the chosen convention
  * Keep its recorded level consistent with where you link it

## Growing `n` and number of levels

HDT assumes a fixed `n` for `L = O(log n)`. In a Python library, nodes can appear over time.

Policy:

* Support monotonic growth of node count; never shrink `L`.
* When `n` crosses a power of two and `floor(log2 n)` increases:

  * Append a new `LevelState` with an empty `EttForest` seeded from the previous top forest (or built by replaying existing top-level tree edges).
  * Easiest: rebuild only the new top-level forest by linking all current tree edges whose level is at least the new top level threshold (under your convention). This is a one-time cost per doubling.

Expose this as internal maintenance; do not surprise the user.

## Python API design (NetworkX-friendly)

### Primary class

`rustdynconn.DynamicGraph`

Methods:

* `add_node(node)`
* `add_edge(u, v)`
* `remove_edge(u, v)`
* `has_edge(u, v) -> bool`
* `connected(u, v) -> bool`
* `nodes() -> Iterator[node]`
* `edges() -> Iterator[tuple[node,node]]`
* `to_networkx() -> networkx.Graph`
* `update(edges_add=[...], edges_remove=[...])` (batch mode)

Properties:

* `n`, `m`
* `levels` (read-only; useful for debugging)

### NetworkX adapter

Provide a lightweight wrapper that mimics a subset of `networkx.Graph`:

`rustdynconn.nx.DynamicGraphAdapter`

* Supports `G.add_edge`, `G.remove_edge`, `G.has_edge`, `G.nodes`, `G.edges`
* Adds `G.connected(u, v)` which NetworkX does not have as a method
* Clear docstrings: this is not a full NetworkX drop-in

Rationale: users can keep their mental model and swap in your structure where dynamic connectivity matters.

## Rust crate layout

Repository structure:

```text
rustdynconn/
  Cargo.toml
  crates/
    rustdynconn-core/
      Cargo.toml
      src/
        lib.rs
        hdt/
          mod.rs
          dynamic_graph.rs
          levels.rs
          replacement.rs
        ett/
          mod.rs
          splay.rs
          ett_forest.rs
        util/
          mod.rs
          edge.rs
          smallvec.rs
    rustdynconn-py/
      Cargo.toml
      pyproject.toml
      src/
        lib.rs          # PyO3 bindings
        mapping.rs      # Py object <-> node_id
        api.rs          # DynamicGraph class
  python/
    rustdynconn/
      __init__.py
      nx.py            # adapter
      _typing.py
  tests/
    python/
      test_correctness.py
      test_nx_compat.py
      test_stress_random.py
      test_performance_smoke.py
  benches/
    rust/
      hdt_bench.rs     # Criterion benches
    python/
      bench_connected.py
  .github/
    workflows/
      ci.yml
      wheels.yml
      release.yml
  README.md
  LICENSE
```

Key Rust dependencies:

* `hashbrown` for sets/maps
* `pyo3` and `maturin` for Python extension
* `criterion` for Rust benchmarks
* `proptest` for property-based tests
* `rayon` behind a feature flag for parallel batch queries/updates

Cargo features:

* `default = ["pyo3/extension-module"]` in the Python crate
* `parallel = ["rayon"]` in core crate
* `debug_invariants` to enable expensive checks in tests

## Parallelization and SIMD opportunities

### Parallelism that is safe and worthwhile

1. Batch connectivity queries

   * `connected_many(pairs: &[(u,v)]) -> Vec<bool>`
   * These are read-only; run with rayon over pairs.
   * ETT `connected` must be thread-safe for reads:

     * Either guard the whole graph with an `RwLock` and take a read lock for the duration, or
     * Keep the Rust core single-threaded and parallelize at Python level by releasing the GIL and using internal read locks

2. Batch updates (optional, advanced)

   * HDT updates mutate shared state across levels and adjacency.
   * True parallel edge updates are hard without heavy locking and tend to lose the point.
   * Provide a batch API that is still sequential internally but reduces Python overhead:

     * Accept arrays of edges to add/remove in one call
     * Release the GIL for the whole batch

3. Replacement search inner loops (limited)

   * Naively parallelizing “scan smaller component” is tempting, but you promote edges during scanning, which is mutating.
   * A workable pattern:

     * Phase 1 (read-only): collect candidate incident edges from vertices in `S` into a Vec
     * Phase 2 (sequential): walk that Vec, test connectivity, apply promotions, stop when replacement found
   * This can reduce iterator overhead, not asymptotic time. Keep it behind a feature flag and benchmark.

### SIMD

Most work is pointer chasing in splay trees and hashing neighbors. SIMD has limited upside.

Where SIMD can help a bit:

* If you represent adjacency temporarily as sorted `Vec<u32>` during batch operations, you can speed up intersection-style scans with `std::simd` or by delegating to `packed_simd`-style patterns.
* This is not core to HDT; treat as a later optimization.

## Correctness instrumentation

Implement internal invariants with `debug_invariants`:

* For each level `i`:

  * `F_{i+1} ⊆ F_i` (nesting)
  * Forest is acyclic (ETT structure consistency)
* For each edge:

  * If `is_tree`, it must be represented in forests according to convention
  * If non-tree at level `ℓ`, it must exist in adjacency sets exactly at `ℓ`
* After every update in debug mode:

  * Compare `connected(u, v)` from HDT vs a slow baseline for a small sampled set of pairs

Expose a developer method in Python:

* `G._check_invariants()` raising `AssertionError` with details

## Test suite specification

### Rust unit tests

1. ETT micro-tests

   * Build small trees, test `link/cut/connected/size`
   * Random sequences of `link/cut` that always maintain a forest, compare to a simple parent-pointer baseline

2. HDT deterministic scenarios

   * Known graphs:

     * Chain, star, balanced tree plus extra chords
   * Delete tree edges and verify replacement selection maintains connectivity if a non-tree replacement exists

3. Property-based tests (`proptest`)

   * Generate a sequence of operations on up to N=200 nodes:

     * `add_edge`, `remove_edge`, `connected_query`
   * Maintain a baseline adjacency set, answer queries with BFS/Union-Find rebuild on demand
   * Assert query equality every time

### Python tests (pytest)

1. API behavior

   * Node mapping preserves identity
   * `remove_edge` idempotency policy
   * `has_edge`, `edges()`, `nodes()`

2. NetworkX comparison

   * For each random operation sequence:

     * Apply to `rustdynconn.DynamicGraph`
     * Apply to `networkx.Graph`
     * Validate `connected(u,v)` equals `nx.has_path(G, u, v)` for sampled pairs

NetworkX is static in the sense that it does not maintain connectivity incrementally, but it is a reliable oracle for correctness on each step. ([courses.csail.mit.edu][4])

3. Stress tests (marked slow)

   * N=10_000 nodes
   * Perform 100_000 mixed updates and queries
   * Assert no panics, track basic timings

4. Performance smoke tests

   * Use `pytest-benchmark`:

     * Compare `connected` throughput against:

       * NetworkX `has_path` on the same graph snapshot
     * Keep thresholds loose to avoid flaky CI, but record benchmark output as an artifact

### Benchmarks (not gating CI)

Rust `criterion` benches:

* `connected` on various densities
* Update-heavy workloads:

  * mostly deletions of tree edges with available replacements
  * adversarial-ish patterns (still realistic)

Python benches:

* Microbench `connected` for repeated queries
* Mixed workload benchmark

## Packaging and build system

Use `maturin` + PyO3 for the Python extension.

`pyproject.toml` in `crates/rustdynconn-py/`:

* Build backend: `maturin`
* Provide wheels with `abi3` if possible:

  * If you avoid CPython-version-specific APIs in PyO3, you can target `abi3-py39` and ship one wheel per platform for 3.9+
  * This cuts build matrix size

If you need full CPython-per-version wheels (fine for v1), build for 3.9–3.13.

## GitHub Actions specification

### Workflow 1: CI (PR + push)

`.github/workflows/ci.yml`

Triggers:

* `pull_request`
* `push` to `main`

Jobs:

1. `lint-rust`

   * `rustfmt` check
   * `clippy` with `-D warnings`

2. `test-rust`

   * `cargo test -p rustdynconn-core`
   * Enable `debug_invariants`

3. `test-python`

   * Strategy matrix:

     * OS: ubuntu-latest, macos-latest, windows-latest
     * Python: 3.9, 3.10, 3.11, 3.12, 3.13
   * Steps:

     * checkout
     * setup-python
     * install maturin
     * `maturin develop -m crates/rustdynconn-py/pyproject.toml`
     * `pip install -r tests/python/requirements.txt` (pytest, networkx, pytest-benchmark)
     * `pytest -q tests/python -m "not slow"`

Artifacts:

* Upload pytest-benchmark JSON results per OS/python

### Workflow 2: Wheels (release build, but still test)

`.github/workflows/wheels.yml`

Triggers:

* `workflow_dispatch`
* `push` tags `v*`

Jobs:

1. `build-wheels`

   * Use `PyO3/maturin-action@v1`
   * Matrix:

     * `manylinux` x86_64 (+ aarch64 if you want)
     * `macos` universal2
     * `windows` x86_64
   * Args:

     * `--release`
     * `--out dist`
     * `--sdist`
   * Upload artifacts `dist/*`

2. `smoke-test-wheels`

   * Download wheel artifacts
   * Create venv
   * `pip install rustdynconn --no-index --find-links dist`
   * Run a tiny import + small correctness test

### Workflow 3: Release (publish to PyPI)

`.github/workflows/release.yml`

Triggers:

* `push` tags `v*`

Permissions:

* `id-token: write` (trusted publishing)
* `contents: read`

Steps:

* Reuse wheels from `wheels.yml` via `workflow_call`, or rebuild
* `maturin upload --non-interactive --skip-existing dist/*`

Security:

* Use PyPI Trusted Publishing, no API token in secrets.

## Documentation and examples

Ship examples that prove the point:

1. “Dynamic vs static”

   * Build graph, remove edge, compare:

     * `rustdynconn.connected(a,b)` is fast
     * NetworkX recomputes via `has_path`

2. Simulation-style workload

   * Random add/remove edges with periodic connectivity queries
   * Track throughput

3. NetworkX interop

   * Convert from NetworkX edge list into `DynamicGraph`
   * Convert back to NetworkX snapshot

4. Debugging hooks

   * Show `levels`, `edge_level_histogram`, invariant checks

## Implementation milestones

1. Rust ETT (splay) with `link/cut/connected/size`
2. Level-0 HDT only (acts like “dynamic forest + extra edges”)
3. Full multi-level HDT with promotions and smaller-side scan
4. Python bindings + node mapping
5. Correctness: proptest + pytest vs NetworkX oracle
6. Wheels + CI + release pipeline
7. Benchmarks and profiling pass

## References used for the spec

* Original JACM HDT paper (Poly-logarithmic deterministic fully-dynamic algorithms…) ([ACM Digital Library][2])
* MIT 6.851 lecture notes on dynamic connectivity and Euler Tour Trees ([courses.csail.mit.edu][4])
* Notes emphasizing the “raise levels to pay for replacement search” amortization intuition ([CMU School of Computer Science][3])
* Existing Python HDT implementation on PyPI (`hdtgraph`) for competitive context ([PyPI][1])


[1]: https://pypi.org/project/hdtgraph/?utm_source=chatgpt.com "hdtgraph"
[2]: https://dl.acm.org/doi/pdf/10.1145/502090.502095?utm_source=chatgpt.com "Poly-logarithmic deterministic fully-dynamic algorithms for ..."
[3]: https://www.cs.cmu.edu/afs/cs.cmu.edu/academic/class/15850-f20/www/notes/lec3.pdf?utm_source=chatgpt.com "3 Dynamic Algorithms for Graph Connectivity"
[4]: https://courses.csail.mit.edu/6.851/spring12/scribe/L20.pdf?utm_source=chatgpt.com "1 Overview 2 Dynamic Connectivity"

