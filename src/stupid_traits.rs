// Copyright (c) 2024 Venkatesh Omkaram

// These are some stupid traits created made for Abstractions. I regret having these lines but I don't see how to have a function
// generic over T, where T is some external library Concrete types, but still I need to be able to make method call on T. Is this how everyone does it?
// Just to save a few lines of code duplication in lib.rs/fn walk_and_recurse_dirs_inner(), I wrote all of this nonsense. 
// Need to find a way to avoid writing this much. Maybe Macros help here

use std::{ffi::OsString, fs::{self, DirEntry, Metadata}, path::PathBuf};

pub(crate) trait CommonDirWalker<K, U> {
    fn metadata_custom(&self) -> K;
    fn unwrap_custom(self) -> U;
}

impl CommonDirWalker<Result<fs::Metadata, jwalk::Error>, jwalk::DirEntry<((), ())>>
    for Result<jwalk::DirEntry<((), ())>, jwalk::Error>
{
    fn metadata_custom(&self) -> Result<fs::Metadata, jwalk::Error> {
        self.as_ref().unwrap().metadata()
    }

    fn unwrap_custom(self) -> jwalk::DirEntry<((), ())> {
        self.unwrap()
    }
}

impl CommonDirWalker<Result<fs::Metadata, std::io::Error>, DirEntry>
    for Result<DirEntry, std::io::Error>
{
    fn metadata_custom(&self) -> Result<fs::Metadata, std::io::Error> {
        self.as_ref().unwrap().metadata()
    }

    fn unwrap_custom(self) -> DirEntry {
        self.unwrap()
    }
}

pub(crate) trait MetaDataPathBufCommon {
    fn result_unwrap(self) -> Metadata;
    fn get_path(&self) -> PathBuf;
}

impl MetaDataPathBufCommon for Result<fs::Metadata, jwalk::Error> {
    fn result_unwrap(self) -> Metadata {
        self.unwrap()
    }

    fn get_path(&self) -> PathBuf {
        unimplemented!()
    }
}

impl MetaDataPathBufCommon for Result<fs::Metadata, std::io::Error> {
    fn result_unwrap(self) -> Metadata {
        self.unwrap()
    }

    fn get_path(&self) -> PathBuf {
        unimplemented!()
    }
}

impl MetaDataPathBufCommon for DirEntry {
    fn result_unwrap(self) -> Metadata {
        unimplemented!()
    }

    fn get_path(&self) -> PathBuf {
        self.path()
    }
}

impl MetaDataPathBufCommon for jwalk::DirEntry<((), ())> {
    fn result_unwrap(self) -> Metadata {
        unimplemented!()
    }

    fn get_path(&self) -> PathBuf {
        self.path()
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
