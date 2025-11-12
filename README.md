# Dedupler

`dedupler` is a simple and fast command-line tool written in Rust to remove duplicate lines from files. It can process a single file or an entire directory, with options for outputting to a file or the terminal, ignoring specific files, and displaying execution statistics.

## Features

- Fast deduplication using `HashSet` for efficient line processing.
- Parallel processing for directories using Rayon to significantly speed up large-scale operations.
- Recursive directory processing to find and handle files.
- Support for `.gitignore` patterns and custom ignore rules via the `ignore` crate.
- Flexible output to a specified file or standard output.
- Visual feedback with a progress bar from `indicatif`.
- Detailed execution statistics, including lines read, duplicates, and processing time.
- Cross-platform compatibility with Linux, macOS, and Windows.
- Robust handling of various file encodings like UTF-8, UTF-16, and Windows-1252.
- Memory-efficient streaming mode for processing very large files with controlled memory usage.
- Encoding cache to avoid redundant encoding detection across similar files.
- Performance benchmarks using Criterion for measuring and validating optimizations.

## Dependencies

This project uses the following Rust dependencies (as defined in `Cargo.toml`):

- `clap` (version `4.5.51`) : For command-line argument parsing.
- `indicatif` (version `0.18.3`) : For displaying a progress bar.
- `encoding_rs` (version `0.8.35`) : For file encoding management.
- `encoding_rs_io` (version `0.1.7`) : For reading files with different encodings.
- `ignore` (version `0.4.25`) : For ignoring files and directories.
- `rayon` (version `1.11.0`) : For parallel processing of directories.
- `chardet` (version `0.2.4`) : For automatic character encoding detection.

### Development Dependencies

- `tempfile` (version `3.23.0`) : For creating temporary files and directories in tests.
- `criterion` (version `0.7.0`) : For performance benchmarking and profiling.

## Installation

### Prerequisites

Make sure you have Rust and Cargo installed on your system. You can install them by following the instructions on the official Rust website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Compiling for Linux (from Linux)
1.  Clone this repository:
    ```sh
    git clone https://github.com/cederig/dedupler.git
    cd dedupler
    ```
2.  Compile the project:
    ```sh
    cargo build --release
    ```
    The executable will be located in `target/release/dedupler`.

### Compiling for Windows (from Linux/macOS)

To cross-compile this project for Windows from another operating system (like Linux or macOS), you will need the Rust target for Windows.

1.  Add the Windows target to your Rust installation:
    ```sh
    rustup target add x86_64-pc-windows-gnu
    ```

2.  Compile the project for the Windows target:
    ```sh
    cargo build --release --target=x86_64-pc-windows-gnu
    ```

The Windows executable will be located in `target/x86_64-pc-windows-gnu/release/dedupler.exe`.

### Compiling for macOS (from Linux/macOS)

To cross-compile this project for macOS from another operating system (like Linux or macOS), you will need the Rust target for macOS.

1.  Add the macOS target to your Rust installation (choose the correct architecture):
    *   For Intel Macs (x86_64):
        ```sh
        rustup target add x86_64-apple-darwin
        ```
    *   For Apple Silicon Macs (aarch64):
        ```sh
        rustup target add aarch64-apple-darwin
        ```

2.  Compile the project for the macOS target (choose the correct architecture):
    *   For Intel Macs:
        ```sh
        cargo build --release --target=x86_64-apple-darwin
        ```
    *   For Apple Silicon Macs:
        ```sh
        cargo build --release --target=aarch64-apple-darwin
        ```

The macOS executable will be located in `target/<your_mac_target>/release/dedupler`.

## Usage

```sh
dedupler [OPTIONS] [FILE]
```

### Arguments

-   `[FILE]` : The input file to process. Cannot be used with `-d` / `--directory`.

### Options

