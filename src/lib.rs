//! Neighborhood-overlap link-prediction scores, value-exact against networkx 3.6.1.
//!
//! Each score is a per-pair function of the two nodes' neighbor sets. Labels are
//! interned to `0..n` integer ids once; adjacency is stored as sorted `Vec<usize>`
//! plus a `HashSet<usize>` for O(1) membership, so the per-pair inner loop never
//! touches a `HashMap<String, _>` — the win over networkx's per-pair Python loop.

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Jaccard,
    AdamicAdar,
    ResourceAllocation,
    PreferentialAttachment,
    CommonNeighborCentrality,
}

impl Method {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "jaccard" => Ok(Method::Jaccard),
            "adamic-adar" => Ok(Method::AdamicAdar),
            "resource-allocation" => Ok(Method::ResourceAllocation),
            "preferential-attachment" => Ok(Method::PreferentialAttachment),
            "common-neighbor-centrality" => Ok(Method::CommonNeighborCentrality),
            other => Err(format!(
                "unknown method '{other}' (expected jaccard, adamic-adar, \
                 resource-allocation, preferential-attachment, or \
                 common-neighbor-centrality)"
            )),
        }
    }
}

/// One scored pair, carrying the original string labels for output.
#[derive(Debug, Clone, Serialize)]
pub struct Prediction {
    pub u: String,
    pub v: String,
    pub score: f64,
}

/// Interned undirected graph matching `nx.Graph` semantics: parallel edges
/// deduped, self-loops dropped, node ids assigned in first-seen order.
pub struct Graph {
    labels: Vec<String>,
    ids: HashMap<String, usize>,
    adj_sorted: Vec<Vec<usize>>,
    adj_set: Vec<HashSet<usize>>,
    deg: Vec<usize>,
}

impl Graph {
    pub fn from_edge_list(input: &str) -> Self {
        let mut labels: Vec<String> = Vec::new();
        let mut ids: HashMap<String, usize> = HashMap::new();
        let mut adj_set: Vec<HashSet<usize>> = Vec::new();

        let intern = |label: &str,
                      labels: &mut Vec<String>,
                      ids: &mut HashMap<String, usize>,
                      adj_set: &mut Vec<HashSet<usize>>|
         -> usize {
            if let Some(&id) = ids.get(label) {
                id
            } else {
                let id = labels.len();
                labels.push(label.to_string());
                ids.insert(label.to_string(), id);
                adj_set.push(HashSet::new());
                id
            }
        };

        for line in input.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut it = line.split_whitespace();
            let (Some(a), Some(b)) = (it.next(), it.next()) else {
                continue;
            };
            let ua = intern(a, &mut labels, &mut ids, &mut adj_set);
            let ub = intern(b, &mut labels, &mut ids, &mut adj_set);
            if ua == ub {
                continue; // self-loop dropped, matching nx.Graph
            }
            adj_set[ua].insert(ub);
            adj_set[ub].insert(ua);
        }

        let n = labels.len();
        let mut adj_sorted = Vec::with_capacity(n);
        let mut deg = Vec::with_capacity(n);
        for s in &adj_set {
            let mut v: Vec<usize> = s.iter().copied().collect();
            v.sort_unstable();
            deg.push(v.len());
            adj_sorted.push(v);
        }

