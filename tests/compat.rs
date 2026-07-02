//! Value-exact compat against networkx 3.6.1. Goldens are hardcoded constants
//! generated once from nx at build time; graphs + query pairs are committed
//! `.txt` files under tests/golden loaded via `include_str!`. No subprocess,
//! no Python at test time.

use rsomics_link_prediction::{link_prediction_from_edge_list, Method, Prediction};

const EPS: f64 = 1e-12;

// -- committed graphs + query-pair lists --
const HAND: &str = include_str!("golden/hand.txt");
const HAND_PAIRS: &str = include_str!("golden/hand_pairs.txt");
const KARATE: &str = include_str!("golden/karate.txt");
const KARATE_PAIRS: &str = include_str!("golden/karate_pairs.txt");
const GNM1: &str = include_str!("golden/gnm1.txt");
const GNM1_PAIRS: &str = include_str!("golden/gnm1_pairs.txt");
const GNM2: &str = include_str!("golden/gnm2.txt");
const GNM2_PAIRS: &str = include_str!("golden/gnm2_pairs.txt");
const SMALL: &str = include_str!("golden/small.txt");
const DISC: &str = include_str!("golden/disc.txt");
const DISC_PAIRS: &str = include_str!("golden/disc_pairs.txt");
const SELFLOOP: &str = include_str!("golden/selfloop.txt");
const SELFLOOP_PAIRS: &str = include_str!("golden/selfloop_pairs.txt");

// -- golden scores from networkx 3.6.1 (do not edit) --
const HAND_JACCARD: &[f64] = &[0.5, 0.0, 0.5, 1.0, 0.0, 0.5, 0.3333333333333333];
const HAND_ADAMIC_ADAR: &[f64] = &[
    1.8204784532536746,
    0.0,
    1.8204784532536746,
    2.730717679880512,
    0.0,
    1.8204784532536746,
    0.9102392266268373,
];
const HAND_RESOURCE_ALLOCATION: &[f64] = &[
    0.6666666666666666,
    0.0,
    0.6666666666666666,
    1.0,
    0.0,
    0.6666666666666666,
    0.3333333333333333,
];
const HAND_PREFERENTIAL_ATTACHMENT: &[u64] = &[9, 3, 9, 9, 3, 9, 3];

const KARATE_JACCARD: &[f64] = &[
    0.13793103448275862,
    0.058823529411764705,
    0.0,
    0.047619047619047616,
    1.0,
    0.0,
    0.3333333333333333,
    0.3888888888888889,
    0.058823529411764705,
    0.0,
];
const KARATE_ADAMIC_ADAR: &[f64] = &[
    2.7110197222973085,
    0.43429448190325176,
    0.0,
    0.6213349345596119,
    0.7553857282466059,
    0.0,
    1.8033688011112043,
    6.130716871863356,
    0.35295612386476116,
    0.0,
];
const KARATE_RESOURCE_ALLOCATION: &[f64] = &[
    0.9,
    0.1,
    0.0,
    0.2,
    0.14215686274509803,
    0.0,
    0.5625,
    2.05,
    0.058823529411764705,
    0.0,
];
const KARATE_PREFERENTIAL_ATTACHMENT: &[u64] = &[272, 32, 36, 120, 4, 15, 16, 144, 72, 4];

const GNM1_JACCARD: &[f64] = &[
    0.13333333333333333,
    0.2857142857142857,
    0.0,
    0.2857142857142857,
    0.0,
    0.09090909090909091,
    0.08333333333333333,
    0.0,
    0.25,
    0.2,
];
const GNM1_ADAMIC_ADAR: &[f64] = &[
    1.1162212531024944,
    1.3426824550040934,
    0.0,
    0.9924051084544989,
    0.0,
    0.5581106265512472,
    0.5581106265512472,
    0.0,
    1.3479631252182434,
    1.9007930815553409,
];
const GNM1_RESOURCE_ALLOCATION: &[f64] = &[
    0.3333333333333333,
    0.45,
    0.0,
    0.26666666666666666,
    0.0,
    0.16666666666666666,
    0.16666666666666666,
    0.0,
    0.3246753246753247,
    0.6166666666666667,
];
const GNM1_PREFERENTIAL_ATTACHMENT: &[u64] = &[66, 14, 18, 20, 15, 36, 42, 8, 56, 77];

