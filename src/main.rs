use std::{env, fs::{self, DirEntry}, io::ErrorKind, path::Path, process::exit};

use clap::Parser;
use filesize::PathExt;
use indicatif::{DecimalBytes, HumanBytes, ProgressBar, ProgressStyle};
use log::{error, info};
use regex_filtered::Regexes;

#[derive(Parser, Debug, Clone)]
#[command(about, version)]
struct Args {
    root: String,

    /// Exclude files that match any of these patterns
    #[arg(long, short = 'x')]
    exclude: Vec<String>,

    /// Only include files that match any of these patterns
    #[arg(long, short)]
    include: Vec<String>,

    /// Print raw size in bytes
    #[arg(long, short, default_value_t = false)]
    raw: bool,

    /// Print files as they are scanned
    #[arg(long, short, default_value_t = false)]
    verbose: bool,

    /// Show progress bar
    #[arg(long, short, default_value_t = false)]
    progress: bool,
    
    /// Use humanized size units (GB -> GiB, MB -> MiB, etc.)
    #[arg(long, default_value_t = false)]
    human: bool,
    
    /// Instead of panicking at errors, skip them
    #[arg(long, default_value_t = false)]
    ignore_errors: bool,
}

fn walk(args: &Args, entry: DirEntry, inclusions: &Regexes, exclusions: &Regexes, progress: &mut Option<ProgressBar>) -> u64 {
    if exclusions.regexes().len() != 0 {
        if exclusions.is_match(entry.path().to_str().unwrap()) {
            for r in inclusions.matching(entry.path().to_str().unwrap()) {
                panic!("{} matches {} {}", entry.path().to_str().unwrap(), r.0, r.1);
            }
            return 0;
        }
    }

    if entry.file_type().unwrap().is_dir() {
        let dir = fs::read_dir(entry.path().clone());

        if let Err(e) = &dir {
            if !args.ignore_errors {
                if let Some(bar) = progress {
                    bar.abandon();
                }

                error!("Failed to open directory '{}': {}", entry.path().to_str().unwrap(), e);
                exit(1);
            } else {
                return 0;
            }
        }

        if let Some(bar) = progress {
            bar.inc_length(fs::read_dir(entry.path().clone()).unwrap().count() as u64);
            bar.set_message(format!("{}", entry.file_name().into_string().unwrap()));
        }

        let mut sum = 0;
        dir
            .unwrap()
            .for_each(|e|
                sum += walk(args, e.unwrap(), inclusions, exclusions, progress));

        return sum
    } else if entry.file_type().unwrap().is_symlink() {
        return 0;
    }

    if inclusions.regexes().len() != 0 {
        if !inclusions.is_match(entry.path().to_str().unwrap()) {
            return 0;
        }
    }

    let size = entry.path().size_on_disk().unwrap_or_else(|err| {
        if !args.ignore_errors {
            if let Some(bar) = progress {
                bar.abandon();
            }

            error!(
                "Failed to get size of file {} ({})",
                    entry
                        .path()
                        .to_str()
                        .unwrap_or("??"),
                    err);

            exit(1);
        }

        0
    });
    
    if args.verbose {
        info!("{:<20} => {}", entry.path().to_str().unwrap(), DecimalBytes(size).to_string());
    }

    if let Some(bar) = progress {
        bar.inc(1);
    }

    return size;
}

fn main() {
    let mut args = Args::parse();
    
    // Makes path display for verbose mode pretty on windows (../..\otherpath => ..\..\otherpath)
    if env::consts::OS == "windows" {
        args.root = args.root.replace("/", "\\");
    }

    colog::default_builder()
        .default_format()
        .format_timestamp(None)
        .format_target(false)
        .filter_level( if cfg!(debug_assertions) || args.verbose  { log::LevelFilter::Debug } else { log::LevelFilter::Info } )
        .init();

    let root = fs::read_dir(args.root.clone());

    let mut progress = 
        if args.progress {
            let bar = ProgressBar::new(1);
            bar.set_style(
                ProgressStyle::with_template("[{elapsed_precise}] {bar:40.white} Scanned ({pos:<7}/{len:7}) Scanning {msg}")
                .unwrap());
            bar.set_message(format!("Scanning {}...", args.root));

            Some(bar)
        } else {
            None
        };

    let exclusions = build_regexes(args.exclude.clone());
    let inclusions = build_regexes(args.include.clone());

    let mut sum = 0;

    match root {
        Ok(dir) => {
            dir.for_each(
                |elem|
                    sum += walk(
                        &args,
                        elem.unwrap(),
                        &inclusions,
                        &exclusions,
                        &mut progress));
            
            if let Some(bar) = progress {
                bar.finish();
            }

            println!("{}", maybe_raw(args.raw, args.human, sum));
        },
        Err(err) => {
            if err.kind() == ErrorKind::NotADirectory {
                println!(
                    "{}",
                    maybe_raw(
                        args.raw, args.human,
                        Path::new(&args.root)
                            .size_on_disk()
                            .unwrap_or_else(|err| {
                                if !args.ignore_errors {
                                    error!("Failed to get size of file {} ({})", args.root, err);
                                    exit(1);
                                }
                                0
                            })));
            } else {
                error!("Failed to open directory '{}'", args.root);
                error!("Error: {}", err);
                exit(1);
            }

        },
    }
}

fn maybe_raw(raw: bool, human: bool, size: u64) -> String {
    if raw {
        format!("{}", size)
    } else {
        if human {
            HumanBytes(size).to_string()
        } else {
            DecimalBytes(size).to_string()
        }
    }
}

fn build_regexes(patterns: Vec<String>) -> Regexes {
    let mut matcher = regex_filtered::Builder::new();
    if patterns.len() != 0 {
        for pattern in patterns.clone() {
            matcher = matcher.push(&pattern).unwrap_or_else(|err| {
                error!("Invalid regex pattern '{}'", pattern);
                error!("{}", err);

                exit(1)
            })
        }
    }

    matcher.build().unwrap()
}
