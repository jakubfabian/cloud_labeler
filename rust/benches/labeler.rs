use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use cloud_labeler::gen_labels;

fn build_cross(nx: usize, ny: usize, nz: usize) -> Vec<bool> {
    let mut cld = vec![false; nx * ny * nz];
    let idx = |x: usize, y: usize, z: usize| x + nx * (y + ny * z);
    for z in 0..nz {
        for y in 1..ny - 1 { cld[idx(nx / 2, y, z)] = true; }
        for x in 1..nx - 1 { cld[idx(x, ny / 2, z)] = true; }
    }
    cld
}

fn build_field(nx: usize, ny: usize, nz: usize, density: f64) -> Vec<bool> {
    let period = (1.0 / density).round() as usize;
    (0..nx * ny * nz).map(|i| i % period == 0).collect()
}

fn bench_fortran_equivalent(c: &mut Criterion) {
    let mut group = c.benchmark_group("fortran_test_equivalent");
    for (nx, ny, nz) in [(10,10,1), (100,100,1), (50,50,50)] {
        let cld = build_cross(nx, ny, nz);
        group.bench_function(format!("cross_{}x{}x{}", nx, ny, nz), |b| {
            b.iter(|| gen_labels(black_box(&cld), nx, ny, nz));
        });
    }
    group.finish();
}

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling_30pct");
    for (nx, ny, nz) in [(32,32,32), (64,64,64), (128,128,32)] {
        let cld = build_field(nx, ny, nz, 0.3);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}x{}x{}", nx, ny, nz)),
            &(cld, nx, ny, nz),
            |b, (cld, nx, ny, nz)| b.iter(|| gen_labels(black_box(cld), *nx, *ny, *nz)),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_fortran_equivalent, bench_scaling);
criterion_main!(benches);
