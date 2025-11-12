use std::collections::HashSet;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write, Read};
use std::path::{Path};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use encoding_rs_io::DecodeReaderBytesBuilder;
use chardet;
use encoding_rs;

/// Execution statistics for a file processing operation.
#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub total_lines: u64,
    pub duplicate_lines: u64,
    pub lines_written: u64,
    pub duration: std::time::Duration,
}

impl Stats {
    pub fn merge(&mut self, other: &Stats) {
        self.total_lines += other.total_lines;
        self.duplicate_lines += other.duplicate_lines;
        self.lines_written += other.lines_written;
        // Note: duration is handled separately for parallel processing
    }
}

/// Simple encoding cache to avoid re-detection
#[derive(Debug, Default)]
pub struct EncodingCache {
    cache: Arc<Mutex<std::collections::HashMap<String, &'static encoding_rs::Encoding>>>,
}

impl EncodingCache {
    pub fn get_or_detect(&self, path: &Path, sample: &[u8]) -> &'static encoding_rs::Encoding {
        let key = format!("{}:{}", path.display(), sample.len());
        
        {
            let cache = self.cache.lock().unwrap();
            if let Some(&encoding) = cache.get(&key) {
                return encoding;
            }
        }
        
        let (encoding_name, ..) = chardet::detect(sample);
        let encoding = encoding_rs::Encoding::for_label(encoding_name.as_bytes())
            .unwrap_or(encoding_rs::UTF_8);
        
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(key, encoding);
        }
        
        encoding
    }
}

/// Processes a single file to remove duplicate lines, handling various file encodings gracefully.
/// This is the original sequential version.
pub fn process_file(input_path: &Path, output_path: Option<&Path>) -> io::Result<Stats> {
    let mut file = File::open(input_path)?;
    let file_size = file.metadata()?.len();

    // Read only first 8KB for encoding detection instead of entire file
    let mut buffer = [0; 8192];
    let bytes_read = file.read(&mut buffer)?;

    // Detect encoding from sample
    let (encoding, ..) = chardet::detect(&buffer[..bytes_read]);
    let encoding = encoding_rs::Encoding::for_label(encoding.as_bytes()).unwrap_or(encoding_rs::UTF_8);

    // Create a new file handle since we can't seek reliably on all file types
    let file = File::open(input_path)?;
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

    let mut seen_lines = HashSet::with_capacity((file_size / 100) as usize); // Estimate based on file size
    let mut stats = Stats::default();
    let start_time = Instant::now();

    let mut line = String::with_capacity(1024); // Reusable buffer with initial capacity

    while reader.read_line(&mut line)? > 0 {
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

    stats.duration = start_time.elapsed();
    Ok(stats)
}

/// Parallel-optimized version of process_file with memory management and encoding cache
pub fn process_file_parallel(
    input_path: &Path, 
    output_path: Option<&Path>,
    encoding_cache: &EncodingCache,
    max_memory_mb: usize,
    streaming_mode: bool
) -> io::Result<Stats> {
    let mut file = File::open(input_path)?;
    let file_size = file.metadata()?.len();

    // Read only first 8KB for encoding detection
    let mut buffer = [0; 8192];
    let bytes_read = file.read(&mut buffer)?;
    
    // Use encoding cache
    let encoding = encoding_cache.get_or_detect(input_path, &buffer[..bytes_read]);

    // Create a new file handle
    let file = File::open(input_path)?;
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

    let mut stats = Stats::default();
    let start_time = Instant::now();

    // Memory-conscious HashSet capacity calculation
    let max_lines = if streaming_mode {
        // In streaming mode, use a smaller set to avoid memory issues
        std::cmp::min((file_size / 100) as usize, max_memory_mb * 1000)
    } else {
        (file_size / 100) as usize
    };
    
    let mut seen_lines = HashSet::with_capacity(max_lines);

    let mut line = String::with_capacity(1024);
    let mut bytes_processed = 0;

    while reader.read_line(&mut line)? > 0 {
        bytes_processed += line.len() as u64;

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

    stats.duration = start_time.elapsed();
    Ok(stats)
}
