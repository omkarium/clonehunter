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

## Example usage:
 ```
clonehunter your-folder-path -t 12 -c -v -m 50 -e pdf -s both -o asc
 ```
`-c` stands for checksum. If you pass this option, clonehunter will find the file clones (aka duplicate files or identical files) based on a partial checksum by reading bytes from the beginning and the ending of the file content.
If you do not pass -c option, then clonehunter will scan for clones based on a combination of file name, modified time and file size hash combined.

`-m` stands for max depth. The number after -m indicates how many sub levels we need to look for clones. The default value is 10. If you do not wish to specify a max depth, then pass the option `--no-max-depth`

`-v` stands for verbose. It prints the hashes of each and every file for you to compare and manually figure out clones.

`-t` stands for threads. Choose the number of threads to allocate the program to hunt. In the above example, I am using 12 threads.

`-e` stands for extension and this feature enables you to target specific file types aka file extensions. In the above example, I am targeting `pdf`. If you do not want to target any specific file types, then do not use the option. You can also pass something like `pdf,txt,mp4`. This will target all the three file types.

`-s` stands for sort-by and this feature helps to sort the output to be printed on the screen based on 3 types. 
The three types are `file-type`, `file-size`, and `both`. When you pass the value as `both` the output will be sorted based on `file-size` first and `file-type` next.

`-o` stands for order-by and this feature helps to order the sorted output which was achieved by the `-s` option. This option only applies for `-s both` or `-s file-size`. It does not matter what the order is when your already sorted using the file-type alone.

## How it works?
There are two modes the program looks for duplicate files.
1. Without checksum calculation
2. With checksum calculation (by passing `-c`)

### Without checksum calculation: 
This applies when you do not pass a `-c` option. The program will look for clones based on a hash operation performed on the combination of file size, file name and modified times.
- If two file names and file size are the same, that does not qualify as a clone
- If two files file sizes and modified times are the same, that does not qualify as a clone
- If two file names and modified times are the same, that does not qualify as a clone
- Finally, if two file names, file size and modified times all are the same then it is definitely a clone

Now, the question may arise, what if there are two files with different file names, but the content inside is absolute the same, regardless of the modified time? It must be a clone correct? Yes. This is where 'With checksum calculation' `-c` option helps.

### With checksum calculation:
A checksum is also a hash, but this is performed on the file content instead of the file metadata such as name, size and time.

- If the file size is less than (<) 1 MB, then a checksum is performed on entire length of the file data.
- If the file size is greater than (>) 1 MB, then the program takes the first 1 MB, the last 1 MB of the file and the file size is all combined together and a hash is taken on it. This way, we can be sure if two files are absolutely clones without performing a checksum on the whole length of the file



## Some considerations
The program scans and outputs identical files based on best effort basis. This means that not all files it reports on can be deemed as 'Absolutely identical'. So, the key term here is "Possibly identical". This tool can be used when you want to do a quick analysis to see which files are POSSIBLY identical. This tool must not be used in critical places and business solutions, and must not be considered as the source of truth to delete any of those found identical files.

Also, using this tool will not destroy any files on your machine. There are no delete or write operations performed in the code. If you found any such strangeness, please raise an Issue. At most, the tool reports incorrect identical files or skips some of the files which are not accessible due to file permission restrictions.

## Regarding files with 0 bytes size
If you are running clonehunter on bunch of different files or file types, let's say some mp4, pdf, txt etc but they all have file sizes of 0 bytes, and if you used the -c checksum option, you will observe all of the 0 size files grouped together as duplicates in the final output on the screen.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/clonehunter.svg
[crate-link]: https://crates.io/crates/clonehunter
[license-image]: https://img.shields.io/badge/License-MIT_or_Apache_2.0-yellow.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.75+-blue.svg
[downloads-image]: https://img.shields.io/crates/d/clonehunter.svg
[category-image]: https://img.shields.io/badge/category-Duplicate_Files_Finder-darkred.svg
