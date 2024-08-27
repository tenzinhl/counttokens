use tiktoken_rs::{cl100k_base, CoreBPE};
use std::{env, panic};
use std::path::Path;
use std::fs;
use std::collections::HashMap;
use std::io::Read;
use rayon::prelude::*;
use simple_tqdm::ParTqdm;
use num_format::{SystemLocale, ToFormattedString};

struct FileStats {
    token_count: i32,
    line_count: i32,
    file_count: i32,
}

struct Args {
    directory: String,
    file_extensions: Vec<String>,
}

fn parse_args() -> Args {
    let args: Vec<String> = env::args().collect();
    let mut directory = String::from(".");
    let mut file_extensions = Vec::new();
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-d" => {
                if i + 1 < args.len() {
                    directory = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: -d option requires a directory path");
                    std::process::exit(1);
                }
            }
            _ => {
                file_extensions.push(args[i].clone());
                i += 1;
            }
        }
    }

    Args { directory, file_extensions }
}

fn main() {
    let bpe = cl100k_base().unwrap();
    let args = parse_args();

    let locale = SystemLocale::default().unwrap();

    let dir = Path::new(&args.directory);
    if dir.is_dir() {
        let all_files: Vec<_> = find_files_parallel(dir, &args.file_extensions);

        let file_stats: HashMap<String, FileStats> = all_files.into_par_iter()
            .tqdm()
            .map(|path| process_file(&path, &bpe))
            .reduce(HashMap::new, |mut acc, item| {
                for (ext, stats) in item {
                    let entry = acc.entry(ext).or_insert(FileStats { token_count: 0, line_count: 0, file_count: 0 });
                    entry.token_count += stats.token_count;
                    entry.line_count += stats.line_count;
                    entry.file_count += stats.file_count;
                }
                acc
            });

        let mut file_stats_vec: Vec<(&String, &FileStats)> = file_stats.iter().collect();
        file_stats_vec.sort_by(|a, b| b.1.token_count.cmp(&a.1.token_count));

        for (extension, stats) in file_stats_vec.iter() {
            if stats.token_count > 0 {
                let formatted_token_count = stats.token_count.to_formatted_string(&locale);
                let formatted_line_count = stats.line_count.to_formatted_string(&locale);
                let formatted_file_count = stats.file_count.to_formatted_string(&locale);
                println!("{}: {} tokens, {} lines, {} files", extension, formatted_token_count, formatted_line_count, formatted_file_count);
            }
        }

        // Don't print the totals if there is only 1 file extension
        if file_stats_vec.len() > 1 {
            let total_tokens: i32 = file_stats_vec.iter().map(|(_, stats)| stats.token_count).sum();
            let total_lines: i32 = file_stats_vec.iter().map(|(_, stats)| stats.line_count).sum();
            let total_files: i32 = file_stats_vec.iter().map(|(_, stats)| stats.file_count).sum();

            let formatted_total_tokens = total_tokens.to_formatted_string(&locale);
            let formatted_total_lines = total_lines.to_formatted_string(&locale);
            let formatted_total_files = total_files.to_formatted_string(&locale);

            println!("Total: {} tokens, {} lines, {} files", formatted_total_tokens, formatted_total_lines, formatted_total_files);
        }
    } else {
        eprintln!("Error: The specified directory does not exist or is not a directory");
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

fn process_file(path: &Path, tokenizer: &CoreBPE) -> HashMap<String, FileStats> {
    let mut file_stats = HashMap::new();
    let extension = path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("");

    let stats = count_tokens_and_lines(path, tokenizer);
    file_stats.insert(extension.to_string(), stats);
    file_stats
}

fn count_tokens_and_lines(path: &Path, tokenizer: &CoreBPE) -> FileStats {
    let file = fs::File::open(&path);
    let mut file = match file {
        Ok(f) => f,
        Err(_) => return FileStats { token_count: 0, line_count: 0, file_count: 1 },
    };

    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_err() {
        return FileStats { token_count: 0, line_count: 0, file_count: 1 };
    }

    let result = panic::catch_unwind(|| {
        let tokens = tokenizer.encode_with_special_tokens(&contents);
        let line_count = contents.lines().count() as i32;
        FileStats {
            token_count: tokens.len() as i32,
            line_count,
            file_count: 1,
        }
    });

    result.unwrap_or_else(|_| {
        println!("Error while processing file: {}", path.display());
        FileStats { token_count: 0, line_count: 0, file_count: 1 }
    })
}