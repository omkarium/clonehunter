## CloneHunter: A simple command line utility that identifies groups of identical files and displays them to the console.

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
But first, you need to install `Cargo`.  Read this installation doc [![cargo-install]][cargo-install]

## Commands available
- hunt
- delete

## Example usage of the `hunt` command with full options:
 ```
clonehunter hunt your-folder-path -t 12 -c -v -m 50 -e pdf -s both -o asc -u json -f output-report.json --max "20 MiB"
 ```
Note: the below options are short options. There is a long form available for each short option. Also you do not need to pass all of the above options all the time. The simplest command you can run is `clonehunter hunt .`

`-c` stands for checksum. If you pass this option, clonehunter will find the file clones (aka duplicate files or identical files) based on a partial checksum by reading bytes from the beginning and the end of the file.
If you do not pass -c option, then clonehunter will scan for clones based on a combination of file name, modified time and file size hash combined. Use this if want to hunt for clones aggressively.

`-m` stands for max depth. The number after -m indicates how many sub levels we need to look for clones. The default value is 10. If you do not wish to specify a max depth, then pass the option `--no-max-depth` explicitly.

`-v` stands for verbose. This options helps to print the hashes of each and every file for you to compare and manually figure out clones.

`-t` stands for threads. Choose the number of threads to allocate the program to hunt. In the above example, I am using 12 threads. If you do not provide this by the default threads used will be 8.

`-e` stands for extension and this feature enables you to target specific file types aka file extensions. In the above example, I am targeting `pdf`. If you do not want to target any specific file types, then do not use the option. You can also pass something like `pdf,txt,mp4`. This will target all the three file types.

`-s` stands for sort-by and this feature helps to sort the output to be printed on the screen based on 3 variants. 
The three variants are `file-type`, `file-size`, and `both`. When you pass the value as `both` the output will be sorted based on `file-size` first and `file-type` next.

`-o` stands for order-by and this feature helps to order the sorted output which was achieved by the `-s` option. This option only applies when `-s both` or `-s file-size` is in effect. Also, it does not matter what the order is when your already sorted using the file-type alone.

`-u` stands for output-style. There are two variants. The first is `json` and the second is `default`. Basically this determines the output style while writing the final report to a file

`-f` stands for output-file. As the name implies, this writes the final report to a file using a certain output-style given by `-u`. 

> Note: If you want to use the `delete` command to delete the found clones, then you need to use `-u json -f report-name.json` to first generate a JSON report which then can be later feed as input to the delete command.

`--min` stands for minimum file size. This options targets the minimum file sizes in bytes (not to be used with --max)
    (Additionally you can also use "KiB", "MiB", "GiB", "KB", "MB", "GB". For example: "13 MiB" with quotes)

`--max` stands for maximum file size. This options targets the maximum file sizes in bytes (not to be used with --min)
    (Additionally you can also use "KiB", "MiB", "GiB", "KB", "MB", "GB". For example: "13 MiB" with quotes)

### How the core algorithm works?
There are two modes the program looks for duplicate files.
1. Without checksum calculation
2. With checksum calculation (by passing `-c`)


### Without checksum calculation: 
This applies when you do not pass a `-c` option. The program will look for clones based on a hash operation performed on the combination of file size, file name and modified times.
- If two file names and file size are the same, that does not qualify as a clone because either the file content can be different or the modified times can be different
- If two files file sizes and modified times are the same, that does not qualify as a clone because the file content can be different
- If two file names and modified times are the same, that does not qualify as a clone because the file sizes can be different
- Finally, if two file names, file size and modified times all are the same then it is definitely a clone

Now, a question may arise i.e, what if there are two files with different file names, but the content inside is absolute the same, regardless of the modified time? It must be a clone correct? Yes. This is where 'With checksum calculation' `-c` option helps.


### With checksum calculation:
A checksum is also a hash, but this is performed on the file content instead of the file metadata such as name, size and time.

- If the file size is less than (<) 1 MB, then a checksum is performed on entire length of the file data.
- If the file size is greater than (>) 1 MB, then the program takes the first 1 MB, the last 1 MB of the file and the file size is all combined together and a hash is taken on it. This way, we can be sure if two files are absolutely clones without performing a checksum on the whole length of the file


### Some considerations
The program scans and outputs identical files based on best effort basis. This means that not all files it reports on can be deemed as 'Absolutely identical'. So, the key term here is "Possibly identical". This tool can be used when you want to do a quick analysis to see which files are POSSIBLY identical. This tool must not be used in critical places and business solutions, and must not be considered as the source of truth to delete any of those found identical files. At most, the tool reports incorrect identical files or skips some of the files which are not accessible due to file permission restrictions. Make sure you do some test runs on some sample files of different file types first.


### Regarding files with 0 bytes size
If you are running clonehunter on bunch of different files or file types, let's say some mp4, pdf, txt etc but they all have file sizes of 0 bytes, and if you used the -c checksum option, you will observe all of the 0 size files grouped together as duplicates in the final output on the screen.

## Example usage of the `delete` command with options:
```
clonehunter delete -i ../some.json --dry-run
```
The delete command does not have many options.

`-i` take the input json report file which you have generated with the `hunt` command and options `-u json -f report-name.json`
`--dry-run` lets you test the deletion algorithm without really deleting anything


[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/clonehunter.svg
[crate-link]: https://crates.io/crates/clonehunter
[license-image]: https://img.shields.io/badge/License-MIT_or_Apache_2.0-yellow.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.75+-blue.svg
[downloads-image]: https://img.shields.io/crates/d/clonehunter.svg
[category-image]: https://img.shields.io/badge/category-Duplicate_Files_Finder-darkred.svg


[cargo-install]: https://doc.rust-lang.org/cargo/getting-started/installation.html