        Graph {
            labels,
            ids,
            adj_sorted,
            adj_set,
            deg,
        }
    }

    pub fn n_nodes(&self) -> usize {
        self.labels.len()
    }

    pub fn id_of(&self, label: &str) -> Option<usize> {
        self.ids.get(label).copied()
    }

    pub fn label(&self, id: usize) -> &str {
        &self.labels[id]
    }

    fn adjacent(&self, u: usize, v: usize) -> bool {
        self.adj_set[u].contains(&v)
    }

    /// Common neighbors in ascending interned-id order.
    ///
    /// Mirrors `nx.common_neighbors`: intersection of the two neighbor sets with
    /// `{u, v}` removed. Walks the smaller sorted adjacency list, probing the
    /// other's membership set. Ascending id order fixes the float summation order.
    fn common_neighbors(&self, u: usize, v: usize) -> impl Iterator<Item = usize> + '_ {
        let (small, big) = if self.adj_sorted[u].len() <= self.adj_sorted[v].len() {
            (u, v)
        } else {
            (v, u)
        };
        let other = &self.adj_set[big];
        self.adj_sorted[small]
            .iter()
            .copied()
            .filter(move |&w| w != u && w != v && other.contains(&w))
    }

    /// Hop distance `u→v` (edges on a shortest path), matching
    /// `nx.shortest_path_length`. `None` when `v` is unreachable from `u`.
    fn bfs_dist(&self, u: usize, v: usize) -> Option<usize> {
        if u == v {
            return Some(0);
        }
        let mut seen = vec![false; self.adj_sorted.len()];
        let mut q: VecDeque<(usize, usize)> = VecDeque::new();
        seen[u] = true;
        q.push_back((u, 0));
        while let Some((node, d)) = q.pop_front() {
            for &w in &self.adj_sorted[node] {
                if w == v {
                    return Some(d + 1);
                }
                if !seen[w] {
                    seen[w] = true;
                    q.push_back((w, d + 1));
                }
            }
        }
        None
    }

    fn score_pair(&self, method: Method, u: usize, v: usize) -> f64 {
        match method {
            Method::Jaccard => {
                let union = self.union_size(u, v);
                if union == 0 {
                    0.0
                } else {
                    self.common_neighbors(u, v).count() as f64 / union as f64
                }
            }
            // `+ 0.0` canonicalises the empty-sum identity `-0.0` to `0.0` so the
            // text output reads `0` like nx; it leaves every nonzero sum bit-exact.
            Method::AdamicAdar => {
                let s: f64 = self
                    .common_neighbors(u, v)
                    .map(|w| 1.0 / (self.deg[w] as f64).ln())
                    .sum();
                s + 0.0
            }
            Method::ResourceAllocation => {
                let s: f64 = self
                    .common_neighbors(u, v)
                    .map(|w| 1.0 / self.deg[w] as f64)
                    .sum();
                s + 0.0
            }
            Method::PreferentialAttachment => (self.deg[u] * self.deg[v]) as f64,
            // CCPA needs alpha + BFS distance; it is scored by the closure in
            // `link_prediction_from_edge_list`, never here.
            Method::CommonNeighborCentrality => {
                unreachable!("common-neighbor-centrality is scored with alpha, not score_pair")
            }
        }
    }

    /// `|N(u) ∪ N(v)|` over the raw neighbor sets — matches nx jaccard's
    /// `len(set(G[u]) | set(G[v]))`, which does NOT exclude u or v.
    fn union_size(&self, u: usize, v: usize) -> usize {
        let (small, big) = if self.adj_sorted[u].len() <= self.adj_sorted[v].len() {
            (u, v)
        } else {
            (v, u)
        };
        let extra = self.adj_sorted[small]
            .iter()
            .filter(|w| !self.adj_set[big].contains(w))
            .count();
        self.deg[big] + extra
    }
}

fn score_to_json(method: Method, s: f64) -> serde_json::Value {
    if matches!(method, Method::PreferentialAttachment) {
        serde_json::json!(s as u64)
    } else {
        serde_json::json!(s)
    }
}

