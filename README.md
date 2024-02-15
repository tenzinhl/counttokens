**Token Counter**

This program counts the number of tokens within files of specified extensions in a given directory. It utilizes the `tiktoken_rs` library for tokenization and `rayon` for efficient parallel processing.

**How it Works**

1. **Configuration:** The program optionally takes command-line arguments to specify file extensions to process. If no extensions are provided, all files will be considered.
2. **File Discovery:** The program recursively searches the provided directory (defaulting to the current directory) and builds a list of files matching the target extensions. File discovery is made faster through parallelization using the `rayon` library.
3. **Tokenization and Counting:** The program processes each file in parallel:
     * The file's contents are read.
     * The `tiktoken_rs` library's `cl100k_base` model is used to tokenize the content.
     * The total number of tokens is counted.
4. **Results:** Token counts are aggregated by file extension. The program then displays a sorted list of extensions and their corresponding token counts.

**Dependencies**

* Rust ([https://www.rust-lang.org/](https://www.rust-lang.org/))
* `tiktoken_rs` crate
* `rayon` crate

**Build Instructions**

1. **Install Rust:** If you don't have Rust installed, follow the instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
2. **Clone or download this project:** Get the code from the source repository.
4. **Build:** Navigate to the project directory and run:
   cargo build

**Usage**
```bash
cargo run [extension1] [extension2] ...
```
* [extension1], [extension2], ... : Optional list of file extensions to consider (e.g., "txt", "py", "rs"). If omitted, all file extensions will be processed.

**Example**

Count the tokens in all python and rust files in this repository.

```bash
cargo run py rs
```
