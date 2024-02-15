use tiktoken_rs::{cl100k_base, CoreBPE};
use std::env;
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use std::io::Read;

fn main() {
    let bpe = cl100k_base().unwrap();
    let args: Vec<String> = env::args().collect();
    let file_extensions: Vec<String> = if args.len() > 1 { args[1..].to_vec() } else { Vec::new() };
    let mut token_counts: HashMap<String, i32> = HashMap::new();

    let dir = Path::new(".");
    if dir.is_dir() {
        iterate_files(&dir, &file_extensions, &mut token_counts, &bpe);
    }

    // Sort HashMap by token count and convert it into a vector of tuples
    let mut token_counts_vec: Vec<(&String, &i32)> = token_counts.iter().collect();
    token_counts_vec.sort_by(|a, b| b.1.cmp(a.1));

    for &(extension, &count) in token_counts_vec.iter() {
        if count > 0 {
            println!("{}: {}", extension, count);
        }
    }
}

fn iterate_files(dir: &Path, file_extensions: &Vec<String>, token_counts: &mut HashMap<String, i32>, tokenizer: &CoreBPE) {
    for entry in fs::read_dir(dir).expect("Unable to read directory") {
        let entry = entry.expect("Unable to read entry");
        let path = entry.path();
        if path.is_dir() {
            iterate_files(&path, file_extensions, token_counts, tokenizer);
        } else {
            let extension = path.extension()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("");
            if file_extensions.is_empty() || file_extensions.contains(&extension.to_string()) {
                let count = count_tokens(&path, tokenizer);
                let entry = token_counts.entry(extension.to_string()).or_insert(0);
                *entry += count;
            }
        }
    }
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
        return 0
    }
    let tokens = tokenizer.encode_with_special_tokens(&contents);
    tokens.len() as i32
}