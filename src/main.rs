//! # CloneHunter
//!
// Copyright (c) 2024 Venkatesh Omkaram

//! CloneHunter is an ultra simple command line utility that identifies groups of identical files and displays them to the console.
//!
//! ## How to Use?
//! If you have the program as a binary executable then run, `clonehunter --help` for usage.
//! If you are running this program via Cargo, then run `cargo run -- --help` from the root folder for usage.
//!
//! To install the program permanently on your system do `cargo install clonehunter`.
//!
//! # Example usage:
//!  ```
//! clonehunter your-folder-path -t 12 -c -v -m 50 -e pdf -s both -o asc
//!
//!  ```
//! `-c` stands for checksum. If you pass this option, clonehunter will find the file clones (aka duplicate files or identical files)
//! based on a partial checksum by reading bytes from the beginning and the ending of the file content.
//! If you do not pass -c option, then clonehunter will scan for clones based on a combination of file name, modified time and file size hash combined.
//!
//! `-m` stands for max depth. The number after -m indicates how many sub levels we need to look for clones. The default value is 10. If you do not wish to specify a max depth, then pass the option `--no-max-depth`
//!
//! `-v` stands for verbose. It prints the hashes of each and every file for you to compare and manually figure out clones.
//!
//! `-t` stands for threads. Choose the number of threads to allocate the program to hunt. In the above example I am using 12 threads.
//!
//! `-e` stands for extension and this feature enables you to target specific file types aka file extensions. In the above example, I am targeting `pdf`. If you do not want to target any specific file types, then do not use the option. You can also pass something like `pdf,txt,mp4`. This will target all the three file types.
//!
//! `-s` stands for sort-by and this feature helps to sort the output to be printed on the screen based on 3 types.
//!  The three types are `file-type`, `file-size`, and `both`. When you pass the value as `both` the output will be sorted based on `file-size` first and `file-type` next.
//!
//! `-o` stands for order-by and this feature helps to order the sorted output which was achieved by the `-s` option. This option only applies for `-s both` or `-s file-size`. It does not matter what the order is when your already sorted using the file-type alone.
//!
//! ## Some considerations
//!
//! The program scans and outputs identical files based on best effort basis. This means that not all files it reports on can be deemed as 'Absolutely identical'. So, the key term here is "Possibly identical". This tool can be used when you want to do a quick analysis to see which files are POSSIBLY identical. This tool must not be used in critical places and business solutions, and must not be considered as the source of truth to delete any of those found identical files.
//!
//! Also, using this tool will not destroy any files on your machine. There are no delete or write operations performed in the code. If you found any such strangeness, please raise an Issue. At most, the tool reports incorrect identical files or skips some of the files which are not accessible due to file permission.
//!
//! ## Regarding files with 0 bytes size
//!
//! If you are running clonehunter on bunch of different files or file types, let's say some mp4, pdf, txt etc but they all have file sizes of 0 bytes, and if you used the -c checksum option, you will observe all of the 0 size files grouped together as duplicates in the final output on the screen.

mod operations;
use crate::operations::run;
use clap::Parser;
use colored::Colorize;
use common::{config::{Args, Command, OrderBy, OutputStyle, SortBy}, core::{
    confirmation, recurse_dirs, walk_dirs, PrinterConfig, SortOrder, DIR_LIST,
    FILES_SIZE_BYTES, FILE_LIST, VERBOSE,
}};
use human_bytes::human_bytes;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env,
    fs::File,
    path::PathBuf,
    time::{Duration, Instant},
};

