use std::{fs::{self, DirEntry, OpenOptions}, io::ErrorKind, os::windows::fs::MetadataExt, process::exit};

use clap::Parser;
use log::{debug, error};
use readable::byte::Byte;

#[derive(Parser, Debug)]
#[command(about, version)]
struct Args {
    root: String,

    #[arg(long, short, default_value_t = false)]
    raw: bool,
    #[arg(long, short)]
    exclude: Option<Vec<String>>
}

fn walk(entry: DirEntry) -> u64 {
    if entry.file_type().unwrap().is_dir() {
        let mut sum = 0;

        fs::read_dir(entry.path().clone()).unwrap_or_else(|e| {
            error!("Failed to open directory '{}'", entry.path().to_str().unwrap());
            error!("Error: {}", e);
            exit(1);
        }).for_each(|e| sum += walk(e.unwrap()));

        return sum
    }

    if entry.file_type().unwrap().is_symlink() {
        return 0;
    }

    let size = entry
        .metadata()
        .unwrap()
            .file_size();

    debug!("{} => {}", entry.path().to_str().unwrap(), Byte::from(size));

    return size;
}

fn main() {
    colog::default_builder()
        .filter_level( if cfg!(debug_assertions) { log::LevelFilter::Debug } else { log::LevelFilter::Warn } )
        .format_level(false)
        .init();
    
    let args = Args::parse();
    let root = fs::read_dir(args.root.clone());
    
    let mut sum = 0;
    match root {
        Ok(dir) => {
            dir.for_each(|elem| sum += walk(elem.unwrap()));
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