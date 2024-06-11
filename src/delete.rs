use std::fs::remove_file;

use clonehunter::common::core::{confirmation, log, LogLevel, PrinterJSONObject};
use colored::Colorize;
use human_bytes::human_bytes;

pub fn delete(input_json: Vec<PrinterJSONObject>, dry_run: bool) {
    let mut failed_to_delete = Vec::new();
    let mut total_files_size = 0;
    let tota_groups = input_json.len();
    for i in &input_json {
        total_files_size += i.duplicate_group_bytes_each;
    }
    log(LogLevel::INFO, format!("Is this a dry run? : {}", dry_run.to_string().blink()).as_str());
    log(
        LogLevel::INFO,
        format!("Found {} group(s) with {} total files size on the disk",
        tota_groups,
        human_bytes(total_files_size as f64).blink()
    ).as_str());

    if tota_groups != 0 {
        println!("{}", "\nShall I proceed to delete the duplicates?".bright_blue());
        if confirmation() == "Y" {
            for mut i in input_json {
                println!(
                    "Trying deleting {} file(s) in group {} of size {}",
                    i.duplicate_group_count - 1,
                    i.duplicate_group_no,
                    human_bytes(i.duplicate_group_bytes_each as f64)
                );
                if let Some(retained_file) = i.duplicate_list.pop() {
                    let mut last_file_no = 0;
                    for (l, j) in i.duplicate_list.iter().enumerate() {
                        last_file_no = l;
                        if !dry_run {
                            if let Err(result) = remove_file(j.as_str()) {
                                failed_to_delete.push(format!("Failed to delete the file {} due to {}", j, result));
                            } else {
                                println!("      Deleted the file ({}) :: {}", l, j.bright_blue());
                            }
                        } else {
                            println!("      Deleted the file ({}) :: {}", l, j.bright_blue());
                        }
                    }
                    if i.duplicate_group_bytes_each == 0 {
                        if let Err(result) = remove_file(retained_file.as_str()) {
                            failed_to_delete.push(format!("Failed to delete the file {} due to {}", retained_file, result));
                        } else {
                            println!("      Deleted the file ({}) :: {}", last_file_no+1, retained_file.bright_blue());
                        }
                    } else {
                        println!(
                            "\n      Retained the file :: {}\n",
                            retained_file.bright_green()
                        );
                    }
                }
            }

            if !failed_to_delete.is_empty() && !dry_run {
                println!("## {} ##\n", "Error: Looks like there were some failures while deleting certain duplicates. Here is the list".bright_red().bold());
                for i in failed_to_delete {
                    eprintln!("{}", i.bright_magenta());
                }
            } else if dry_run {
                log(LogLevel::INFO, "Nothing changed. This was a dry run.\n");
            } else {
                println!("\nLooks like we are done deleting. Now please don't start crying.\n");
            }
        } else {
            println!("Phew... You QUIT!\n");
        }
    } else {
        println!("\nFound no duplicates. You lucky son of a gun.\n");
    }
}
