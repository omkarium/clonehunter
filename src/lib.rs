//! This library crate is only for internal use. Do not use it in your own projects independently.

// Copyright (c) 2024 Venkatesh Omkaram
mod traits;

use hashbrown::HashMap;
use human_bytes::human_bytes;
use indicatif::ProgressBar;
use jwalk::WalkDir;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use rayon::iter::{IntoParallelRefMutIterator, ParallelBridge, ParallelIterator};
use std::{
    fmt::Debug,
    fs,
    hash::Hash,
    io::{stdin, stdout, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use traits::*;

#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

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

    println!("\nYou typed: {}\n", confirmation);

    confirmation
}

/// A simple macro which prints two items only when verbose printing is specified.
/// VERBOSE is a RwLock
#[macro_export]
macro_rules! logger {
    ($value: literal, $item: expr, $item2: expr) => {
        use common::VERBOSE;

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

/// Used to store Hash Digest as BigInt and Path of Files for Sorting Operations
#[derive(Ord, PartialOrd, PartialEq, Eq, Debug)]
struct Grouper {
    hash_to_bigint: BigUint,
    path_buf: PathBuf,
}

// Common code for recurse_dirs and walk_dirs
fn walk_and_recurse_dirs_inner<T>(path: T, ext: Option<&str>)
where
    T: DirectoryMetaData,
{
    let metadata = path.get_metadata();
    let entry = Rc::new(path.get_path());

    if metadata.is_dir() {
        let base_path = entry.to_path_buf();

        DIR_LIST.lock().unwrap().push(base_path);
    } else {
        if let Some(x) = entry.extension() {
            if let Some(ext) = ext {
                let mut vec_ext = ext.split(",");
                if vec_ext.any(|y| x.eq(y)) {
                    FILE_LIST.lock().unwrap().push(entry.to_path_buf());
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
                                    *o += match entry.get_path().metadata() {
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
                FILE_LIST.lock().unwrap().push(entry.to_path_buf());
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
                                *o += match entry.get_path().metadata() {
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
}

/// Used to recursively capture path entries and capture them separately in two separate Vecs.
/// DIR_LIST is used to hold Directory paths.
/// FILE_LIST is used to hold File.
pub fn recurse_dirs(item: &PathBuf, ext: Option<&str>) {
    if item.is_dir() {
        if let Ok(paths) = fs::read_dir(item) {
            for path in paths {
                walk_and_recurse_dirs_inner(&path, ext);
                recurse_dirs(&path.unwrap().path(), ext)
            }
        }
    }
}

/// DIR_LIST is used to hold Directory paths.
/// FILE_LIST is used to hold File paths.
/// But uses WalkDir and Rayon to make it fast.
pub fn walk_dirs(item: &PathBuf, max_depth: usize, threads: u8, ext: Option<&str>) {
    if item.is_dir() {
        let _: Vec<_> = WalkDir::new(item)
            .skip_hidden(false)
            .max_depth(max_depth)
            .parallelism(jwalk::Parallelism::RayonNewPool(threads.into()))
            .into_iter()
            .par_bridge()
            .filter_map(|dir_entry| {
                walk_and_recurse_dirs_inner(&dir_entry, ext);
                Some(())
            })
            .collect();
    }
}

/// This free standing function helps to display all the duplicate file and their respective groups file sizes.
/// It filters for duplicate files from the provided arc_vec_paths HashMap, and figures out the file sizes for each
/// group based on arc_capacities HashMap. Once the filtering and printing to screen is completed, it return the total number of duplicate records count
pub fn print_duplicates<T, U, K>(
    arc_vec_paths: &mut Arc<Mutex<HashMap<K, T>>>,
    arc_capacities: &Arc<Mutex<HashMap<K, U>>>,
) -> (u64, u64)
where
    T: IntoIterator + ExactSize + Clone,
    <T as IntoIterator>::Item: Debug,
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

    for (u, (i, k)) in filtered_duplicates_result.enumerate() {
        let x = arc_capacities.get(i).unwrap();
        let y = human_bytes(x.cast());
        duplicates_total_size += x.cast() as u64;
        println!("\nDuplicate {:?}, {} ({} bytes) each", u, y, x.cast());
        for i in k.clone().into_iter() {
            println!("      {:?}", i);
        }
    }

    (duplicates_count,duplicates_total_size)
}

/// This function helps in sorting the vec of Hash digest and filePath.
/// Once the sort is finished it will group Duplicates with the help of HashMap and Parallel Iterator
pub fn sort_and_group_duplicates(
    list_hashes: &[(BigUint, &Path)],
) -> Arc<Mutex<HashMap<BigUint, Vec<PathBuf>>>> {
    let num_hashes_vec = Arc::new(Mutex::new(Vec::new()));
    let bar = ProgressBar::new(num_hashes_vec.lock().unwrap().len() as u64);
    let hashmap_accumulator: Arc<Mutex<HashMap<BigUint, Vec<PathBuf>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    for (i, k) in list_hashes.into_iter() {
        num_hashes_vec.lock().unwrap().push(Grouper {
            hash_to_bigint: i.to_owned(),
            path_buf: k.to_owned().into(),
        });
    }

    num_hashes_vec.lock().unwrap().sort_unstable();

    println!("\nFinding duplicates...");

    let mut num_hashes_vec = num_hashes_vec.lock().unwrap();

    num_hashes_vec.par_iter_mut().for_each(|x| {
        let r = &x.path_buf;
        let r1 = &x.hash_to_bigint;
        if hashmap_accumulator.lock().unwrap().contains_key(r1) {
            let mut new = hashmap_accumulator
                .lock()
                .unwrap()
                .get(r1)
                .unwrap()
                .to_owned();
            new.push(r.clone());
            hashmap_accumulator.lock().unwrap().insert(r1.clone(), new);
        } else {
            hashmap_accumulator
                .lock()
                .unwrap()
                .insert(r1.clone(), vec![r.clone()]);
        }
        bar.inc(1);
    });

    hashmap_accumulator
}
