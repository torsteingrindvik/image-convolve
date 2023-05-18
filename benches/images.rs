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
    // let backend = gpu::offscreen::Offscreen::new(ctx, kernel)?;

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
