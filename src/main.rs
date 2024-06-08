//! # CloneHunter
//!
// Copyright (c) 2024 Venkatesh Omkaram
#![doc = include_str!("../README.md")]
#![deny(clippy::all)]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
