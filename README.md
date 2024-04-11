## CloneHunter: An ultra simple command line utility that identifies groups of identical files and displays them to the console.

[![crate][crate-image]][crate-link]
![MIT licensed][license-image]
![Rust Version][rustc-image]
[![Downloads][downloads-image]][crate-link]
![Category][category-image]

Copyright (c) 2024 Venkatesh Omkaram


## How to Use?
If you have the program as a binary executable then run, `clonehunter --help` for usage. 
If you are running this program via Cargo, then run `cargo run -- --help` from the root folder for usage.

To install the program permanently on your system do `cargo install clonehunter`.

# Example usage:
 ```
clonehunter your-folder-path -t 12 -c -v
 ```
`-c` stands for checksum. If you pass this option, clonehunter will find the file clones (aka duplicate files or identical files) based on a partial checksum by reading bytes from the beginning and the ending of the file content.
If you do not pass -c option, then clonehunter will scan for clones based on a combination of file name, modified time and file size hash combined.

`-v` stands for verbose. It prints the hashes of each and every file for you to compare and manually figure out clones.

`-t` stands for threads. Choose the number of threads to allocate the program to hunt. In the above example I am using 12 threads.

## Some considerations
The program scans and outputs identical files based on best effort basis. This means that not all files it reports on can be deemed as 'Absolutely identical'. So, the key term here is "Possibly identical". This tool can be used when you want to do a quick analysis to see which files are POSSIBLY identical. This tool must not be used in critical places and business solutions, and must not be considered as the source of truth to delete any of those found identical files.

Also, using this tool will not destroy any files on your machine. There are no delete or write operations performed in the code. If you found any such strangeness, please raise an Issue. At most, the tool reports incorrect identical files or skips some of the files which are not accessible due to file permission restrictions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/clonehunter.svg
[crate-link]: https://crates.io/crates/clonehunter
[license-image]: https://img.shields.io/badge/License-MIT_or_Apache_2.0-yellow.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.75+-blue.svg
[downloads-image]: https://img.shields.io/crates/d/clonehunter.svg
[category-image]: https://img.shields.io/badge/category-Duplicate_Files_Finder-darkred.svg
