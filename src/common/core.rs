// Copyright (c) 2024 Venkatesh Omkaram

use chrono::{DateTime, Local};
use clap::builder::OsStr;
use colored::Colorize;
use hashbrown::HashMap;
use human_bytes::human_bytes;
use indicatif::ProgressBar;
use jwalk::WalkDir;
use lazy_static::lazy_static;
use rayon::iter::{ParallelBridge, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    fs::{self, File},
    hash::Hash,
    io::{stdin, stdout, BufWriter, Write},
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use trait_defs::*;

#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

use crate::common::{
    config::{OrderBy, OutputStyle, SortBy},
    trait_defs,
};

lazy_static! {
    /// A Lazy static reference to hold a List of Directory Paths
    pub static ref DIR_LIST: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
    /// A Lazy static reference to hold a list of File Paths
    pub static ref FILE_LIST: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
    /// A Lazy static reference which hold the file sizes in bytes
    pub static ref FILES_SIZE_BYTES: Mutex<Option<u64>> = Mutex::new(Some(0));
}

pub static mut VERBOSE: bool = false;

/// This function can be used for all sorts of confirmation input from the user
pub fn confirmation() -> String {
    let mut confirmation: String = String::new();

    print!("\nPlease type Y for yes, and N for no : ");

    let _ = stdout().flush();

    stdin()
        .read_line(&mut confirmation)
        .expect("You entered incorrect response");

    if let Some('\n') = confirmation.chars().next_back() {
        confirmation.pop();
    }

    if let Some('\r') = confirmation.chars().next_back() {
        confirmation.pop();
    }

    println!("\nYou typed: {}\n", confirmation.blink());

    confirmation
}

/// A simple macro which prints two items only when verbose printing is specified.
/// VERBOSE is a RwLock
#[macro_export]
macro_rules! logger {
    ($value: literal, $item: expr, $item2: expr) => {
        use clonehunter::common::core::VERBOSE;

        if unsafe { VERBOSE } {
            println!($value, $item, $item2);
        }
    };
}

/// A Struct which can help generate a Hash on its fields
#[derive(Hash)]
pub struct FileMetaData<'a> {
    pub file_name: &'a str,
    pub modified_date: SystemTime,
    pub file_size: u64,
}

/// Struct which holds SortBy and OrderBy User Options
pub struct SortOrder(pub SortBy, pub Option<OrderBy>);

/// Printer configuration
pub struct PrinterConfig {
    pub file: Option<File>,
    pub sort_order: SortOrder,
    pub output_style: OutputStyle,
}

/// JSON printer
#[derive(Serialize, Deserialize, Debug)]
pub struct PrinterJSONObject {
    pub duplicate_group_no: usize,
    pub duplicate_group_count: usize,
    pub duplicate_group_bytes_each: usize,
    pub duplicate_list: Vec<String>,
}

pub struct WalkConfig<'a> {
    pub ext: Option<&'a str>,
    pub max_depth: Option<usize>,
    pub max_file_size: Option<u64>,
    pub min_file_size: Option<u64>,
}

pub enum FileLimitingFactor {
    GreaterThan(usize),
    LessThan(usize),
}

