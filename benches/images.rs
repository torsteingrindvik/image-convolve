// Small input images
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use image_convolve::{
    convolution::{
        backends::{
            cpu,
            gpu::{self, offscreen::context::GpuCtx},
        },
        strategy::prepare,
    },
    prelude::*,
};

fn impl_bench(c: &mut Criterion, name: &str, input: &str) {
    let input = prepare(input).unwrap();
    let gpu_ctx = GpuCtx::new(input.clone()).unwrap();

    let mut group = c.benchmark_group(name);

    for kernel in [
        Kernel::Identity,
        Kernel::EdgeDetection1,
        Kernel::GaussianBlur,
    ]
    .iter()
    {
        group.bench_with_input(
            BenchmarkId::new("CPU Single Loops", kernel),
            kernel,
            |bencher, kernel| {
                bencher.iter_batched(
                    || cpu::single::NestedLoops::from((input.clone(), *kernel)),
                    |mut backend| backend.convolve(),
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("CPU Single Iterators", kernel),
            kernel,
            |bencher, kernel| {
                bencher.iter_batched(
                    || cpu::single::NestedIterators::from((input.clone(), *kernel)),
                    |mut backend| backend.convolve(),
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("CPU Multi Rayon", kernel),
            kernel,
            |bencher, kernel| {
                bencher.iter_batched(
                    || cpu::multi::NestedIterators::from((input.clone(), *kernel)),
                    |mut backend| backend.convolve(),
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("GPU Offscreen", kernel),
            kernel,
            |bencher, kernel| {
                bencher.iter_batched(
                    || gpu::offscreen::Offscreen::new(gpu_ctx.clone(), *kernel).unwrap(),
                    |mut backend| backend.convolve(),
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

fn res_1280x720(c: &mut Criterion) {
    impl_bench(c, "1280x720", "images/1280x720.jpg");
}

fn res_1920x1080(c: &mut Criterion) {
    impl_bench(c, "1920x1080", "images/1920x1080.jpg");
}

fn res_3840x2160(c: &mut Criterion) {
    impl_bench(c, "3840x2160", "images/3840x2160.jpg");
}

criterion_group!(benches, res_1280x720, res_1920x1080, res_3840x2160);
criterion_main!(benches);
