// Copyright (c) 2024 Venkatesh Omkaram

use clonehunter::{common::core::{log, print_duplicates, FileMetaData, LogLevel, PrinterConfig}, logger};
use fxhash::FxHasher64;
use hashbrown::HashMap;
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use md5::compute;
use num_bigint::BigUint;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::{ffi::OsString, path::Path };
use std::sync::Mutex;
use std::{
    fmt::Write,
    fs::File,
    hash::{Hash, Hasher},
    io::{BufReader, Read},
    path::PathBuf,
    sync::Arc,
};
//use gxhash::GxHasher;

#[cfg(target_os = "linux")]
use std::os::unix::fs::MetadataExt;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

/// The working of this function is very straightforward. It takes a List of File paths from the main function
/// Instantiates a Progress bar, creates a ThreadPool and check if checksum is true or false.
/// If the checksum is false, it means we won't need to do a checksum to hunt for duplicate files. Based on the file_name,
/// modified_date and file_size a Hash is generated and this hash and its respective file_path will be stored in a HashMap.
/// Finally, we will group file paths in the HashMap using the hash digest and extract them to a separate list.
/// That list is finally sent to print_duplicates function to filter duplicate files and print them to the screen
///
///
/// The only difference between no checksum and checksum is, in checksum, we will do some additional steps to ensure if the files are
/// truly duplicate. No checksum is easy and fast, but using the checksum feature is reliable. Also, the checksum feature is not
/// going to calculate the checksum of each file to the end of file. Instead, it will only generate a checksum based on first few thousand
/// and last few thousand bytes. This makes it fast and not resource hungry.
pub fn hunt(paths: Vec<PathBuf>, checksum: bool, threads: u8, print_config: PrinterConfig) -> (u64, u64) {
    let list_hashes: Arc<Mutex<Vec<(BigUint, &std::path::Path)>>> =
        Arc::new(Mutex::new(Vec::new()));

    let list_hashes_caps: Arc<Mutex<HashMap<BigUint, u64>>> = Arc::new(Mutex::new(HashMap::new()));
    let pb = Arc::new(Mutex::new(ProgressBar::new(paths.len() as u64)));

    let mut hashmap_for_duplicates_meta: Arc<Mutex<HashMap<u64, Vec<OsString>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let hashmap_for_duplicates_meta_caps: Arc<Mutex<HashMap<u64, u64>>> =
        Arc::new(Mutex::new(HashMap::new()));

    pb.lock().unwrap().set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos} /{percent}% hashes completed ({eta_precise}) {msg}")
    .unwrap()
    .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
    .progress_chars("#>-"));

    log(LogLevel::INFO, "Generating hashes");

    let pb_increment: Arc<Mutex<u64>> = Arc::new(Mutex::new(1));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.into())
        .build()
        .unwrap();

    if !checksum {
        pool.install(|| {
            rayon::scope(|s| {
                for path in &paths {
                    let pb = pb.clone();
                    let pb_increment = pb_increment.clone();

                    if let Some(file_name) = path.as_path().file_name() {
                        if let Some(file_name) = file_name.to_str() {
                            let modified_date = if let Ok(modified_date) = path.metadata() {
                                if let Ok(system_time) = modified_date.modified() {
                                    system_time
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            };

                            let mut file_size = 0;

                            if cfg!(unix) {
                                #[cfg(target_os = "linux")]
                                {
                                    file_size = path.metadata().unwrap().size();
                                }
                            } else if cfg!(windows) {
                                #[cfg(target_os = "windows")]
                                {
                                    file_size = path.metadata().unwrap().file_size();
                                }
                            };

                            let hashmap_for_duplicates_meta = hashmap_for_duplicates_meta.clone();
                            let hashmap_for_duplicates_meta_caps =
                                hashmap_for_duplicates_meta_caps.clone();

                            s.spawn(move |_| {
                                let pb = pb.clone();
                                pb.lock()
                                    .unwrap()
                                    .set_position(*pb_increment.lock().unwrap());
                                *pb_increment.lock().unwrap() += 1;
                                let duplicates_by_metadata = FileMetaData {
                                    file_name,
                                    modified_date,
                                    file_size,
                                };

                                let mut file_metadata_hasher = FxHasher64::default();
                                duplicates_by_metadata.hash(&mut file_metadata_hasher);

                                let hash_u64: u64 = file_metadata_hasher.finish();
                                hashmap_for_duplicates_meta_caps
                                    .lock()
                                    .unwrap()
                                    .insert(hash_u64, file_size);

                                logger!("hash {:?} -> file {:?}", hash_u64, path);

                                if hashmap_for_duplicates_meta
                                    .lock()
                                    .unwrap()
                                    .contains_key(&hash_u64)
                                {
                                    let mut path_vec = hashmap_for_duplicates_meta
                                        .lock()
                                        .unwrap()
                                        .get(&hash_u64)
                                        .unwrap()
                                        .to_owned();

                                    path_vec.push(path.to_owned().into_os_string());

                                    hashmap_for_duplicates_meta
                                        .lock()
                                        .unwrap()
                                        .insert(hash_u64, path_vec);
                                } else {
                                    hashmap_for_duplicates_meta
                                        .lock()
                                        .unwrap()
                                        .insert(hash_u64, vec![path.to_owned().into_os_string()]);
                                }
                            });
                        }
                    }
                }
            })
        });

        pb.lock().unwrap().finish_and_clear();

        log(LogLevel::INFO, "Finding clones");

        print_duplicates(
            &mut hashmap_for_duplicates_meta,
            &hashmap_for_duplicates_meta_caps,
            print_config,
        )
    } else {
        pool.install(|| {
            rayon::scope(|s| {
                for path in &paths {
                    let pb = pb.clone();
                    let pb_increment = pb_increment.clone();

                    let list_hashes = list_hashes.clone();
                    let list_hashes_caps = list_hashes_caps.clone();

                    // Spawn the threads here
                    s.spawn(move |_| {
                        let pb = pb.clone();
                        pb.lock()
                            .unwrap()
                            .set_position(*pb_increment.lock().unwrap());

                        match File::open(path) {
                            Ok(file) => {
                                let mut reader = BufReader::new(file);

                                let file_length = path.metadata().unwrap().len();

                                let hash_combine = if file_length > 2048 {
                                    let mut buffer_front = vec![0; 1024];
                                    let _ = reader.read_exact(&mut buffer_front);

                                    let _ = reader.seek_relative(-1024);

                                    let mut buffer_back = vec![0; 1024];
                                    let _ = reader.read_exact(&mut buffer_back);

                                    compute(
                                        [
                                            buffer_front,
                                            buffer_back,
                                            file_length.to_le_bytes().to_vec(),
                                        ]
                                        .concat(),
                                    )
                                } else {
                                    let mut buffer_full =
                                        vec![
                                            0;
                                            <u64 as TryInto<usize>>::try_into(file_length).unwrap()
                                        ];

                                    let _ = reader.read_exact(&mut buffer_full);

                                    compute(buffer_full)
                                };

                                let hash_to_bigint = hash_combine
                                    .iter()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .concat()
                                    .parse::<BigUint>()
                                    .unwrap();

                                list_hashes
                                    .lock()
                                    .unwrap()
                                    .push((hash_to_bigint.clone(), path.as_path()));

                                list_hashes_caps
                                    .lock()
                                    .unwrap()
                                    .insert(hash_to_bigint, file_length);
                            }
                            Err(e) => log(LogLevel::ERROR, format!("File {:?} {:?}", path, e.kind()).as_str()),
                        }
                        *pb_increment.lock().unwrap() += 1;
                    });
                }
            });
        });

        pb.lock().unwrap().finish_and_clear();

        for (i, k) in &*list_hashes.clone().lock().unwrap() {
            logger!("hash {:?} -> file {:?}", i, k);
        }

        let mut hashmap_group = sort_and_group_duplicates(list_hashes.lock().unwrap().as_slice());

        print_duplicates(&mut hashmap_group, &list_hashes_caps, print_config)
    }
}


// This function helps in sorting the vec of Hash digest and filePath.
// Once the sort is finished it will group Duplicates with the help of HashMap and Parallel Iterator
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

    log(LogLevel::INFO, "Finding clones");

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

// Used to store Hash Digest as BigInt and Path of Files for Sorting Operations
#[derive(Ord, PartialOrd, PartialEq, Eq, Debug)]
struct Grouper {
    hash_to_bigint: BigUint,
    path_buf: PathBuf,
}