const GNM2_JACCARD: &[f64] = &[
    0.14285714285714285,
    0.0,
    0.13333333333333333,
    0.08695652173913043,
    0.0625,
    0.14285714285714285,
    0.13333333333333333,
    0.0,
    0.10526315789473684,
    0.1111111111111111,
];
const GNM2_ADAMIC_ADAR: &[f64] = &[
    1.2127776525177003,
    0.0,
    0.7802707381644454,
    0.8048592087636893,
    0.3789231816899512,
    0.8894140952166705,
    0.7959555731141975,
    0.0,
    0.7059122477295223,
    0.7152456293300486,
];
const GNM2_RESOURCE_ALLOCATION: &[f64] = &[
    0.2552521008403361,
    0.0,
    0.15555555555555556,
    0.16666666666666666,
    0.07142857142857142,
    0.2111111111111111,
    0.16233766233766234,
    0.0,
    0.11764705882352941,
    0.12222222222222222,
];
const GNM2_PREFERENTIAL_ATTACHMENT: &[u64] = &[119, 64, 66, 144, 70, 55, 70, 90, 98, 96];

// -- common-neighbor-centrality (CCPA), nx.common_neighbor_centrality --
// _08 = default alpha 0.8; _10 = alpha 1.0 (score collapses to |common neighbors|).
const HAND_CCPA_08: &[f64] = &[
    2.2,
    0.3999999999999999,
    2.2,
    3.0,
    0.3999999999999999,
    2.8,
    1.4,
];
const HAND_CCPA_10: &[f64] = &[2.0, 0.0, 2.0, 3.0, 0.0, 2.0, 1.0];

const KARATE_CCPA_08: &[f64] = &[
    6.6,
    4.199999999999999,
    6.799999999999999,
    7.599999999999999,
    5.0,
    6.799999999999999,
    8.399999999999999,
    12.399999999999999,
    7.599999999999999,
    2.266666666666666,
];
const KARATE_CCPA_10: &[f64] = &[4.0, 1.0, 0.0, 1.0, 2.0, 0.0, 2.0, 7.0, 1.0, 0.0];

const GNM1_CCPA_08: &[f64] = &[
    5.6,
    5.6,
    2.666666666666666,
    5.6,
    1.9999999999999996,
    4.799999999999999,
    8.799999999999999,
    1.9999999999999996,
    6.3999999999999995,
    6.3999999999999995,
];
const GNM1_CCPA_10: &[f64] = &[2.0, 2.0, 0.0, 2.0, 0.0, 1.0, 1.0, 0.0, 3.0, 3.0];

const GNM2_CCPA_08: &[f64] = &[
    8.399999999999999,
    3.9999999999999987,
    7.599999999999998,
    7.599999999999998,
    6.799999999999998,
    7.599999999999998,
    7.599999999999998,
    3.9999999999999987,
    13.599999999999996,
    7.599999999999998,
];
const GNM2_CCPA_10: &[f64] = &[3.0, 0.0, 2.0, 2.0, 1.0, 2.0, 2.0, 0.0, 2.0, 2.0];

// Disconnected pairs: nx takes d_uv = ∞ (no NetworkXNoPath from the ebunch
// path), so the distance term is 0 and the score is alpha·|CN| = 0 (a pair in
// different components shares no common neighbor).
const DISC_CCPA_08: &[f64] = &[0.0, 0.0];
const DISC_CCPA_10: &[f64] = &[0.0, 0.0];

// Self-loops on a common neighbor (c) and on an endpoint (a): nx.Graph keeps
// them and counts a self-loop twice toward degree, so every degree-based score
// diverges from a self-loop-dropping port. Goldens from networkx 3.6.1.
const SELFLOOP_JACCARD: &[f64] = &[
    0.5,
    0.5,
    0.6666666666666666,
    0.8,
    0.16666666666666666,
    1.0,
    0.5,
];
const SELFLOOP_ADAMIC_ADAR: &[f64] = &[
    1.279458146995729,
    1.279458146995729,
    1.279458146995729,
    4.416964242964376,
    0.6213349345596119,
    1.279458146995729,
    1.279458146995729,
];
const SELFLOOP_RESOURCE_ALLOCATION: &[f64] = &[
    0.41666666666666663,
    0.41666666666666663,
    0.41666666666666663,
    1.5333333333333332,
    0.2,
    0.41666666666666663,
    0.41666666666666663,
];
const SELFLOOP_PREFERENTIAL_ATTACHMENT: &[u64] = &[15, 10, 6, 24, 12, 4, 10];
const SELFLOOP_CCPA_08: &[f64] = &[2.8, 2.2, 2.2, 3.8, 1.9999999999999998, 2.2, 2.2];
const SELFLOOP_CCPA_10: &[f64] = &[2.0, 2.0, 2.0, 4.0, 1.0, 2.0, 2.0];

