use std::{fs::{self, DirEntry, OpenOptions}, io::ErrorKind, os::windows::fs::MetadataExt, process::exit};

use clap::Parser;
use log::{info, error};
use readable::byte::Byte;
use regex_filtered::Regexes;

#[derive(Parser, Debug)]
#[command(about, version)]
struct Args {
    root: String,
    
    #[arg(long, short = 'x')]
    /// Exclude files that match any of these patterns
    exclude: Vec<String>,

    #[arg(long, short, default_value_t = false)]
    /// Print raw size in bytes
    raw: bool,

    #[arg(long, short, default_value_t = false)]
    /// Print files as they are scanned
    verbose: bool,
}

fn walk(entry: DirEntry, patterns: &Regexes) -> u64 {
    if patterns.regexes().len() != 0 {
        if patterns.is_match(entry.path().to_str().unwrap()) {
            return 0;
        }
    }

    if entry.file_type().unwrap().is_dir() {
        let mut sum = 0;

        fs::read_dir(entry.path().clone()).unwrap_or_else(|e| {
            error!("Failed to open directory '{}'", entry.path().to_str().unwrap());
            error!("Error: {}", e);
            exit(1);
        }).for_each(|e| sum += walk(e.unwrap(), patterns));

        return sum
    }

    if entry.file_type().unwrap().is_symlink() {
        return 0;
    }

    let size = entry
        .metadata()
        .unwrap()
            .file_size();

    info!("{} => {}", entry.path().to_str().unwrap(), Byte::from(size));

    return size;
}

fn main() {
    let args = Args::parse();
    
    colog::default_builder()
        .default_format()
        .format_timestamp(None)
        .format_target(false)
        .filter_level( if cfg!(debug_assertions) || args.verbose  { log::LevelFilter::Info } else { log::LevelFilter::Warn } )
        .init();
    
    let root = fs::read_dir(args.root.clone());
    let mut matcher = regex_filtered::Builder::new();

    if args.exclude.len() != 0 {

        for pattern in args.exclude.clone() {
            matcher = matcher.push(&pattern).unwrap_or_else(|err| {
                error!("Invalid regex pattern '{}'", pattern);
                error!("{}", err);

                exit(1)
            })
        }
    }

    let regex = matcher.build().unwrap();
    
    let mut sum = 0;
    match root {
        Ok(dir) => {
            dir.for_each(
                |elem|
                    sum += walk(elem.unwrap(), &regex));
            println!("{}", maybe_raw(args.raw, sum));
        },
        Err(err) => {
            if err.kind() == ErrorKind::NotADirectory {
                println!(
                    "{}",
                    maybe_raw(args.raw, 
                        OpenOptions::new()
                            .read(true)
                            .open(args.root)
                            .unwrap()
                                .metadata()
                                .unwrap()
                                    .file_size()));
            } else {
                error!("Failed to open directory '{}'", args.root);
                error!("Error: {}", err);
                exit(1);
            }

        },
    }
}

fn maybe_raw(raw: bool, size: u64) -> String {
    if raw {
        format!("{}", size)
    } else {
        Byte::from(size).to_string()
    }
}