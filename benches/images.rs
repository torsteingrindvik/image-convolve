// Small input images
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use image_convolve::{
    convolution::{backends::cpu_naive::CpuNaive, strategy::prepare},
    prelude::*,
};

fn impl_bench(c: &mut Criterion, name: &str, input: &str) {
    let (input, output) = prepare(input).unwrap();

    let mut group = c.benchmark_group(name);

    for kernel in [
        Kernel::Identity,
        Kernel::EdgeDetection1,
        Kernel::EdgeDetection2,
        Kernel::Sharpen,
        Kernel::BoxBlur,
        Kernel::GaussianBlur,
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("CPU Naive", kernel),
            kernel,
            |bencher, kernel| {
                bencher.iter_batched(
                    || (input.clone(), output.clone()),
                    |(input, mut output)| CpuNaive::convolve(input, &mut output, *kernel),
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn small(c: &mut Criterion) {
    impl_bench(c, "Small Image", "images/animal.png");
}

fn medium(c: &mut Criterion) {
    impl_bench(c, "Medium Image", "images/camera.jpg");
}

fn large(c: &mut Criterion) {
    impl_bench(c, "Large Image", "images/gecko.jpg");
}

criterion_group!(benches, small, medium, large);
criterion_main!(benches);
