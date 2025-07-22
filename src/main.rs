use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;
use ignore::WalkBuilder;
use chardet;
use encoding_rs;
use encoding_rs_io::DecodeReaderBytesBuilder;

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

    /// Globs of files/directories to ignore. Can be used multiple times.
    #[arg(long)]
    ignore: Vec<String>,
}

/// Execution statistics for a file processing operation.
#[derive(Debug, Default)]
struct Stats {
    total_lines: u64,
    duplicate_lines: u64,
    lines_written: u64,
    duration: std::time::Duration,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let start_time = Instant::now();
    let mut total_stats = Stats::default();

    if let Some(dir_path) = args.directory {
        // Process a directory
        let mut walk_builder = WalkBuilder::new(&dir_path);
        walk_builder.hidden(false); // Process hidden files by default unless ignored

        for pattern in &args.ignore {
            walk_builder.add_ignore(pattern);
        }

        let files_to_process: Vec<_> = walk_builder.build()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
            .map(|e| e.into_path())
            .collect();

        println!("Found {} files to process in directory.", files_to_process.len());

        for file_path in files_to_process {
            let output_path = args.output.as_ref().map(|o| {
                // Create a structured output directory if a single output file is not specified
                let file_name = file_path.file_name().unwrap();
                o.join(file_name)
            });

             match process_file(&file_path, output_path.as_deref()) {
                Ok(stats) => {
                    if args.stat {
                        println!("\nStats for {}:", file_path.display());
                        print_stats(&stats);
                    }
                    total_stats.total_lines += stats.total_lines;
                    total_stats.duplicate_lines += stats.duplicate_lines;
                    total_stats.lines_written += stats.lines_written;
                }
                Err(e) => eprintln!("Error processing file {}: {}", file_path.display(), e),
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

/// Processes a single file to remove duplicate lines, handling various file encodings gracefully.
///
/// This function detects the file encoding and decodes it to UTF-8 on the fly.
///
/// # Arguments
///
/// * `input_path` - The path to the file to process.
/// * `output_path` - Optional path to an output file. If None, prints to stdout.
///
/// # Returns
///
/// A `Result` containing the `Stats` of the operation or an `io::Error`.
fn process_file(input_path: &Path, output_path: Option<&Path>) -> io::Result<Stats> {
    let mut file = File::open(input_path)?;
    let file_size = file.metadata()?.len();

    // Buffer to read a chunk of the file for encoding detection
    let mut buffer = Vec::new();
    let mut temp_file = file.try_clone()?;
    temp_file.read_to_end(&mut buffer)?;
    drop(temp_file);

    // Detect encoding
    let (encoding, ..) = chardet::detect(&buffer);
    let encoding = encoding_rs::Encoding::for_label(encoding.as_bytes()).unwrap_or(encoding_rs::UTF_8);

    // Rewind the file to the beginning before processing
    file = File::open(input_path)?;

    let mut reader = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding))
            .build(file)
    );

    let mut writer: Box<dyn Write> = if let Some(path) = output_path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Box::new(BufWriter::new(File::create(path)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

    let mut seen_lines = HashSet::new();
    let mut stats = Stats::default();
    let start_time = Instant::now();

    let pb = ProgressBar::new(file_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>- "));

    let mut line = String::new();
    let mut bytes_read = 0;

    while reader.read_line(&mut line)? > 0 {
        bytes_read += line.as_bytes().len() as u64;
        pb.set_position(bytes_read);

        let trimmed_line = line.trim_end();

        stats.total_lines += 1;
        if seen_lines.insert(trimmed_line.to_string()) {
            writeln!(writer, "{}", trimmed_line)?;
            stats.lines_written += 1;
        } else {
            stats.duplicate_lines += 1;
        }

        line.clear();
    }

    pb.finish_with_message("done");
    stats.duration = start_time.elapsed();
    Ok(stats)
}

/// Prints the statistics to the console.
fn print_stats(stats: &Stats) {
    println!("  Total lines read: {}", stats.total_lines);
    println!("  Duplicate lines found: {}", stats.duplicate_lines);
    println!("  Lines written: {}", stats.lines_written);
    println!("  Duration: {:.2?}", stats.duration);
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    // Helper function to create a temporary file with content
    fn create_temp_file(content: &str) -> io::Result<NamedTempFile> {
        let file = NamedTempFile::new()?;
        fs::write(file.path(), content)?;
        Ok(file)
    }

    // Helper function to create a temp file with byte content
    fn create_temp_file_bytes(content: &[u8]) -> io::Result<NamedTempFile> {
        let file = NamedTempFile::new()?;
        fs::write(file.path(), content)?;
        Ok(file)
    }

    #[test]
    fn test_process_file_no_duplicates() -> io::Result<()> {
        let input_file = create_temp_file("line1\nline2\nline3")?;
        let output_file = NamedTempFile::new()?;

        let stats = process_file(input_file.path(), Some(output_file.path()))?;

        assert_eq!(stats.total_lines, 3);
        assert_eq!(stats.duplicate_lines, 0);
        assert_eq!(stats.lines_written, 3);

        let output_content = fs::read_to_string(output_file.path())?;
        assert_eq!(output_content, "line1\nline2\nline3\n");

        Ok(())
    }

    #[test]
    fn test_process_file_with_duplicates() -> io::Result<()> {
        let input_file = create_temp_file("apple\nbanana\napple\norange\nbanana")?;
        let output_file = NamedTempFile::new()?;

        let stats = process_file(input_file.path(), Some(output_file.path()))?;

        assert_eq!(stats.total_lines, 5);
        assert_eq!(stats.duplicate_lines, 2);
        assert_eq!(stats.lines_written, 3);

        let output_content = fs::read_to_string(output_file.path())?;
        assert_eq!(output_content, "apple\nbanana\norange\n");

        Ok(())
    }

    #[test]
    fn test_process_empty_file() -> io::Result<()> {
        let input_file = create_temp_file("")?;
        let output_file = NamedTempFile::new()?;

        let stats = process_file(input_file.path(), Some(output_file.path()))?;

        assert_eq!(stats.total_lines, 0);
        assert_eq!(stats.duplicate_lines, 0);
        assert_eq!(stats.lines_written, 0);

        let output_content = fs::read_to_string(output_file.path())?;
        assert_eq!(output_content, "");

        Ok(())
    }

    #[test]
    fn test_process_file_with_blank_lines() -> io::Result<()> {
        let input_file = create_temp_file("a\n\nb\n\na")?;
        let output_file = NamedTempFile::new()?;

        let stats = process_file(input_file.path(), Some(output_file.path()))?;

        // The lines are: "a", "", "b", "", "a"
        assert_eq!(stats.total_lines, 5);
        assert_eq!(stats.duplicate_lines, 2); // The second "" and the second "a" are duplicates
        assert_eq!(stats.lines_written, 3); // "a", "", "b" are written

        let output_content = fs::read_to_string(output_file.path())?;
        assert_eq!(output_content, "a\n\nb\n");

        Ok(())
    }

    #[test]
    fn test_process_file_with_windows1252() -> io::Result<()> {
        // Simulate a file with Windows-1252 encoding
        let content = b"h\xe9llo\nworld\nh\xe9llo"; // héllo world héllo
        let input_file = create_temp_file_bytes(content)?;
        let output_file = NamedTempFile::new()?;

        let stats = process_file(input_file.path(), Some(output_file.path()))?;

        assert_eq!(stats.total_lines, 3);
        assert_eq!(stats.duplicate_lines, 1);
        assert_eq!(stats.lines_written, 2);

        let output_content = fs::read_to_string(output_file.path())?;
        assert_eq!(output_content, "héllo\nworld\n");

        Ok(())
    }
}