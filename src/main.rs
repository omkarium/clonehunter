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
//! clonehunter your-folder-path -t 12 -c -v
//! 
//!  ```
//! `-c` stands for checksum. If you pass this option, clonehunter will find the file clones (aka duplicate files or identical files) 
//! based on a partial checksum by reading bytes from the beginning and the ending of the file content.
//! If you do not pass -c option, then clonehunter will scan for clones based on a combination of file name, modified time and file size hash combined.
//! 
//! `-v` stands for verbose. It prints the hashes of each and every file for you to compare and manually figure out clones.
//! 
//! `-t` stands for threads. Choose the number of threads to allocate the program to hunt. In the above example I am using 12 threads.
//! 
//! ## Some considerations
//! 
//! The program scans and outputs identical files based on best effort basis. This means that not all files it reports on can be deemed as 'Absolutely identical'. So, the key term here is "Possibly identical". This tool can be used when you want to do a quick analysis to see which files are POSSIBLY identical. This tool must not be used in critical places and business solutions, and must not be considered as the source of truth to delete any of those found identical files.
//! 
//! Also, using this tool will not destroy any files on your machine. There are no delete or write operations performed in the code. If you found any such strangeness, please raise an Issue. At most, the tool reports incorrect identical files or skips some of the files which are not accessible due to file permission.
mod operations;
use common::{confirmation, recurse_dirs, DIR_LIST, FILE_LIST, VERBOSE};
use crate::operations::run;
use clap::Parser;
use std::env;
use std::{path::PathBuf, time::Instant};
use colored::Colorize;

#[derive(Parser)]
#[command(author="@github.com/omkarium", version, about, long_about = None)]
struct Args {
    /// Enter the Source Dir here (This is the directory under which you are looking for the identical files)
    source_dir: String,
    /// Print verbose output
    #[clap(short, long, default_value_t = false)]
    checksum: bool,
    /// Threads to speed up the execution
    #[clap(short, long, default_value_t = 8)]
    threads: u8,
    /// Print verbose output
    #[clap(short, long, default_value_t = false)]
    verbose: bool    
}


fn main() {
    let args = Args::parse();

    *VERBOSE.write().unwrap() = args.verbose;

    let path = PathBuf::from(args.source_dir.clone());

    DIR_LIST.lock().unwrap().push(path.clone());

    recurse_dirs(&path);

    println!("\n################### CloneHunter ({}) #########################\n", "by Omkarium".green());
    println!("\n{}\n", 
    "[Please read the documentation at https://github.com/omkarium before you use this program]".red());
    println!("\n**** Operational Info ****\n");
    println!("Operating system                              : {}", env::consts::OS);
    println!("The source directory you provided             : {}", args.source_dir);
    println!("Total directories found in the path provided  : {}", DIR_LIST.lock().unwrap().to_vec().capacity());
    println!("Total files found in the directories          : {}",FILE_LIST.lock().unwrap().to_vec().capacity());
    println!("Total threads about to be used                : {}", args.threads);
    println!("Perform a Checksum?                           : {}", args.checksum);
    println!("\nWe will now hunt for duplicate files. Make sure to redirect the output to a file now. Are you ready?");
    println!("\n~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");


    if confirmation() == "Y" {
        let a = FILE_LIST.lock().unwrap().to_vec();
        let start_time = Instant::now();
        let dup_count = run(a, args.checksum, args.threads);
        let elapsed = Some(start_time.elapsed());

        println!("\n============Results==============\n");
        
        println!("Time taken to finish Operation: {:?}", elapsed.unwrap());
        println!("Total duplicate records found {}", dup_count);
        println!("\nWe are done. Enjoy hacker!!! 😎");

        println!("\n=================================\n");
    } else {
        println!("\nPhew... You QUIT!\n");
    }
}
