// Copyright (c) 2024 Venkatesh Omkaram

use std::fmt;

use clap::Parser;

#[derive(clap::Args, Debug, Clone)]
#[command(disable_version_flag = true)]
pub struct Delete {}

#[derive(clap::Args, Debug, Clone)]
#[command(disable_version_flag = true)]
pub struct HunterOptions {
    /// Pass the Source Directory (This is the directory under which will be looking for the identical files (aka 'Clones')
    pub source_dir: String,
    /// Pass the Maximum Depth of directories to scan
    #[clap(short, long, default_value_t = 10)]
    pub max_depth: usize,
    /// Use this option if you don't wish to specify a max_depth.
    #[clap(long, default_value_t = false)]
    pub no_max_depth: bool,
    /// Hunt for clones by performing partial file checksums.
    #[clap(short, long, default_value_t = false)]
    pub checksum: bool,
    /// Find clones for a specific file type. Example -e pdf or -e pdf,txt,mp4
    #[clap(short, long)]
    pub extension: Option<String>,
    /// Sorts the output.
    #[clap(short, long, value_enum, default_value_t = SortBy::FileType)]
    pub sort_by: SortBy,
    /// Prints the sorted output either in the Ascending or the Descending order
    #[clap(short, long)]
    pub order_by: Option<OrderBy>,
    /// Targets maximum file sizes in bytes
    /// 
    /// (Additionally you can also use "KiB", "MiB", "GiB", "KB", "MB", "GB". For example: "150 KiB" with quotes)
    #[clap(short = 'j', long)]
    pub max: Option<String>,
    /// Targets minimum file sizes in bytes (not to be used with --max)
    /// 
    /// (Additionally you can also use "KiB", "MiB", "GiB", "KB", "MB", "GB". For example: "13 MiB" with quotes)
    #[clap(short = 'l', long)]
    pub min: Option<String>,
    /// Write the output to a file using various styles (requires `-f`)
    #[clap(short = 'u', long, requires = "output_file")]
    pub output_style: Option<OutputStyle>,
    /// Write the output to a file (requires `-u`)
    #[clap(short = 'f', long, requires = "output_style")]
    pub output_file: Option<String>,
}

#[derive(clap::Args, Debug, Clone)]
#[command(disable_version_flag = true)]
pub struct DeleteOptions {
    /// Provide the JSON input file path which you generated using the `hunt -u json -f report_file.json` command
    #[clap(short, long)]
    pub input_file: String,
    /// Dry run (test to see if the deletion works without deleting anything)
    #[clap(long, default_value_t = false)]
    pub dry_run: bool,
}

/// SortBy User Option
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum SortBy {
    FileType,
    FileSize,
    Both,
}

/// OrderBy User Option
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum OrderBy {
    Asc,
    Desc,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum OutputStyle {
    /// Use the default style of printing the output to a file
    Default,
    /// Use the JSON style of printing the output to a file
    JSON,
}

#[derive(clap::Subcommand, Debug, Clone)]
//#[command(disable_version_flag = true)]
pub enum Command {
    /// Search for clones (duplicates)
    Hunt(HunterOptions),
    /// Delete the extracted clones
    Delete(DeleteOptions),
}

#[derive(Parser, Clone)]
#[command(author="@github.com/omkarium", version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
    /// Threads to speed up the execution
    #[clap(short, long, default_value_t = 8)]
    pub threads: u8,
    /// Print verbose output
    #[clap(short, long, default_value_t = false)]
    pub verbose: bool,
}

impl fmt::Display for OrderBy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderBy::Asc => write!(f, "{}", "Asc"),
            OrderBy::Desc => write!(f, "{}", "Desc"),
        }
    }
}

impl fmt::Display for OutputStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OutputStyle::Default => write!(f, "{}", "Default"),
            OutputStyle::JSON => write!(f, "{}", "JSON"),
        }
    }
}
