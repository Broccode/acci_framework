# Performance Testing Guide

## Overview

Performance testing is crucial for ensuring that our application meets its performance requirements. This guide covers various types of performance tests, tools, and best practices for measuring and optimizing application performance.

## Key Concepts

1. **Benchmark Tests**
   - Measure execution time
   - Compare different implementations
   - Track performance regressions

2. **Load Tests**
   - Simulate multiple users
   - Test system under load
   - Measure response times

3. **Stress Tests**
   - Test system limits
   - Resource exhaustion scenarios
   - Recovery behavior

## Using Criterion.rs

### Basic Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn bench_fibonacci(c: &mut Criterion) {
    c.bench_function("fibonacci 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, bench_fibonacci);
criterion_main!(benches);
```

### Parameterized Benchmarks

```rust
fn bench_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorting");
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("vec_sort", size),
            size,
            |b, &size| {
                let mut vec: Vec<i32> = (0..size).collect();
                b.iter(|| {
                    vec.shuffle(&mut rand::thread_rng());
                    vec.sort();
                })
            },
        );
    }
    group.finish();
}
```

## Test Categories

### 1. CPU Performance

```rust
fn bench_computation(c: &mut Criterion) {
    c.bench_function("complex_calculation", |b| {
        b.iter(|| {
            // CPU-intensive operation
            perform_complex_calculation()
        })
    });
}
```

### 2. Memory Usage

```rust
fn bench_memory(c: &mut Criterion) {
    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let mut data = Vec::with_capacity(1000);
            for i in 0..1000 {
                data.push(i);
            }
            black_box(data)
        })
    });
}
```

### 3. I/O Performance

```rust
fn bench_io(c: &mut Criterion) {
    c.bench_function("file_read", |b| {
        b.iter(|| {
            let mut file = File::open("test_data.txt").unwrap();
            let mut buffer = String::new();
            file.read_to_string(&mut buffer).unwrap();
            black_box(buffer)
        })
    });
}
```

## Advanced Techniques

### 1. Async Benchmarks

```rust
fn bench_async(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("async_operation", |b| {
        b.to_async(&rt).iter(|| async {
            let result = async_operation().await;
            black_box(result)
        })
    });
}
```

### 2. Custom Measurements

```rust
fn bench_custom(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_metrics");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    group.bench_function("operation", |b| {
        b.iter(|| operation_to_measure())
    });
    group.finish();
}
```

## Best Practices

1. **Benchmark Environment**
   - Use dedicated hardware
   - Minimize background processes
   - Consistent system state

2. **Test Design**
   - Realistic workloads
   - Representative data sets
   - Proper warm-up periods

3. **Metrics Collection**
   - Throughput measurements
   - Latency distributions
   - Resource utilization

4. **Analysis**
   - Statistical significance
   - Performance regression detection
   - Trend analysis

## Running Tests

1. Run all benchmarks:

   ```bash
   cargo bench
   ```

2. Run specific benchmark:

   ```bash
   cargo bench --bench my_benchmark
   ```

3. Run with custom configuration:

   ```bash
   CRITERION_DEBUG=1 cargo bench
   ```

## Common Patterns

### Setup and Teardown

```rust
fn bench_with_setup(c: &mut Criterion) {
    c.bench_function("operation_with_setup", |b| {
        b.iter_with_setup(
            || setup_test_data(),
            |test_data| {
                perform_operation(test_data)
            }
        )
    });
}
```

### Comparing Implementations

```rust
fn bench_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("implementations");
    group.bench_function("implementation_a", |b| {
        b.iter(|| implementation_a())
    });
    group.bench_function("implementation_b", |b| {
        b.iter(|| implementation_b())
    });
    group.finish();
}
```

## Performance Monitoring

### 1. Metrics Collection

```rust
use metrics::{counter, gauge, histogram};

fn track_performance() {
    counter!("api.requests.total").increment(1);
    gauge!("system.memory.usage").set(get_memory_usage());
    histogram!("api.response.time").record(response_time);
}
```

### 2. Continuous Monitoring

```rust
fn setup_monitoring() {
    metrics_exporter_prometheus::install().expect("failed to install prometheus exporter");
    
    tokio::spawn(async move {
        collect_metrics().await;
    });
}
```

## Further Reading

- [Criterion.rs Documentation](https://docs.rs/criterion)
- [Performance Testing Best Practices](https://rust-lang.github.io/rust-performance-book/)
- [System Monitoring Guide](../monitoring/README.md)
- [Benchmarking Guide](../benchmarking/README.md)