fn main() {
    println!(
        "\n################### CloneHunter ({}) #########################\n",
        "by Omkarium".green()
    );
    println!("\n{}\n", 
    "[Please read the documentation at https://github.com/omkarium before you use this program]".red());

    let command = Args::parse().command;
    let threads = Args::parse().threads;
    let verbose = Args::parse().verbose;

    match command {
        Command::Hunt(args) => {
            match args.sort_by {
                SortBy::FileType => {
                    if args.order_by.is_some() {
                        eprintln!("Error: --order-by cannot be used with --sort-by file-type\n");
                        std::process::exit(1);
                    }
                    println!("I will sort the final output by file type\n");
                }
                SortBy::FileSize => {
                    if let Some(order_by) = args.order_by {
                        match order_by {
                            OrderBy::Asc => println!("I will sort the final output by file size in the ascending order\n"),
                            OrderBy::Desc => println!("I will sort the final output by file size in the descending order\n"),
                        }
                    } else {
                        eprintln!("Error: --order-by is required with --sort-by file-size\n");
                        std::process::exit(1);
                    }
                }
                SortBy::Both => {
                    if let Some(order_by) = args.order_by {
                        match order_by {
                            OrderBy::Asc => println!("I will sort the final output by file size and file type in the ascending order\n"),
                            OrderBy::Desc => println!("I will sort the final output by file size and file type in the descending order\n"),
                        }
                    } else {
                        eprintln!("Error: --order-by is required with --sort-by file-size and file type\n");
                        std::process::exit(1);
                    }
                }
            }

            let pb = ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(120));
            pb.set_style(
                ProgressStyle::with_template("{spinner:.blue} {msg} {spinner:.blue}")
                    .unwrap()
                    // For more spinners check out the cli-spinners project:
                    // https://github.com/sindresorhus/cli-spinners/blob/master/spinners.json
                    .tick_strings(&[
                        "▹▹▹▹▹",
                        "▸▹▹▹▹",
                        "▹▸▹▹▹",
                        "▹▹▸▹▹",
                        "▹▹▹▸▹",
                        "▹▹▹▹▸",
                        "▪▪▪▪▪",
                    ]),
            );
            pb.set_message("Please be patient while I am scanning for files. The time it takes has a direct relation to the size of the source directory specified");

            unsafe {
                VERBOSE = verbose;
            }

            let path = PathBuf::from(args.source_dir.clone());

            if args.no_max_depth {
                DIR_LIST.lock().unwrap().push(path.clone());
                recurse_dirs(&path, args.extension.as_deref());
            } else {
                walk_dirs(&path, args.max_depth, threads, args.extension.as_deref());
            }

            pb.finish_with_message("Scan completed");

            let total_files_size = FILES_SIZE_BYTES.lock().unwrap();

            println!("\n\n**** Operational Info ****\n");
            println!(
                "Operating system                              : {}",
                env::consts::OS
            );
            println!(
                "The source directory you provided             : {}",
                args.source_dir
            );
            println!(
                "Maximum depth of directories to look for      : {}",
                if args.no_max_depth {
                    "Ignored".to_owned()
                } else {
                    args.max_depth.to_string()
                }
            );
            println!(
                "Total directories found in the path provided  : {}",
                DIR_LIST.lock().unwrap().to_vec().capacity()
            );
            println!(
                "Total files found in the directories          : {}",
                FILE_LIST.lock().unwrap().to_vec().capacity()
            );
            println!(
                "Total size of source directory                : {}",
                human_bytes(total_files_size.unwrap_or_default() as f64)
            );
            println!(
                "Total threads about to be used                : {}",
                threads
            );
            println!(
                "Perform a Checksum?                           : {}",
                args.checksum
            );
            println!(
                "Verbose printing?                             : {}",
                verbose
            );
            println!(
                "Target file type / Extension                  : {}",
                args.extension.unwrap_or("NA".to_string())
            );
            println!("\nWe will now hunt for duplicate files. Make sure to redirect the output to a file now. Are you ready?");
            println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");

            if confirmation() == "Y" {
                let a = FILE_LIST.lock().unwrap().to_vec();
                let start_time = Instant::now();
                let sort_order = SortOrder(args.sort_by, args.order_by);

                let print_conf = if args.output_style.is_some() && args.output_file.is_some() {
                    match args.output_style.unwrap() {
                        OutputStyle::Default | OutputStyle::JSON  => {
                            let file = File::create(args.output_file.unwrap()).expect("Error: Failed to create the output file you passed via --out-path option\n");
                            PrinterConfig {
                                file: Some(file),
                                sort_order,
                            }
                        }
                    }
                } else {
                    PrinterConfig {
                        file: None,
                        sort_order,
                    }
                };

                let dup_data = run(a, args.checksum, threads, print_conf);
                let elapsed = Some(start_time.elapsed());

                println!("\n============Results==============\n");

                println!("Time taken to finish Operation: {:?}", elapsed.unwrap());
                println!("Total duplicate records found: {}", dup_data.0);
                println!(
                    "Total duplicate records file size on the disk: {}",
                    human_bytes(dup_data.1 as f64)
                );
                println!("\nWe are done. Have a nice day 😎");

                println!("\n=================================\n");
            } else {
                println!("\nPhew... You QUIT!\n");
            }
        }
        Command::Delete => todo!(),
    };
}
