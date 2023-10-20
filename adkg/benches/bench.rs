extern crate criterion;

use criterion::*;
use adkg::run;

fn bench_run(c: &mut Criterion) {
    // n 从 4 到 64，步长为 4
    // 满足 3f+1 <= n 条件下 f 取最大值
    let mut group = c.benchmark_group("run");
    for n in (7..=65).step_by(4) {
        let f = (n-1)/3;
        println!("n: {}, f: {}", n, f);
        group.bench_with_input(BenchmarkId::new("run", n), &n, |b, &n| b.iter(|| run(n, f)));
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_run
);
criterion_main!(benches);