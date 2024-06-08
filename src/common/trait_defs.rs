// Copyright (c) 2024 Venkatesh Omkaram

use std::{
    ffi::OsString,
    fs::{self, DirEntry},
    path::PathBuf, time::SystemTime,
};

pub(crate) trait DirectoryMetaData {
    fn get_metadata(&self) -> fs::Metadata;
    fn get_path(&self) -> PathBuf;
}

impl<'a> DirectoryMetaData for &'a Result<jwalk::DirEntry<((), ())>, jwalk::Error> {
    fn get_metadata(&self) -> fs::Metadata {
        self.as_ref().unwrap().metadata().unwrap()
    }

    fn get_path(&self) -> PathBuf {
        self.as_ref().unwrap().path()
    }
}

impl<'a> DirectoryMetaData for &'a Result<DirEntry, std::io::Error> {
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

impl ExactSize for Vec<(OsString, Option<SystemTime>)> {
    fn len(&self) -> usize {
        self.len()
    }
}

pub trait Paths {
    fn get_path(&self) -> PathBuf;
    fn get_self(self) -> Self;
}

impl Paths for Vec<(OsString, Option<SystemTime>)> {
    fn get_path(&self) -> PathBuf {
        self.get(0).unwrap().clone().0.into()
    }
    
    fn get_self(self) -> Self {
        self
    }
}

impl Paths for Vec<PathBuf> {
    fn get_path(&self) -> PathBuf {
        self.get(0).unwrap().to_path_buf()
    }
    
    fn get_self(self) -> Self {
        self
    }
}

pub trait Times {
    fn get_modified(&self) -> Option<SystemTime>;
}

impl Times for (OsString, Option<SystemTime>) {
    fn get_modified(&self) -> Option<SystemTime> {
        self.1
    }
}

impl Times for PathBuf {
    fn get_modified(&self) -> Option<SystemTime> {
     unimplemented!()
    }
}

pub trait Displayer {
    fn to_string(&self) -> String;
}

impl Displayer for (OsString, Option<SystemTime>) {
    fn to_string(&self) -> String {
        self.clone().0.into_string().unwrap_or_else(|_| "failure".to_string())
    }
}

impl Displayer for OsString {
    fn to_string(&self) -> String {
        self.clone().into_string().unwrap_or_else(|_| "failure".to_string())
    }
}

impl Displayer for PathBuf {
    fn to_string(&self) -> String {
        self.clone().into_os_string().to_string()
    }
}