const SMALL_DEFAULT_JAC_PAIRS: &[(&str, &str)] =
    &[("q", "w"), ("q", "x"), ("q", "y"), ("w", "x"), ("w", "z")];
const SMALL_DEFAULT_JAC_SCORES: &[f64] = &[0.0, 0.5, 0.3333333333333333, 0.5, 0.3333333333333333];

fn parse_pairs(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let l = l.trim();
            if l.is_empty() || l.starts_with('#') {
                return None;
            }
            let mut it = l.split_whitespace();
            Some((
                it.next().unwrap().to_string(),
                it.next().unwrap().to_string(),
            ))
        })
        .collect()
}

fn run(graph: &str, pairs_text: &str, method: Method) -> Vec<Prediction> {
    run_alpha(graph, pairs_text, method, 0.8)
}

fn run_alpha(graph: &str, pairs_text: &str, method: Method, alpha: f64) -> Vec<Prediction> {
    let pairs = parse_pairs(pairs_text);
    link_prediction_from_edge_list(graph, method, Some(&pairs), alpha).expect("scoring")
}

fn assert_floats(got: &[Prediction], golden: &[f64]) {
    assert_eq!(got.len(), golden.len(), "row count");
    for (i, (p, &g)) in got.iter().zip(golden).enumerate() {
        assert!(
            (p.score - g).abs() <= EPS,
            "row {i} ({} {}): got {} want {} (|Δ|={:e})",
            p.u,
            p.v,
            p.score,
            g,
            (p.score - g).abs()
        );
    }
}

fn assert_ints(got: &[Prediction], golden: &[u64]) {
    assert_eq!(got.len(), golden.len(), "row count");
    for (i, (p, &g)) in got.iter().zip(golden).enumerate() {
        assert_eq!(p.score as u64, g, "row {i} ({} {})", p.u, p.v);
    }
}

macro_rules! case {
    ($name:ident, $graph:ident, $pairs:ident, $method:expr, floats $golden:ident) => {
        #[test]
        fn $name() {
            assert_floats(&run($graph, $pairs, $method), $golden);
        }
    };
    ($name:ident, $graph:ident, $pairs:ident, $method:expr, ints $golden:ident) => {
        #[test]
        fn $name() {
            assert_ints(&run($graph, $pairs, $method), $golden);
        }
    };
    ($name:ident, $graph:ident, $pairs:ident, $alpha:expr, ccpa $golden:ident) => {
        #[test]
        fn $name() {
            assert_floats(
                &run_alpha($graph, $pairs, Method::CommonNeighborCentrality, $alpha),
                $golden,
            );
        }
    };
}

case!(hand_jaccard, HAND, HAND_PAIRS, Method::Jaccard, floats HAND_JACCARD);
case!(hand_adamic_adar, HAND, HAND_PAIRS, Method::AdamicAdar, floats HAND_ADAMIC_ADAR);
case!(hand_resource_allocation, HAND, HAND_PAIRS, Method::ResourceAllocation, floats HAND_RESOURCE_ALLOCATION);
case!(hand_preferential_attachment, HAND, HAND_PAIRS, Method::PreferentialAttachment, ints HAND_PREFERENTIAL_ATTACHMENT);

case!(karate_jaccard, KARATE, KARATE_PAIRS, Method::Jaccard, floats KARATE_JACCARD);
case!(karate_adamic_adar, KARATE, KARATE_PAIRS, Method::AdamicAdar, floats KARATE_ADAMIC_ADAR);
case!(karate_resource_allocation, KARATE, KARATE_PAIRS, Method::ResourceAllocation, floats KARATE_RESOURCE_ALLOCATION);
case!(karate_preferential_attachment, KARATE, KARATE_PAIRS, Method::PreferentialAttachment, ints KARATE_PREFERENTIAL_ATTACHMENT);

