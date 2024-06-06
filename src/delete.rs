use std::fs::remove_file;

use clonehunter::common::core::{confirmation, PrinterJSONObject};
use colored::Colorize;
use human_bytes::human_bytes;

pub fn delete(input_json: Vec<PrinterJSONObject>, dry_run: bool) {
    let mut failed_to_delete = Vec::new();
    let mut total_files_size = 0;
    let tota_groups = input_json.len();
    for i in &input_json {
        total_files_size += i.duplicate_group_bytes_each;
    }
    println!("Is this a dry run? : {}\n", dry_run);
    println!(
        "Found {} group(s) with {} total files size on the disk.\n",
        tota_groups,
        human_bytes(total_files_size as f64)
    );
    if tota_groups != 0 {
        println!("{}", "Shall I proceed to delete the duplicates?".bright_blue());
        if confirmation() == "Y" {
            for mut i in input_json {
                println!(
                    "\nTrying deleting {} file(s) in group {}",
                    i.duplicate_group_count - 1,
                    i.duplicate_group_no
                );
                if let Some(retained_file) = i.duplicate_list.pop() {
                    for (l, j) in i.duplicate_list.iter().enumerate() {
                        if !dry_run {
                            if let Err(result) = remove_file(j.as_str()) {
                                dbg!(j);
                                failed_to_delete
                                    .push(format!("Failed to delete the file {} due to {}", j, result));
                            } else {
                                println!("      Deleted the file ({}) :: {}", l, j.bright_blue());
                            }
                        } else {
                            println!("      Deleted the file ({}) :: {}", l, j.bright_blue());
                        }
                    }
                    println!(
                        "\n      Retained the file :: {}\n",
                        retained_file.bright_green()
                    );
                }
            }

            if !failed_to_delete.is_empty() && !dry_run {
                println!("## {} ##\n", "Error: Looks like there were some failures while deleting certain duplicates. Here is the list".bright_red().bold());
                for i in failed_to_delete {
                    eprintln!("{}", i.bright_magenta());
                }
            } else if dry_run {
                println!("\nNothing changed. This was a dry run.\n");
            } else {
                println!("\nLooks like we are done. Now don't start crying\n");
            }
        } else {
            println!("Phew... You QUIT!\n");
        }
    } else {
        println!("\nFound no duplicates. You lucky son of a gun.\n");
    }
}
