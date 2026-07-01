# rsomics-link-prediction

Score node pairs by neighborhood overlap — a value-exact Rust port of the local
link-prediction predictors in `networkx.algorithms.link_prediction`. One crate,
one operation ("score node pairs by neighborhood overlap"); the five methods
differ only in the per-pair score formula, selected with `--method`.

## Install

```
cargo install rsomics-link-prediction
```

## Usage

Reads an undirected edge list on stdin (`u v` per line, string labels; `#`
comments and blank lines ignored; parallel edges deduped; self-loops dropped —
matching `nx.Graph`).

```
# score all non-adjacent pairs (nx ebunch=None), default method = jaccard
rsomics-link-prediction < graph.edges

# a specific method over a chosen set of pairs
rsomics-link-prediction --method adamic-adar --pairs query.txt < graph.edges

# machine-readable
rsomics-link-prediction --method resource-allocation --json < graph.edges

# CCPA with a custom alpha
rsomics-link-prediction --method common-neighbor-centrality --alpha 0.8 --pairs query.txt < graph.edges
```

`--pairs FILE` scores exactly the `u v` pairs in the file (any pair whose two
nodes exist in the graph — nx does not require them to be non-adjacent). Without
`--pairs`, every unordered non-adjacent pair is scored once (`u < v` by label),
which is O(n²).

### Methods (`--method`)

| flag | formula |
|---|---|
| `jaccard` (default) | \|N(u)∩N(v)\| / \|N(u)∪N(v)\| (0 if the union is empty) |
| `adamic-adar` | Σ_{w∈N(u)∩N(v)} 1/ln(deg w) |
| `resource-allocation` | Σ_{w∈N(u)∩N(v)} 1/deg(w) |
| `preferential-attachment` | deg(u)·deg(v) (integer) |
| `common-neighbor-centrality` (CCPA) | α·\|N(u)∩N(v)\| + (1−α)·N / d(u,v) |

Common neighbors are `(N(u)∩N(v)) \ {u,v}`, exactly as `nx.common_neighbors`.
The Jaccard union is `|N(u)∪N(v)|` over the raw neighbor sets, which — matching
nx — includes u and v when the pair is mutually adjacent.

For `common-neighbor-centrality`, `N` is the node count and `d(u,v)` is the
shortest-path hop distance (BFS). `--alpha` (default `0.8`) mixes the
common-neighbor and centrality terms; `--alpha 1` collapses the score to the
common-neighbor count (no distance term). A pair in different components takes
`d = ∞`, so — as in networkx — its centrality term vanishes.

Output: `u v score` per line, or a `--json` array of `{"u","v","score"}`.

## Performance

String labels are interned to `0..n` integer ids once; adjacency is stored as
sorted `Vec<usize>` plus a `HashSet<usize>` for O(1) membership, with degrees
precomputed. The per-pair inner loop never touches a `HashMap<String, _>` —
common neighbors are found by walking the smaller sorted adjacency list against
the other endpoint's membership set. This is the structural win over networkx's
per-pair Python loop.

## Origin

This crate is an independent Rust reimplementation of the local
link-prediction predictors in
[NetworkX](https://networkx.org) `networkx.algorithms.link_prediction`
(version 3.6.1, BSD-3-Clause). NetworkX is permissively licensed, so its source
was read and used as the behavioral reference for exact semantics (neighbor-set
exclusion of `{u,v}`, the Jaccard union convention, natural-log base for
Adamic-Adar, empty-union → 0, and CCPA's `alpha == 1` short-circuit and
infinite-distance handling for disconnected pairs).

Methods trace to:

- D. Liben-Nowell, J. Kleinberg. *The Link Prediction Problem for Social
  Networks* (2004) — Jaccard coefficient, Adamic-Adar index, preferential
  attachment framing for link prediction.
- L. A. Adamic, E. Adar. *Friends and neighbors on the Web*. Social Networks
  25(3), 211–230 (2003) — the 1/ln(deg) weighting.
- T. Zhou, L. Lü, Y.-C. Zhang. *Predicting missing links via local
  information*. Eur. Phys. J. B 71, 623–630 (2009).
  https://arxiv.org/abs/0901.0553 — resource allocation index.
- I. Ahmad, M. U. Akhtar, S. Noor, et al. *Missing Link Prediction using
  Common Neighbor and Centrality based Parameterized Algorithm (CCPA)*.
  Sci. Rep. 10, 364 (2020). https://doi.org/10.1038/s41598-019-57304-y —
  common-neighbor-centrality.

Value-exactness is verified in `tests/compat.rs` against golden scores captured
from NetworkX 3.6.1 (floats compared within 1 ULP; preferential-attachment as
exact integers). No NetworkX/Python is invoked at test time.

License: MIT OR Apache-2.0.
Upstream credit: NetworkX (https://networkx.org), BSD-3-Clause.
