//! Criterion benchmark for hp41-core dispatch() key latency.
//!
//! Measures the compute path of dispatch() over a representative set of HP-41 ops.
//! Terminal I/O is excluded — this captures the variable-cost part of each keystroke.
//!
//! Advisory only: results do NOT gate CI builds (D-11). Benchmark CI VMs have too
//! much timing variance for absolute latency gates. Run locally on release hardware.
//!
//! Usage: just bench

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hp41_core::ops::{dispatch, Op};
use hp41_core::{CalcState, HpNum};

/// Benchmark a sequence of 20 representative ops on a fresh CalcState.
/// Covers arithmetic, stack, trig, and register ops — the common hot path.
fn bench_dispatch_mixed(c: &mut Criterion) {
    let ops: Vec<Op> = vec![
        Op::PushNum(HpNum::from(3i32)),
        Op::Enter,
        Op::PushNum(HpNum::from(4i32)),
        Op::Add,
        Op::PushNum(HpNum::from(2i32)),
        Op::Mul,
        Op::Sqrt,
        Op::Recip,
        Op::PushNum(HpNum::from(30i32)),
        Op::Sin,
        Op::Cos,
        Op::Tan,
        Op::Asin,
        Op::StoReg(0),
        Op::RclReg(0),
        Op::Sub,
        Op::Clx,
        Op::Rdn,
        Op::XySwap,
        Op::Chs,
    ];

    c.bench_function("dispatch_mixed_20ops", |b| {
        b.iter(|| {
            let mut state = CalcState::default();
            for op in &ops {
                // Ignore errors — domain errors (e.g., asin of value > 1) are expected
                // in a rapid-fire sequence; they don't affect timing accuracy.
                let _ = dispatch(&mut state, op.clone());
            }
        });
    });
}

/// Benchmark a single dispatch() call for the most common op: Add.
/// Useful for measuring the per-op overhead of dispatch() itself.
fn bench_dispatch_single_add(c: &mut Criterion) {
    c.bench_function("dispatch_single_add", |b| {
        let mut state = CalcState::default();
        // Pre-load stack so Add always has valid operands
        let _ = dispatch(&mut state, Op::PushNum(HpNum::from(1i32)));
        let _ = dispatch(&mut state, Op::Enter);
        b.iter(|| {
            let _ = dispatch(&mut state, Op::PushNum(HpNum::from(1i32)));
            let _ = dispatch(&mut state, Op::Add);
        });
    });
}

/// Benchmark 1000 dispatch() calls (equivalent to 1000 keystrokes).
/// Per D-10: reports statistical median; target is <= 50ms for the full 1000-op run
/// (i.e., <= 50 us per op, which is well within the 50ms budget per key-press).
fn bench_dispatch_1000_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_1000");
    group.sample_size(20); // criterion default is 100; 20 is sufficient for advisory

    group.bench_function(BenchmarkId::new("arithmetic", "1000x_add"), |b| {
        b.iter(|| {
            let mut state = CalcState::default();
            for i in 0..1000i32 {
                let _ = dispatch(&mut state, Op::PushNum(HpNum::from(i)));
                let _ = dispatch(&mut state, Op::Enter);
                let _ = dispatch(&mut state, Op::PushNum(HpNum::from(i + 1)));
                let _ = dispatch(&mut state, Op::Add);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_dispatch_mixed,
    bench_dispatch_single_add,
    bench_dispatch_1000_ops,
);
criterion_main!(benches);