/// Score the requested pairs (or all non-adjacent pairs when `pairs` is `None`).
///
/// `pairs = None` replicates `nx` `ebunch=None`: every unordered non-adjacent
/// pair once, `u < v` in interned id order (O(n²)). With `pairs`, each given
/// pair is scored verbatim — nx does not require them to be non-adjacent.
pub fn link_prediction_from_edge_list(
    input: &str,
    method: Method,
    pairs: Option<&[(String, String)]>,
    alpha: f64,
) -> anyhow::Result<Vec<Prediction>> {
    let g = Graph::from_edge_list(input);

    // CCPA (`nx.common_neighbor_centrality`) is the only alpha/distance-bearing
    // score: alpha·|CN| + (1-alpha)·N/d_uv, where d_uv is the BFS hop distance.
    // Matching nx, `alpha == 1` short-circuits to |CN| (no distance needed), and
    // an unreachable pair takes d_uv = ∞ so the second term vanishes to 0.
    #[allow(clippy::float_cmp)]
    let score = |u: usize, v: usize| -> f64 {
        match method {
            Method::CommonNeighborCentrality => {
                let cn = g.common_neighbors(u, v).count() as f64;
                if alpha == 1.0 {
                    cn
                } else {
                    match g.bfs_dist(u, v) {
                        Some(d) => alpha * cn + (1.0 - alpha) * g.n_nodes() as f64 / d as f64,
                        None => alpha * cn,
                    }
                }
            }
            _ => g.score_pair(method, u, v),
        }
    };

    let out = match pairs {
        None => {
            let n = g.n_nodes();
            let mut out = Vec::new();
            for u in 0..n {
                for v in (u + 1)..n {
                    if g.adjacent(u, v) {
                        continue;
                    }
                    let score = score(u, v);
                    // Emit each unordered pair with the endpoint whose label sorts
                    // first as `u`, so the default output is deterministic by label
                    // (nx `non_edges` iteration order is set-pop, nondeterministic).
                    let (lu, lv) = {
                        let (a, b) = (g.label(u), g.label(v));
                        if a <= b {
                            (a.to_string(), b.to_string())
                        } else {
                            (b.to_string(), a.to_string())
                        }
                    };
                    out.push(Prediction {
                        u: lu,
                        v: lv,
                        score,
                    });
                }
            }
            out.sort_by(|a, b| (&a.u, &a.v).cmp(&(&b.u, &b.v)));
            out
        }
        Some(pairs) => {
            let mut out = Vec::with_capacity(pairs.len());
            for (a, b) in pairs {
                let ua = g
                    .id_of(a)
                    .ok_or_else(|| anyhow::anyhow!("node {a} not in graph (present in --pairs)"))?;
                let ub = g
                    .id_of(b)
                    .ok_or_else(|| anyhow::anyhow!("node {b} not in graph (present in --pairs)"))?;
                let score = score(ua, ub);
                out.push(Prediction {
                    u: a.clone(),
                    v: b.clone(),
                    score,
                });
            }
            out
        }
    };

    Ok(out)
}

/// Text: `u v score` per line. Integer for preferential-attachment, else the
/// shortest round-tripping float ({} on f64 in Rust matches Python's repr for
/// these values within the 1-ULP compat gate).
pub fn format_text(method: Method, preds: &[Prediction]) -> String {
    let mut s = String::new();
    for p in preds {
        if matches!(method, Method::PreferentialAttachment) {
            s.push_str(&format!("{} {} {}\n", p.u, p.v, p.score as u64));
        } else {
            s.push_str(&format!("{} {} {}\n", p.u, p.v, p.score));
        }
    }
    s
}

pub fn to_json(method: Method, preds: &[Prediction]) -> serde_json::Value {
    let arr: Vec<serde_json::Value> = preds
        .iter()
        .map(|p| {
            serde_json::json!({
                "u": p.u,
                "v": p.v,
                "score": score_to_json(method, p.score),
            })
        })
        .collect();
    serde_json::Value::Array(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selfloop_and_parallel_deduped() {
        let g = Graph::from_edge_list("a b\na b\nb a\na a\n");
        assert_eq!(g.n_nodes(), 2);
        assert_eq!(g.deg[0], 1);
        assert_eq!(g.deg[1], 1);
    }

    #[test]
    fn common_neighbors_excludes_uv() {
        // triangle a-b-c plus a-b edge; for pair (a,b) common should be {c}
        let g = Graph::from_edge_list("a b\nb c\na c\n");
        let a = g.id_of("a").unwrap();
        let b = g.id_of("b").unwrap();
        let cn: Vec<usize> = g.common_neighbors(a, b).collect();
        assert_eq!(cn.len(), 1);
    }

    #[test]
    fn jaccard_adjacent_pair_union_includes_endpoints() {
        // reproduces the nx edge case: 0-1,0-2,1-2,1-3; jaccard(0,1)=1/4
        let g = Graph::from_edge_list("0 1\n0 2\n1 2\n1 3\n");
        let u = g.id_of("0").unwrap();
        let v = g.id_of("1").unwrap();
        let s = g.score_pair(Method::Jaccard, u, v);
        assert!((s - 0.25).abs() < 1e-12);
    }

    #[test]
    fn method_parse_roundtrip() {
        assert_eq!(Method::parse("jaccard").unwrap(), Method::Jaccard);
        assert!(Method::parse("nope").is_err());
    }
}
