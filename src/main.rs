use clap::Parser;
use std::path::{PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use ignore::WalkBuilder;
use rayon::prelude::*;

// Import from our library
use dedupler::{process_file, process_file_parallel, EncodingCache, Stats};

/// A tool to deduplicate lines from files.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file path to process.
    #[arg()]
    file: Option<PathBuf>,

    /// Directory to process. Deduplicates each file found in the directory.
    #[arg(short, long, conflicts_with = "file")]
    directory: Option<PathBuf>,

    /// Output file path. If not provided, results are printed to the terminal.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show execution statistics.
    #[arg(long)]
    stat: bool,

    /// Enable parallel processing for directories (default: true).
    #[arg(long, default_value = "true")]
    parallel: bool,

    /// Maximum memory usage for HashSet in MB (default: 100).
    #[arg(long, default_value = "100")]
    max_memory_mb: usize,

    /// Enable streaming mode for large files (default: false).
    #[arg(long)]
    streaming: bool,

    /// Globs of files/directories to ignore. Can be used multiple times.
    #[arg(long)]
    ignore: Vec<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let start_time = Instant::now();
    let mut total_stats = Stats::default();
    let encoding_cache = EncodingCache::default();

    if let Some(dir_path) = args.directory {
        // Process a directory
        let mut walk_builder = WalkBuilder::new(&dir_path);
        walk_builder.hidden(false); // Process hidden files by default unless ignored

        for pattern in &args.ignore {
            walk_builder.add_ignore(pattern);
        }

        let files_to_process: Vec<_> = walk_builder.build()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_some_and(|ft| ft.is_file()))
            .map(|e| e.into_path())
            .collect();

        println!("Found {} files to process in directory.", files_to_process.len());

        if args.parallel && files_to_process.len() > 1 {
            // Parallel processing
            let parallel_stats = Arc::new(Mutex::new(Stats::default()));
            let encoding_cache = Arc::new(encoding_cache);
            
            files_to_process.par_iter().for_each(|file_path| {
                let output_path = args.output.as_ref().map(|o| {
                    let file_name = file_path.file_name().unwrap();
                    o.join(file_name)
                });

                match process_file_parallel(
                    file_path, 
                    output_path.as_deref(),
                    &encoding_cache,
                    args.max_memory_mb,
                    args.streaming
                ) {
                    Ok(stats) => {
                        if args.stat {
                            println!("Stats for {}:", file_path.display());
                            print_stats(&stats);
                        }
                        parallel_stats.lock().unwrap().merge(&stats);
                    }
                    Err(e) => eprintln!("Error processing file {}: {}", file_path.display(), e),
                }
            });
            
            total_stats = Arc::try_unwrap(parallel_stats).unwrap().into_inner().unwrap();
        } else {
            // Sequential processing (original behavior)
            for file_path in files_to_process {
                let output_path = args.output.as_ref().map(|o| {
                    let file_name = file_path.file_name().unwrap();
                    o.join(file_name)
                });

                match process_file(&file_path, output_path.as_deref()) {
                    Ok(stats) => {
                        if args.stat {
                            println!("\nStats for {}:", file_path.display());
                            print_stats(&stats);
                        }
                        total_stats.merge(&stats);
                    }
                    Err(e) => eprintln!("Error processing file {}: {}", file_path.display(), e),
                }
            }
        }
    } else if let Some(file_path) = args.file {
        // Process a single file
        match process_file(&file_path, args.output.as_deref()) {
            Ok(stats) => {
                total_stats = stats;
            }
            Err(e) => eprintln!("Error processing file {}: {}", file_path.display(), e),
        }
    } else {
        eprintln!("Error: You must specify an input file or a directory with -d.");
        std::process::exit(1);
    }

    total_stats.duration = start_time.elapsed();
    if args.stat {
        println!("\n--- Total Execution Stats ---");
        print_stats(&total_stats);
    }

    Ok(())
}

/// Prints the statistics to the console.
fn print_stats(stats: &Stats) {
    println!("  Total lines read: {}", stats.total_lines);
    println!("  Duplicate lines found: {}", stats.duplicate_lines);
    println!("  Lines written: {}", stats.lines_written);
    println!("  Duration: {:.2?}", stats.duration);
}