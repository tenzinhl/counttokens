use tiktoken_rs::{cl100k_base, CoreBPE};
use std::{env, panic};
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use std::io::Read;
use rayon::prelude::*;
use simple_tqdm::{ParTqdm};

fn main() {
    let bpe = cl100k_base().unwrap();
    let args: Vec<String> = env::args().collect();
    let file_extensions: Vec<String> = if args.len() > 1 { args[1..].to_vec() } else { Vec::new() };

    let dir = Path::new(".");
    if dir.is_dir() {
        // Find all files
        let all_files: Vec<_> = find_files_parallel(dir, &file_extensions);

        // Process files in parallel
        let token_counts: HashMap<String, i32> = all_files.into_par_iter()
            .tqdm()
            .map(|path| process_file(&path, &bpe))
            .reduce(HashMap::new, |mut acc, item| {
                for (ext, count) in item {
                    *acc.entry(ext).or_insert(0) += count;
                }
                acc
            });

        // Sort HashMap by token count and convert it into a vector of tuples
        let mut token_counts_vec: Vec<(&String, &i32)> = token_counts.iter().collect();
        token_counts_vec.sort_by(|a, b| b.1.cmp(a.1));
        for &(extension, &count) in token_counts_vec.iter() {
            if count > 0 {
                println!("{}: {}", extension, count);
            }
        }
        // Print the token number of tokens
        println!("Total: {}", token_counts_vec.iter().map(|(_, count)| *count).sum::<i32>());
    }
}

fn find_files_parallel(dir: &Path, file_extensions: &Vec<String>) -> Vec<std::path::PathBuf> {
    match fs::read_dir(dir) {
        Ok(dir) => dir.par_bridge() // Utilize rayon's par_bridge
            .flat_map(|entry| {
                let entry = entry.expect("Unable to read entry");
                let path = entry.path();
                if path.is_dir() {
                    find_files_parallel(&path, file_extensions)
                } else if file_extensions.is_empty()
                    || file_extensions.contains(
                    &path.extension().and_then(std::ffi::OsStr::to_str).unwrap_or("").to_string()
                )
                {
                    vec![path]
                } else {
                    vec![]
                }
            })
            .collect(),
        Err(_) => vec![]
    }
}

fn process_file(path: &Path, tokenizer: &CoreBPE) -> HashMap<String, i32> {
    let mut token_counts = HashMap::new();
    let extension = path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("");
    let count = count_tokens(path, tokenizer);
    token_counts.insert(extension.to_string(), count);
    token_counts
}

fn count_tokens(path: &Path, tokenizer: &CoreBPE) -> i32 {
    let file = fs::File::open(&path);
    let mut file = match file {
        Ok(f) => f,
        Err(_) => {
            return 0;
        }
    };
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return 0;
    }
    let result = panic::catch_unwind(|| {
        let tokens = tokenizer.encode_with_special_tokens(&contents);
        tokens.len() as i32
    });
    drop(contents);
    result.unwrap_or_else(|_| {
        println!("Error while processing file: {}", path.display());
        0
    })
}