case!(gnm1_jaccard, GNM1, GNM1_PAIRS, Method::Jaccard, floats GNM1_JACCARD);
case!(gnm1_adamic_adar, GNM1, GNM1_PAIRS, Method::AdamicAdar, floats GNM1_ADAMIC_ADAR);
case!(gnm1_resource_allocation, GNM1, GNM1_PAIRS, Method::ResourceAllocation, floats GNM1_RESOURCE_ALLOCATION);
case!(gnm1_preferential_attachment, GNM1, GNM1_PAIRS, Method::PreferentialAttachment, ints GNM1_PREFERENTIAL_ATTACHMENT);

case!(gnm2_jaccard, GNM2, GNM2_PAIRS, Method::Jaccard, floats GNM2_JACCARD);
case!(gnm2_adamic_adar, GNM2, GNM2_PAIRS, Method::AdamicAdar, floats GNM2_ADAMIC_ADAR);
case!(gnm2_resource_allocation, GNM2, GNM2_PAIRS, Method::ResourceAllocation, floats GNM2_RESOURCE_ALLOCATION);
case!(gnm2_preferential_attachment, GNM2, GNM2_PAIRS, Method::PreferentialAttachment, ints GNM2_PREFERENTIAL_ATTACHMENT);

case!(hand_ccpa_08, HAND, HAND_PAIRS, 0.8, ccpa HAND_CCPA_08);
case!(hand_ccpa_10, HAND, HAND_PAIRS, 1.0, ccpa HAND_CCPA_10);
case!(karate_ccpa_08, KARATE, KARATE_PAIRS, 0.8, ccpa KARATE_CCPA_08);
case!(karate_ccpa_10, KARATE, KARATE_PAIRS, 1.0, ccpa KARATE_CCPA_10);
case!(gnm1_ccpa_08, GNM1, GNM1_PAIRS, 0.8, ccpa GNM1_CCPA_08);
case!(gnm1_ccpa_10, GNM1, GNM1_PAIRS, 1.0, ccpa GNM1_CCPA_10);
case!(gnm2_ccpa_08, GNM2, GNM2_PAIRS, 0.8, ccpa GNM2_CCPA_08);
case!(gnm2_ccpa_10, GNM2, GNM2_PAIRS, 1.0, ccpa GNM2_CCPA_10);
case!(disc_ccpa_08, DISC, DISC_PAIRS, 0.8, ccpa DISC_CCPA_08);
case!(disc_ccpa_10, DISC, DISC_PAIRS, 1.0, ccpa DISC_CCPA_10);

case!(selfloop_jaccard, SELFLOOP, SELFLOOP_PAIRS, Method::Jaccard, floats SELFLOOP_JACCARD);
case!(selfloop_adamic_adar, SELFLOOP, SELFLOOP_PAIRS, Method::AdamicAdar, floats SELFLOOP_ADAMIC_ADAR);
case!(selfloop_resource_allocation, SELFLOOP, SELFLOOP_PAIRS, Method::ResourceAllocation, floats SELFLOOP_RESOURCE_ALLOCATION);
case!(selfloop_preferential_attachment, SELFLOOP, SELFLOOP_PAIRS, Method::PreferentialAttachment, ints SELFLOOP_PREFERENTIAL_ATTACHMENT);
case!(selfloop_ccpa_08, SELFLOOP, SELFLOOP_PAIRS, 0.8, ccpa SELFLOOP_CCPA_08);
case!(selfloop_ccpa_10, SELFLOOP, SELFLOOP_PAIRS, 1.0, ccpa SELFLOOP_CCPA_10);

#[test]
fn small_default_all_non_edges_jaccard() {
    let got = link_prediction_from_edge_list(SMALL, Method::Jaccard, None, 0.8).expect("scoring");
    assert_eq!(
        got.len(),
        SMALL_DEFAULT_JAC_PAIRS.len(),
        "default ebunch=None non-edge count"
    );
    for (p, ((gu, gv), &gs)) in got
        .iter()
        .zip(SMALL_DEFAULT_JAC_PAIRS.iter().zip(SMALL_DEFAULT_JAC_SCORES))
    {
        assert_eq!(&p.u, gu, "pair label u");
        assert_eq!(&p.v, gv, "pair label v");
        assert!((p.score - gs).abs() <= EPS, "score {} vs {}", p.score, gs);
    }
}
