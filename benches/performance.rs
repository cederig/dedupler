use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs;
use std::io::Write;
use std::hint::black_box;
use tempfile::TempDir;

// Import from the dedupler library
use dedupler::{process_file, process_file_parallel, EncodingCache};

fn create_test_file(dir: &TempDir, name: &str, lines: usize, duplicate_ratio: f64) -> std::path::PathBuf {
    let file_path = dir.path().join(name);
    let mut file = fs::File::create(&file_path).unwrap();
    
    let unique_lines = (lines as f64 * (1.0 - duplicate_ratio)) as usize;
    
    for i in 0..unique_lines {
        writeln!(file, "line_{}", i).unwrap();
    }
    
    // Add duplicates
    for i in 0..(lines - unique_lines) {
        writeln!(file, "line_{}", i % unique_lines).unwrap();
    }
    
    file_path
}

fn bench_sequential_processing(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let _encoding_cache = EncodingCache::default();
    
    let mut group = c.benchmark_group("sequential_processing");
    
    for size in [100, 1000, 10000, 100000].iter() {
        let file_path = create_test_file(&temp_dir, &format!("test_{}.txt", size), *size, 0.3);
        
        group.bench_with_input(
            BenchmarkId::new("process_file", size),
            size,
            |b, _| {
                b.iter(|| {
                    let output_path = temp_dir.path().join("output.txt");
                    process_file(black_box(&file_path), Some(black_box(&output_path))).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

fn bench_parallel_processing(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let encoding_cache = EncodingCache::default();
    
    let mut group = c.benchmark_group("parallel_processing");
    
    for size in [100, 1000, 10000, 100000].iter() {
        let file_path = create_test_file(&temp_dir, &format!("test_{}.txt", size), *size, 0.3);
        
        group.bench_with_input(
            BenchmarkId::new("process_file_parallel", size),
            size,
            |b, _| {
                b.iter(|| {
                    let output_path = temp_dir.path().join("output_parallel.txt");
                    process_file_parallel(
                        black_box(&file_path), 
                        Some(black_box(&output_path)),
                        black_box(&encoding_cache),
                        black_box(100), // max_memory_mb
                        black_box(false) // streaming_mode
                    ).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

fn bench_encoding_cache(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let encoding_cache = EncodingCache::default();
    
    let file_path = create_test_file(&temp_dir, "cache_test.txt", 1000, 0.3);
    
    let mut group = c.benchmark_group("encoding_cache");
    
    // Benchmark without cache
    group.bench_function("without_cache", |b| {
        b.iter(|| {
            let output_path = temp_dir.path().join("output_no_cache.txt");
            process_file(black_box(&file_path), Some(black_box(&output_path))).unwrap();
        });
    });
    
    // Benchmark with cache
    group.bench_function("with_cache", |b| {
        b.iter(|| {
            let output_path = temp_dir.path().join("output_cache.txt");
            process_file_parallel(
                black_box(&file_path), 
                Some(black_box(&output_path)),
                black_box(&encoding_cache),
                black_box(100),
                black_box(false)
            ).unwrap();
        });
    });
    
    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let encoding_cache = EncodingCache::default();
    
    let mut group = c.benchmark_group("memory_usage");
    
    for memory_mb in [10, 50, 100, 500].iter() {
        let file_path = create_test_file(&temp_dir, &format!("memory_test_{}.txt", memory_mb), 50000, 0.3);
        
        group.bench_with_input(
            BenchmarkId::new("streaming_mode", memory_mb),
            memory_mb,
            |b, &memory_mb| {
                b.iter(|| {
                    let output_path = temp_dir.path().join("output_memory.txt");
                    process_file_parallel(
                        black_box(&file_path), 
                        Some(black_box(&output_path)),
                        black_box(&encoding_cache),
                        black_box(memory_mb),
                        black_box(true) // streaming_mode enabled
                    ).unwrap();
                });
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_sequential_processing,
    bench_parallel_processing,
    bench_encoding_cache,
    bench_memory_usage
);
criterion_main!(benches);
