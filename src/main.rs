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

mod hunt;
mod delete;

use crate::hunt::hunt;
use clap::Parser;
use colored::Colorize;
use clonehunter::common::{config::{Args, Command, OrderBy, OutputStyle, SortBy}, core::{
    confirmation, recurse_dirs, walk_dirs, PrinterConfig, PrinterJSONObject, SortOrder, DIR_LIST, FILES_SIZE_BYTES, FILE_LIST, VERBOSE
}};
use delete::delete;
use human_bytes::human_bytes;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    env, fs::File, io::BufReader, path::PathBuf, time::{Duration, Instant}
};

fn main() -> std::io::Result<()> {
    println!(
        "\n################### CloneHunter ({}) #########################\n",
        "by Omkarium".green().bold()
    );
    println!("\n{}\n", 
    "[Please read the documentation at https://github.com/omkarium before you use this program]".bright_magenta());

    let command = Args::parse().command;
    let threads = Args::parse().threads;
    let verbose = Args::parse().verbose;
    

    match command {
        Command::Hunt(options) => {
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
            
            match options.sort_by {
                SortBy::FileType => {
                    if options.order_by.is_some() {
                        eprintln!("Error: --order-by cannot be used with --sort-by file-type\n");
                        std::process::exit(1);
                    }
                    println!("I will sort the final output by file type\n");
                }
                SortBy::FileSize => {
                    if let Some(order_by) = options.order_by {
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
                    if let Some(order_by) = options.order_by {
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

            pb.set_message("Please be patient while I am scanning for files. The time it takes has a direct relation to the size of the source directory specified");

            unsafe {
                VERBOSE = verbose;
            }

            let path = PathBuf::from(options.source_dir.clone());

            if options.no_max_depth {
                DIR_LIST.lock().unwrap().push(path.clone());
                recurse_dirs(&path, options.extension.as_deref());
            } else {
                walk_dirs(&path, options.max_depth, threads, options.extension.as_deref());
            }

            pb.finish_with_message("Scan completed");

            let total_files_size = FILES_SIZE_BYTES.lock().unwrap();
            println!("\n\n**** Operational Info ****\n");
            println!("Operating system                              : {}", env::consts::OS);
            println!("The source directory you provided             : {}", options.source_dir);
            println!("Maximum depth of directories to look for      : {}", if options.no_max_depth {"Ignored".to_owned()} else {options.max_depth.to_string()});
            println!("Total directories found in the path provided  : {}", DIR_LIST.lock().unwrap().to_vec().capacity());
            println!("Total files found in the directories          : {}", FILE_LIST.lock().unwrap().to_vec().capacity());
            println!("Total size of source directory                : {}", human_bytes(total_files_size.unwrap_or_default() as f64));
            println!("Total threads about to be used                : {}", threads);
            println!("Perform a Checksum?                           : {}", options.checksum);
            println!("Verbose printing?                             : {}", verbose);
            println!("Target file type / Extension                  : {}", options.extension.unwrap_or("NA".to_string()));
            println!("Sort by                                       : {:?}", options.sort_by);
            println!("Order by                                      : {}", if options.order_by.is_some() {
                options.order_by.unwrap().to_string()
            } else {
                "NA".to_owned()
            });
            println!("Output file                                   : {}", options.output_file.clone().unwrap_or("NA".to_owned()));
            println!("Output style                                  : {}", if options.output_style.is_some() {
                options.output_style.unwrap().to_string()
            } else {
                "NA".to_owned()
            });

            println!("\nWe will now hunt for duplicate files. Make sure to redirect the output to a file now. Are you ready?");
            println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");

            if confirmation() == "Y" {
                let vec_pathbuf = FILE_LIST.lock().unwrap().to_vec();
                let start_time = Instant::now();
                let sort_order = SortOrder(options.sort_by, options.order_by);

                let print_conf = if options.output_style.is_some() && options.output_file.clone().is_some() {
                    match options.output_style.clone().unwrap() {
                        OutputStyle::Default | OutputStyle::JSON  => {
                            let file = File::create(options.output_file.unwrap()).expect("Error: Failed to create the output file you passed via --out-path option\n");
                            PrinterConfig {
                                file: Some(file),
                                sort_order,
                                output_style: options.output_style.unwrap()
                            }
                        }
                    }
                } else {
                    PrinterConfig {
                        file: None,
                        sort_order,
                        output_style: OutputStyle::Default,
                    }
                };

                let dup_data = hunt(vec_pathbuf, options.checksum, threads, print_conf);
                let elapsed = Some(start_time.elapsed());

                println!("\n============{}==============\n", "Result".bright_blue());

                println!("Time taken to finish Operation: {:?}", elapsed.unwrap());
                println!("Total duplicate records found: {}", dup_data.0.to_string().bright_purple().bold());
                println!(
                    "Total duplicate records file size on the disk: {}",
                    human_bytes(dup_data.1 as f64).bright_purple().bold()
                );
                println!("\nWe are done. Have a nice day 😎");

                println!("\n=================================\n");
            } else {
                println!("\nPhew... You QUIT!\n");
            }
        }
        Command::Delete(options) => {
            let input_file = options.input_file;
            if let Ok(f) = File::open(input_file) {
                let reader = BufReader::new(f);
                if let Ok(input_json) = serde_json::from_reader::<_,Vec<PrinterJSONObject>>(reader) {
                    delete(input_json, options.dry_run);
                } else {
                    eprintln!("{}", "Error: failed to read the input file as JSON. Make sure it is a valid JSON.\n".bright_red());
                }
            } else {
                eprintln!("{}", "Error: the input file you have provided does not exist\n".bright_red());
            }
        },
    };
    Ok(())
}
