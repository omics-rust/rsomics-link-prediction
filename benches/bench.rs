use std::hash::{BuildHasherDefault, Hasher};

use criterion::{criterion_group, criterion_main, Criterion};
use rsomics_link_prediction::{link_prediction_from_edge_list, Method};

/// Deterministic xorshift so the bench graph is reproducible without an RNG dep.
struct XorShift(u64);
impl XorShift {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

// gnm(500, 3000) edge list, string labels, no self-loops or parallel edges.
fn gnm_edge_list(n: usize, m: usize) -> String {
    let mut rng = XorShift(0x1234_5678_9abc_def0);
    let mut seen: std::collections::HashSet<(usize, usize), BuildHasherDefault<IdHasher>> =
        Default::default();
    let mut s = String::new();
    while seen.len() < m {
        let a = rng.below(n as u64) as usize;
        let b = rng.below(n as u64) as usize;
        if a == b {
            continue;
        }
        let key = if a < b { (a, b) } else { (b, a) };
        if seen.insert(key) {
            s.push_str(&format!("n{} n{}\n", key.0, key.1));
        }
    }
    s
}

#[derive(Default)]
struct IdHasher(u64);
impl Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = self.0.wrapping_mul(0x100_0000_01b3) ^ u64::from(b);
        }
    }
}

fn fixed_pairs(n: usize, k: usize) -> Vec<(String, String)> {
    let mut rng = XorShift(0xdead_beef_cafe_babe);
    let mut out = Vec::with_capacity(k);
    while out.len() < k {
        let a = rng.below(n as u64) as usize;
        let b = rng.below(n as u64) as usize;
        if a != b {
            out.push((format!("n{a}"), format!("n{b}")));
        }
    }
    out
}

fn bench(c: &mut Criterion) {
    let graph = gnm_edge_list(500, 3000);
    let pairs = fixed_pairs(500, 2000);

    c.bench_function("jaccard_gnm500_3000_2000pairs", |b| {
        b.iter(|| {
            link_prediction_from_edge_list(
                std::hint::black_box(&graph),
                Method::Jaccard,
                Some(std::hint::black_box(&pairs)),
            )
            .unwrap()
        })
    });

    c.bench_function("adamic_adar_gnm500_3000_2000pairs", |b| {
        b.iter(|| {
            link_prediction_from_edge_list(
                std::hint::black_box(&graph),
                Method::AdamicAdar,
                Some(std::hint::black_box(&pairs)),
            )
            .unwrap()
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
