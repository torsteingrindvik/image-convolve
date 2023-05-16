// Small input images
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use image_convolve::convolution::backends::cpu_naive::CpuNaive;
use image_convolve::{convolution::strategy::prepare, prelude::*};

fn criterion_benchmark(c: &mut Criterion) {
    let (input, output) = prepare("images/animal.png").unwrap();

    let mut group = c.benchmark_group("Small Input Image");

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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
