use std::{fs::{remove_file, File}, io::BufReader};

use clonehunter::common::core::{confirmation, PrinterJSONObject};
use human_bytes::human_bytes;

pub fn delete(file: &File) {
    let reader = BufReader::new(file);
    if let Ok(input_json) = serde_json::from_reader::<_,Vec<PrinterJSONObject>>(reader) {
        let input_json = input_json;
        let mut total_files_size = 0;
        let tota_groups = input_json.len();
        for i in &input_json {
            total_files_size += i.duplicate_group_bytes_each;

        }
        println!(
            "Found {} group(s) with {} total files size on the disk. Shall I proceed to delete the duplicates?\n",
            tota_groups,
            human_bytes(total_files_size as f64)
        );
        if confirmation() == "Y" {
            for mut i in input_json {
                println!("\nDeleting {} file(s) in group {}", i.duplicate_group_count - 1, i.duplicate_group_no);
                if let Some(retained_file) = i.duplicate_list.pop(){
                    for (l, j) in i.duplicate_list.iter().enumerate() {
                        println!("      Deleted the file ({}) :: {}", l, j);
                        if let Err(result) = remove_file(j) {
                            //eprintln!("Failed to delete the file {} due to {}", j, result);
                        }
                    }
                    println!("\n      Retained the file :: {}\n", retained_file);
                }
                

            }

        } else {
            
        }

        
    } else {
        eprintln!("Error: failed to read the input file as JSON. Make sure it is a valid JSON.\n");
    }
}