pub fn file_list_generator(entry: &PathBuf, wc: &WalkConfig) {
    if let Some(x) = entry.extension() {
        if let Some(ext) = wc.ext {
            let mut vec_ext = ext.split(",");
            if vec_ext.any(|y| x.eq(y)) {
                FILE_LIST.lock().unwrap().push(
                    entry
                        .to_path_buf()
                        .canonicalize()
                        .unwrap_or_else(|_| entry.to_path_buf()),
                );
                if cfg!(unix) {
                    #[cfg(target_os = "linux")]
                    {
                        match FILES_SIZE_BYTES.lock().unwrap().as_mut() {
                            Some(o) => {
                                *o += match entry.metadata() {
                                    Ok(p) => p.size(),
                                    Err(_) => 0,
                                }
                            }
                            None => {}
                        }
                    }
                } else if cfg!(windows) {
                    #[cfg(target_os = "windows")]
                    {
                        match FILES_SIZE_BYTES.lock().unwrap().as_mut() {
                            Some(o) => {
                                *o += match entry.metadata() {
                                    Ok(p) => p.file_size(),
                                    Err(_) => 0,
                                }
                            }
                            None => {}
                        }
                    }
                }
            }
        } else {
            FILE_LIST.lock().unwrap().push(
                entry
                    .to_path_buf()
                    .canonicalize()
                    .unwrap_or_else(|_| entry.to_path_buf()),
            );
            if cfg!(unix) {
                #[cfg(target_os = "linux")]
                {
                    match FILES_SIZE_BYTES.lock().unwrap().as_mut() {
                        Some(o) => {
                            *o += match entry.metadata() {
                                Ok(p) => p.size(),
                                Err(_) => 0,
                            }
                        }
                        None => {}
                    }
                }
            } else if cfg!(windows) {
                #[cfg(target_os = "windows")]
                {
                    match FILES_SIZE_BYTES.lock().unwrap().as_mut() {
                        Some(o) => {
                            *o += match entry.metadata() {
                                Ok(p) => p.file_size(),
                                Err(_) => 0,
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
}

// Common code for recurse_dirs and walk_dirs
fn walk_and_recurse_dirs_inner<T>(path: T, wc: &WalkConfig, pb: &ProgressBar, pos: Arc<Mutex<&mut u64>>)
where
    T: DirectoryMetaData,
{
    let metadata = path.get_metadata();
    let entry = Rc::new(path.get_path());

    if metadata.is_dir() {
        let base_path = entry.to_path_buf().canonicalize().unwrap();

        DIR_LIST.lock().unwrap().push(base_path);
    } else {
        **pos.lock().unwrap() += 1;
        pb.set_position(**pos.lock().unwrap());
        let mut actual_file_size = 0;
        if cfg!(unix) {
            #[cfg(target_os = "linux")]
            {
                actual_file_size = match entry.metadata() {
                    Ok(p) => p.size(),
                    Err(_) => 0,
                }
            }
        } else if cfg!(windows) {
            #[cfg(target_os = "windows")]
            {
                actual_file_size = match entry.metadata() {
                    Ok(p) => p.file_size(),
                    Err(_) => 0,
                }
            }
        };

        match wc.min_file_size {
            Some(x) => {
                if actual_file_size > x {
                    file_list_generator(&entry, wc);
                }
            }
            None => {
                match wc.max_file_size {
                    Some(x) => {
                        if actual_file_size < x {
                            file_list_generator(&entry, wc);
                        }
                    }
                    None => file_list_generator(&entry, wc),
                };
            }
        };
    }
}

/// Used to recursively capture path entries and capture them separately in two separate Vecs.
/// DIR_LIST is used to hold Directory paths.
/// FILE_LIST is used to hold File.
pub fn recurse_dirs(item: &PathBuf, wc: &WalkConfig, pb: &ProgressBar, pos: Arc<Mutex<&mut u64>>) {
    if item.is_dir() {
        if let Ok(paths) = fs::read_dir(item) {
            for path in paths {
                walk_and_recurse_dirs_inner(&path, wc, pb, pos.clone());
                recurse_dirs(&path.unwrap().path(), wc, pb, pos.clone());
            }
        }
    }
}

/// DIR_LIST is used to hold Directory paths.
/// FILE_LIST is used to hold File paths.
/// But uses WalkDir and Rayon to make it fast.
pub fn walk_dirs(item: &PathBuf, threads: u8, wc: &WalkConfig, pb: &ProgressBar, pos: Arc<Mutex<&mut u64>>) {
    if item.is_dir() {
        let _: Vec<_> = WalkDir::new(item)
            .skip_hidden(false)
            .max_depth(wc.max_depth.unwrap())
            .parallelism(jwalk::Parallelism::RayonNewPool(threads.into()))
            .into_iter()
            .par_bridge()
            .filter_map(|dir_entry| {
                walk_and_recurse_dirs_inner(&dir_entry, wc, pb, pos.clone());
                Some(())
            })
            .collect();
    }
}

pub enum LogLevel {
    INFO,
    ERROR,
}

pub fn log(level: LogLevel, message: &str) {
    let timestamp_fmt: &str = "[%Y-%m-%d %H:%M:%S.%3f]";
    let now = Local::now();
    let timestamp: DateTime<Local> =
        DateTime::from_naive_utc_and_offset(now.naive_utc(), *now.offset());
    let colored_level = match level {
        LogLevel::INFO => "INFO".bright_yellow(),
        LogLevel::ERROR => "ERROR".bright_red(),
    };
    let print = format!(
        "\n{} {}: {}",
        timestamp.format(timestamp_fmt),
        colored_level,
        message
    );

    match level {
        LogLevel::INFO => println!("{}", print),
        LogLevel::ERROR => eprintln!("{}", print),
    }
}

/// This free standing function helps to display all the duplicate file and their respective groups file sizes.
/// It filters for duplicate files from the provided arc_vec_paths HashMap, and figures out the file sizes for each
/// group based on arc_capacities HashMap. Once the filtering and printing to screen is completed, it return the total number of duplicate records count
pub fn print_duplicates<T, U, K>(
    arc_vec_paths: &mut Arc<Mutex<HashMap<K, T>>>,
    arc_capacities: &Arc<Mutex<HashMap<K, U>>>,
    print_config: PrinterConfig,
) -> (u64, u64)
where
    T: IntoIterator + ExactSize + Clone + Paths,
    <T as IntoIterator>::Item: Debug + Displayer,
    U: AsF64,
    K: Eq + Hash,
{
    let mut duplicates_count: u64 = 0;
    let mut duplicates_total_size: u64 = 0;
    let mut arc_vec_paths = arc_vec_paths.lock().unwrap();

    let arc_capacities = arc_capacities.lock().unwrap();

    arc_vec_paths
        .iter_mut()
        .filter(|x| x.1.len() > 1)
        .for_each(|x| duplicates_count += x.1.len() as u64);

    let filtered_duplicates_result = arc_vec_paths.iter_mut().filter(|x| x.1.len() > 1);
    let mut filtered_duplicates_result: Vec<(&K, &T)> = filtered_duplicates_result
        .map(|(&ref k, v)| (k, &*v))
        .collect();

    let sort_by = print_config.sort_order.0;
    let order_by = print_config.sort_order.1;

    match sort_by {
        SortBy::FileType => {
            // Sorts the duplicates based on the file extension
            filtered_duplicates_result.sort_by(|a, b| {
                a.1.get_path()
                    .extension()
                    .unwrap_or(&OsStr::default())
                    .cmp(&b.1.get_path().extension().unwrap_or(&OsStr::default()))
            });
        }
        SortBy::FileSize => {
            // Sorts the duplicates based on the file sizes
            filtered_duplicates_result.sort_by(|a, b| {
                let x = arc_capacities.get(a.0).unwrap();
                let x2 = arc_capacities.get(b.0).unwrap();
                x.cast().total_cmp(&x2.cast())
            });
        }
        SortBy::Both => {
            // Sorts the duplicates based on the file sizes
            filtered_duplicates_result.sort_by(|a, b| {
                let x = arc_capacities.get(a.0).unwrap();
                let x2 = arc_capacities.get(b.0).unwrap();
                x.cast().total_cmp(&x2.cast())
            });

            // Sorts the duplicates based on the file extension
            filtered_duplicates_result.sort_by(|a, b| {
                a.1.get_path()
                    .extension()
                    .unwrap_or(&OsStr::default())
                    .cmp(&b.1.get_path().extension().unwrap_or(&OsStr::default()))
            });
        }
    };

    match sort_by {
        SortBy::FileType => {}
        SortBy::FileSize | SortBy::Both => {
            if let Some(o) = order_by {
                match o {
                    OrderBy::Asc => {}
                    OrderBy::Desc => filtered_duplicates_result.reverse(),
                }
            }
        }
    };

    log(LogLevel::INFO, "Finished\n");

    if print_config.file.is_none() {
        println!("######## {} ########", "Report".bright_yellow().blink());
        // Prints the duplicates to the Screen
        for (u, (i, k)) in filtered_duplicates_result.into_iter().enumerate() {
            let x = arc_capacities.get(i).unwrap();
            let y = human_bytes(x.cast());
            duplicates_total_size += x.cast() as u64;
            let list = k.clone().into_iter().collect::<Vec<_>>();

            println!(
                "\nClone {:?}, {} ({} bytes) each * {}",
                u+1,
                y,
                x.cast(),
                list.len()
            );
            for i in list {
                println!("      {}", i.to_string().bright_blue());
            }
        }
    } else {
        // Write the output to a file
        let mut writer = BufWriter::new(print_config.file.unwrap());

        log(LogLevel::INFO, "Writing the output to the file");

        match print_config.output_style {
            OutputStyle::Default => {
                for (u, (i, k)) in filtered_duplicates_result.into_iter().enumerate() {
                    let x = arc_capacities.get(i).unwrap();
                    let y = human_bytes(x.cast());
                    let list = k.clone().into_iter().collect::<Vec<_>>();

                    duplicates_total_size += x.cast() as u64;

                    let header = format!(
                        "\nClone {:?}, {} ({} bytes) each * {}\n",
                        u+1,
                        y,
                        x.cast(),
                        list.len()
                    );
                    let _ = writer.write(header.as_bytes());

                    for i in k.clone().into_iter() {
                        let message = format!("      {:?}\n", i);
                        let _ = writer.write(message.as_bytes());
                    }
                }
            }
            OutputStyle::JSON => {
                let mut print_json_array = Vec::new();
                let mut json_output = String::new();

                for (u, (i, k)) in filtered_duplicates_result.into_iter().enumerate() {
                    let mut print_json_object = PrinterJSONObject {
                        duplicate_group_no: 0,
                        duplicate_group_count: 0,
                        duplicate_group_bytes_each: 0,
                        duplicate_list: Vec::new(),
                    };

                    let x = arc_capacities.get(i).unwrap();

                    duplicates_total_size += x.cast() as u64;

                    print_json_object.duplicate_group_no = u+1;
                    print_json_object.duplicate_group_count =
                        k.clone().into_iter().collect::<Vec<_>>().len();
                    print_json_object.duplicate_group_bytes_each = x.cast() as usize;

                    for i in k.clone().into_iter() {
                        print_json_object.duplicate_list.push(i.to_string());
                    }

                    print_json_array.push(print_json_object);

                    // Serialize it to a JSON string.
                    if let Ok(o) = serde_json::to_string_pretty(&print_json_array) {
                        json_output = o;
                    } else {
                        log(LogLevel::ERROR, "Failed to Serialize to JSON String")
                    }
                }

                let _ = writer.write(json_output.as_bytes());
            }
        };

        log(LogLevel::INFO, "Finished writing to the file");
    }

    (duplicates_count, duplicates_total_size)
}