-   `-d, --directory <DIRECTORY>` : Process all files in the specified directory. Cannot be used with `[FILE]`.
-   `-o, --output <OUTPUT>` : Path to the output file. If not provided, results are printed to the terminal. When processing a directory, this specifies an output directory to mirror the input structure.
-   `--stat` : Show detailed execution statistics.
-   `--parallel <BOOLEAN>` : Enable parallel processing for directories (default: true). Significantly speeds up processing of multiple files.
-   `--max-memory-mb <SIZE>` : Maximum memory usage for HashSet in MB (default: 100). Useful when processing very large files.
-   `--streaming` : Enable streaming mode for large files (default: false). Reduces memory footprint at the cost of slightly slower processing.
-   `--ignore <PATTERN>` : A glob pattern of files/directories to ignore. Can be specified multiple times. (e.g., `--ignore '*.log' --ignore 'tmp/'`)
-   `-h, --help` : Print help information.
-   `-V, --version` : Print version information.

## Examples

-   Deduplicate a single file and print to terminal:
    ```sh
    dedupler my_file.txt
    ```

-   Deduplicate a file and save to another file:
    ```sh
    dedupler my_file.txt -o my_file_deduplicated.txt
    ```

-   Deduplicate a file and show stats:
    ```sh
    dedupler my_file.txt --stat
    ```

-   Deduplicate all files in a directory and save them to a new directory:
    ```sh
    mkdir output_dir
    dedupler -d ./source_dir -o ./output_dir
    ```

-   Deduplicate a directory, ignoring log files and the `temp` subdirectory:
    ```sh
    dedupler -d ./my_project --ignore '*.log' --ignore 'temp/'
    ```

-   Process a directory with parallel processing (default behavior):
    ```sh
    dedupler -d ./large_project --parallel true --stat
    ```

-   Process very large files with limited memory usage:
    ```sh
    dedupler huge_file.txt --streaming --max-memory-mb 50 --stat
    ```

-   Disable parallel processing for sequential execution:
    ```sh
    dedupler -d ./small_project --parallel false
    ```

## Ignoring Files

`deduple` automatically respects rules defined in `.gitignore` and `.ignore` files in the directory being processed.

You can add more ignore patterns using the `--ignore` flag.

For example:
-   `--ignore '*.tmp'`: Ignores all files with the `.tmp` extension.
-   `--ignore 'logs/'`: Ignores the `logs` directory.
-   `--ignore '**/temp*'`: Ignores all files and directories starting with `temp` in any subdirectory.

## Tests

This project includes unit tests; to run them, use the following command at the project root:

```sh
cargo test
```

This command compiles the program in test mode and executes all test functions.

## Performance and Benchmarks

### Performance Optimizations

The tool includes several performance optimizations:

- **Parallel Processing**: Uses Rayon to process multiple files simultaneously, significantly reducing processing time for directories.
- **Memory Management**: Intelligent HashSet capacity allocation and streaming mode for memory-efficient processing.
- **Encoding Cache**: Caches encoding detection results to avoid redundant operations.
- **Optimized I/O**: Reduced progress bar overhead and reusable string buffers.

### Running Benchmarks

To run performance benchmarks and measure the effectiveness of optimizations:

```sh
cargo bench
```

This will:
- Compare sequential vs parallel processing performance
- Measure encoding cache effectiveness  
- Test memory usage with different configurations
- Generate detailed HTML reports in `target/criterion/`

### Expected Performance Gains

- **Parallel Processing**: 2-8x speedup for directories with multiple files (depending on CPU cores)
- **Memory Usage**: 50-90% reduction in memory usage with streaming mode for large files
- **Encoding Detection**: 80-95% faster processing of similar files with cache

### Performance Tips

1. Use parallel processing for directories with many files: `--parallel true`
2. Enable streaming mode for files > 100MB: `--streaming`
3. Adjust memory limit based on available RAM: `--max-memory-mb 200`
4. Use SSD storage for better I/O performance with large files