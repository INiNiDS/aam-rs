//! Criterion benchmarks inspired by the `standard_stress` example.
//!
//! Measures:
//! - Building AAML content with `AAMBuilder`
//! - Parsing with `AAM::parse`
//! - Loading from file with `AAM::load`
//! - Zero-copy mmap loading with `AAM::load_fast` (requires feature `aot`)
//! - Key lookup (`AAM::get`) — first, last, and miss scenarios
//! - Iteration (`AAM::iter`)
//!
//! Run with:
//! ```sh
//! cargo bench --bench stress
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

use aam_rs::aam::AAM;
use aam_rs::builder::AAMBuilder;

/// Number of key-value pairs to use in benchmarks.
const SIZES: &[usize] = &[100, 1_000, 10_000];

// ── Helpers ──────────────────────────────────────────────────────────

fn generate_content(count: usize) -> String {
    let mut builder = AAMBuilder::with_capacity(count * 40);
    for i in 0..count {
        builder.add_line(&format!("key_{i}"), &format!("value_{i}"));
    }
    builder.build()
}

fn write_to(dir: &Path, name: &str, content: &str) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, content).unwrap();
    p
}

// ── Build ────────────────────────────────────────────────────────────

fn bench_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("build");
    group.measurement_time(Duration::from_secs(10));

    for &size in SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &count| {
            b.iter(|| {
                let mut builder = AAMBuilder::with_capacity(black_box(count * 40));
                for i in 0..count {
                    builder.add_line(
                        black_box(&format!("key_{i}")),
                        black_box(&format!("value_{i}")),
                    );
                }
                black_box(builder.build());
            });
        });
    }
    group.finish();
}

// ── Parse (from string) ──────────────────────────────────────────────

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    group.measurement_time(Duration::from_secs(10));

    // Leak content strings so the references live for the entire benchmark.
    let contents: &'static [(usize, String)] = Box::leak(
        SIZES
            .iter()
            .map(|&size| (size, generate_content(size)))
            .collect(),
    );

    for (size, content) in contents {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), content, |b, text| {
            b.iter(|| black_box(AAM::parse(text)).unwrap());
        });
    }
    group.finish();
}

// ── Load from file (with AOT caching) ────────────────────────────────

fn bench_load(c: &mut Criterion) {
    let mut group = c.benchmark_group("load");
    group.measurement_time(Duration::from_secs(10));

    let tmp: &'static tempfile::TempDir = Box::leak(Box::new(tempfile::TempDir::new().unwrap()));

    for &size in SIZES {
        let content = generate_content(size);
        let path = write_to(tmp.path(), &format!("{size}.aam"), &content);
        // Force AOT cache to be built so load uses it.
        AAM::cook(&path).unwrap();
    }

    for &size in SIZES {
        let path = tmp.path().join(format!("{size}.aam"));
        group
            .throughput(Throughput::Elements(size as u64))
            .bench_with_input(BenchmarkId::from_parameter(size), &(), |b, _| {
                b.iter(|| black_box(AAM::load(&path)).unwrap());
            });
    }
    group.finish();
}

// ── Load_fast — AOT zero-copy mmap ───────────────────────────────────

#[cfg(feature = "aot")]
fn bench_load_fast(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_fast");
    group.measurement_time(Duration::from_secs(10));

    let tmp: &'static tempfile::TempDir = Box::leak(Box::new(tempfile::TempDir::new().unwrap()));

    for &size in SIZES {
        let content = generate_content(size);
        let path = write_to(tmp.path(), &format!("{size}.aam"), &content);
        // Pre-cook so load_fast hits the cache.
        AAM::cook(&path).unwrap();
    }

    for &size in SIZES {
        let path = tmp.path().join(format!("{size}.aam"));
        group
            .throughput(Throughput::Elements(size as u64))
            .bench_with_input(BenchmarkId::from_parameter(size), &(), |b, _| {
                b.iter(|| black_box(AAM::load_fast(&path)).unwrap());
            });
    }
    group.finish();
}

// ── Lookup ───────────────────────────────────────────────────────────

fn bench_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup");
    group.measurement_time(Duration::from_secs(10));

    let data: &'static [(usize, AAM, String)] = Box::leak(
        SIZES
            .iter()
            .map(|&size| {
                let content = generate_content(size);
                let aam = AAM::parse(&content).unwrap();
                let last_key = format!("key_{}", size - 1);
                (size, aam, last_key)
            })
            .collect(),
    );

    for (size, aam, last_key) in data {
        group.bench_function(BenchmarkId::new("first", size), |b| {
            b.iter(|| black_box(aam.get("key_0")))
        });
        group.bench_function(BenchmarkId::new("last", size), |b| {
            b.iter(|| black_box(aam.get(last_key)))
        });
        group.bench_function(BenchmarkId::new("miss", size), |b| {
            b.iter(|| black_box(aam.get("nonexistent_key")))
        });
    }
    group.finish();
}

// ── Iterate ──────────────────────────────────────────────────────────

fn bench_iterate(c: &mut Criterion) {
    let mut group = c.benchmark_group("iterate");
    group.measurement_time(Duration::from_secs(10));

    let data: &'static [(usize, AAM)] = Box::leak(
        SIZES
            .iter()
            .map(|&size| {
                let content = generate_content(size);
                let aam = AAM::parse(&content).unwrap();
                (size, aam)
            })
            .collect(),
    );

    for (size, aam) in data {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_function(BenchmarkId::from_parameter(size), |b| {
            b.iter(|| for _ in black_box(aam.iter()) {});
        });
    }
    group.finish();
}

// ── Registration ─────────────────────────────────────────────────────

#[cfg(feature = "aot")]
criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10));
    targets = bench_build, bench_parse, bench_load, bench_load_fast, bench_lookup, bench_iterate
}

#[cfg(not(feature = "aot"))]
criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10));
    targets = bench_build, bench_parse, bench_load, bench_lookup, bench_iterate
}

criterion_main!(benches);
