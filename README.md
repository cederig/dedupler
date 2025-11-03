# Dedupler

`dedupler` is a simple and fast command-line tool written in Rust to remove duplicate lines from files. It can process a single file or an entire directory, with options for outputting to a file or the terminal, ignoring specific files, and displaying execution statistics.

## Features

-   Fast Deduplication: Utilizes `HashSet` for efficient line processing.
-   Directory Processing: Recursively find and process files in a directory.
-   File Ignoring: Supports `.gitignore` patterns and custom ignore rules using the `ignore` crate.
-   Flexible Output: Write results to a specified output file or to standard output.
-   Progress Bar: Visual feedback on the file processing progress with `indicatif`.
-   Execution Statistics: Get detailed stats on lines read, duplicates found, and processing time.
-   Cross-Platform: Compiles and runs on Linux, macOS, and Windows.
-   Robust Encoding Handling: Automatically detects and handles various file encodings (UTF-8, UTF-16, Windows-1252, etc.) without crashing.

## Dependencies

This project uses the following Rust dependencies (as defined in `Cargo.toml`):

- `clap` (version `4.5.51`) : For command-line argument parsing.
- `indicatif` (version `0.18.2`) : For displaying a progress bar.
- `encoding_rs` (version `0.8.35`) : For file encoding management.
- `encoding_rs_io` (version `0.1.7`) : For reading files with different encodings.
- `ignore` (version `0.4.25`) : For ignoring files and directories.
- `tempfile` (version `3.23.0`) : For creating temporary files and directories in tests.

## Installation

### Prerequisites

Make sure you have Rust and Cargo installed on your system. You can install them by following the instructions on the official Rust website: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### Compiling for Linux (from Linux/macOS)
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

The macOS executable will be located in `target/<your_mac_target>/release/dedupler` (e.g., `target/x86_64-apple-darwin/release/dedupler`).


## Usage

```bash
dedupler [OPTIONS] [FILE]
```

### Arguments

-   `[FILE]` : The input file to process. Cannot be used with `-d` / `--directory`.

### Options

-   `-d, --directory <DIRECTORY>` : Process all files in the specified directory. Cannot be used with `[FILE]`.
-   `-o, --output <OUTPUT>` : Path to the output file. If not provided, results are printed to the terminal. When processing a directory, this specifies an output directory to mirror the input structure.
-   `--stat` : Show detailed execution statistics.
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

## Ignoring Files

The tool automatically respects `.gitignore` and `.ignore` files in the directory being processed. You can add more ignore patterns using the `--ignore` flag.

The patterns are glob patterns. For example:
-   `--ignore '*.tmp'`: Ignores all files with the `.tmp` extension.
-   `--ignore 'logs/'`: Ignores the `logs` directory.
-   `--ignore '**/temp*'`: Ignores all files and directories starting with `temp` in any subdirectory.

## Tests

This project includes unit tests; to run them, use the following command at the project root:

```sh

cargo test
```

This command compiles the program in test mode and executes all test functions.