use std::{env, fs::{self, DirEntry, OpenOptions}, io::ErrorKind, os::windows::fs::MetadataExt, process::exit};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, error};
use readable::byte::{Byte};
use regex_filtered::Regexes;

#[derive(Parser, Debug)]
#[command(about, version)]
struct Args {
    root: String,

    /// Exclude files that match any of these patterns
    #[arg(long, short = 'x')]
    exclude: Vec<String>,

    /// Print raw size in bytes
    #[arg(long, short, default_value_t = false)]
    raw: bool,

    /// Print files as they are scanned
    #[arg(long, short, default_value_t = false)]
    verbose: bool,

    /// Show progress bar
    #[arg(long, short, default_value_t = false)]
    progress: bool,
}

fn walk(entry: DirEntry, patterns: &Regexes, progress: &mut Option<ProgressBar>) -> u64 {
    if patterns.regexes().len() != 0 {
        if patterns.is_match(entry.path().to_str().unwrap()) {
            return 0;
        }
    }

    
    if entry.file_type().unwrap().is_dir() {
        let mut sum = 0;

        let dir = fs::read_dir(entry.path().clone()).unwrap_or_else(|e| {
            error!("Failed to open directory '{}'", entry.path().to_str().unwrap());
            error!("Error: {}", e);
            exit(1);
        });

        if let Some(bar) = progress {
            bar.inc_length(fs::read_dir(entry.path().clone()).unwrap().count() as u64);
            bar.set_message(format!("{}", entry.file_name().into_string().unwrap()));
        }

        dir.for_each(|e| sum += walk(e.unwrap(), patterns, progress));

        return sum
    } else if entry.file_type().unwrap().is_symlink() {
        return 0;
    }

    let size = entry
        .metadata()
        .unwrap()
            .file_size();
    info!("{} => {}", entry.path().to_str().unwrap(), Byte::from(size));

    if let Some(bar) = progress {
        bar.inc(1);
    }

    return size;
}

fn main() {
    let mut args = Args::parse();
    
    if env::consts::OS == "windows" {
        args.root = args.root.replace("/", "\\");
    }

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

    let mut progress = 
        if args.progress {
            let pb = ProgressBar::new(1);
            pb.set_style(
                ProgressStyle::with_template("[{elapsed_precise}] {bar:40.white} Scanned ({pos:<7}/{len:7}) Scanning {msg}")
                .unwrap());
            pb.set_message(format!("Scanning {}...", args.root));

            Some(pb)
        } else {
            None
        };

    let regex = matcher.build().unwrap();
    let mut sum = 0;

    match root {
        Ok(dir) => {
            dir.for_each(
                |elem|
                    sum += walk(elem.unwrap(), &regex, &mut progress));
            
            if let Some(bar) = progress {
                bar.finish();
            }

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