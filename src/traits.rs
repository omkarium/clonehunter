// Copyright (c) 2024 Venkatesh Omkaram

use std::{ffi::OsString, fs::{self, DirEntry}, path::PathBuf};

pub(crate) trait DirectoryMetaData {
    fn get_metadata(&self) -> fs::Metadata;
    fn get_path(&self) -> PathBuf;
}

impl<'a> DirectoryMetaData
    for &'a Result<jwalk::DirEntry<((), ())>, jwalk::Error>
{
    fn get_metadata(&self) -> fs::Metadata {
        self.as_ref().unwrap().metadata().unwrap()
    }

    fn get_path(&self) -> PathBuf {
        self.as_ref().unwrap().path()
    }
}

impl<'a> DirectoryMetaData
    for &'a Result<DirEntry, std::io::Error>
{
    fn get_metadata(&self) -> fs::Metadata {
        self.as_ref().unwrap().metadata().unwrap()
    }

    fn get_path(&self) -> PathBuf {
        self.as_ref().unwrap().path()
    }
}


// A simple trait to cast implementors to f64. Pretty useful in Function which takes Generic arguments
pub trait AsF64 {
    fn cast(&self) -> f64;
}

impl AsF64 for u64 {
    fn cast(&self) -> f64 {
        *self as f64
    }
}

// A simple trait to return length of its implementors. Pretty useful in Function which takes Generic arguments
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

pub trait Paths {
    fn get_path(&self) -> PathBuf;
}

impl Paths for Vec<OsString> {
    fn get_path(&self) -> PathBuf {
        self.get(0).unwrap().into()
    }
}

impl Paths for Vec<PathBuf> {
    fn get_path(&self) -> PathBuf {
        self.get(0).unwrap().to_path_buf()
    }
}