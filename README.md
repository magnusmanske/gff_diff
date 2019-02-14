# GFF_DIFF
A tool to generate a diff (in JSON format) between two GFF3 files, written in Rust.

## Install rust
See [instructions](https://www.rust-lang.org/learn/get-started), or just do:
`curl https://sh.rustup.rs -sSf | sh`

## Compile the tool
Use `git clone https://github.com/sanger-pathogens/gff_diff.git` to get the tool source, then use
`cargo build --release`
to build the binary (`target/release/gff_diff`).

## Usage
To compare `original.gff` and `modified.gff`, use:
`gff_diff original.gff modified.gff`
