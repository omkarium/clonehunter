//! This library crate is only for internal use. Do not use it in your own projects independently.

// Copyright (c) 2024 Venkatesh Omkaram

use human_bytes::human_bytes;
use indicatif::ProgressBar;
use lazy_static::lazy_static;
use num_bigint::BigUint;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::fmt::Debug;
use std::{collections::HashMap, hash::Hash, sync::Arc};
use std::{
    ffi::OsString,
    fs,
    io::{stdin, stdout, Write},
    path::PathBuf,
    sync::{Mutex, RwLock},
    time::SystemTime,
};


lazy_static! {
    /// A Lazy static reference to hold a List of Directory Paths
    pub static ref DIR_LIST: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
    /// A Lazy static reference to hold a list of File Paths
    pub static ref FILE_LIST: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());
    /// A Lazy static reference which hold a RwLock on a bool used for verbose printing
    pub static ref VERBOSE: RwLock<bool> = RwLock::new(false);
}


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
        use super::VERBOSE;

        if *VERBOSE.read().unwrap() {
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

/// Used to recursively capture path entries and capture them separately in two separate Vecs
/// DIR_LIST is used to hold Directory paths
/// FILE_LIST is used to hold File paths
pub fn recurse_dirs(item: &PathBuf) {
    if item.is_dir() {
        if let Ok(paths) = fs::read_dir(item) {
            for path in paths {
                let metadata = path.as_ref().unwrap().metadata();
                let entry = path.as_ref().unwrap();
                if metadata.unwrap().is_dir() {
                    let base_path = entry.path();
                    DIR_LIST.lock().unwrap().push(base_path);
                    recurse_dirs(&entry.path());
                } else {
                    FILE_LIST.lock().unwrap().push(entry.path());
                }
            }
        }
    }
}

/// This free standing function helps to display all the duplicate file and their respective groups file sizes
/// It filters for duplicate files from the provided arc_vec_paths HashMap, and figures out the file sizes for each 
/// group based on arc_capacities HashMap. Once the filtering and printing to screen is completed, it return the total number of duplicate records count
pub fn print_duplicates<T, U, K>(
    arc_vec_paths: &mut Arc<Mutex<HashMap<K, T>>>,
    arc_capacities: &Arc<Mutex<HashMap<K, U>>>,
) -> u64
where
    T: IntoIterator + ExactSize + Clone,
    <T as IntoIterator>::Item: Debug,
    U: AsF64,
    K: Eq + Hash,
{
    let mut duplicates_count = 0;

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
        println!("\nDuplicate {:?}, {} ({} bytes) each", u, y, x.cast());
        for i in k.clone().into_iter() {
            println!("      {:?}", i);
        }
    }

    duplicates_count
}

/// This function helps in Sorting the vec of Hash digest and filePath
/// Once the sort is finished it will group Duplicates with the help of HashMap and Parallel Iterator
pub fn sort_and_group_duplicates(
    list_hashes: Vec<(md5::Digest, &std::path::Path)>,
) -> Arc<Mutex<HashMap<BigUint, Vec<PathBuf>>>> {
    let num_hashes_vec = Arc::new(Mutex::new(Vec::new()));
    let bar = ProgressBar::new(num_hashes_vec.lock().unwrap().len() as u64);
    let hashmap_accumulator: Arc<Mutex<HashMap<BigUint, Vec<PathBuf>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    for (i, k) in list_hashes {
        let hash_to_bigint =
            i.0.iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .concat()
                .parse::<BigUint>()
                .unwrap()
                .into();

        num_hashes_vec.lock().unwrap().push(Grouper {
            hash_to_bigint,
            path_buf: k.to_owned().into(),
        });
    }

    num_hashes_vec.lock().unwrap().sort_unstable();

    println!("\nFinding duplicates...");

    let mut num_hashes_vec = num_hashes_vec.lock().unwrap();

    let _: Vec<_> = num_hashes_vec
        .par_iter_mut()
        .map(|x| {
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
        })
        .collect();

    hashmap_accumulator
}


/// A simple trait to cast implementors to f64. Pretty useful in Function which takes Generic arguments
pub trait AsF64 {
    fn cast(&self) -> f64;
}


impl AsF64 for u64 {
    fn cast(&self) -> f64 {
        *self as f64
    }
}


/// A simple trait to return length of its implementors. Pretty useful in Function which takes Generic arguments
pub trait ExactSize {
    fn len(&self) -> usize;
}


impl ExactSize for Vec<PathBuf> {
    fn len(&self) -> usize {
        self.len()
    }
}


impl ExactSize for Vec<OsString> {
    fn len(&self) -> usize {
        self.len()
    }
}
