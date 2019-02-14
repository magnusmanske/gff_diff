# GFF_DIFF
A tool to generate a diff (in JSON format) between two GFF3 files, written in Rust.

## Install rust
See [instructions](https://www.rust-lang.org/learn/get-started), or just run:
```
curl https://sh.rustup.rs -sSf | sh
```

## Compile the tool
Use `git clone https://github.com/sanger-pathogens/gff_diff.git` to get the tool source, then use
```
cargo build --release
```
to build the binary (`target/release/gff_diff`).

## Usage
To compare `original.gff` and `modified.gff`, use:
```
gff_diff original.gff modified.gff
```

## Output format
Output is a JSON structure. The changes required to turn `original.gff` into `modified.gff` are in the objects in the `{"changes":[]}` array.
Each object has an `action`, a `what`, and an `id` key. `what` can be `row` (a line in the GFF file, represented by an `id`) or `attribute` (last column in a `row`).

`action` can be:
* `add` / `remove` for `what=attribute`
* `add` / `remove` / `update` for `what=row`

For `what=row` / `action=update`, there are `key` and `value` keys, indicating what should be changed. `key` can be one of `seqname`, `source`, `feature_type`, `start` , `end`, `score`, `strand`, or `frame`. `value` is a string representing the new value for the given key.

For `what=row` / `action=add/remove`, a `data` key holds a JSON structure representing the entire row to be added or removed.

For `what=attribute` /  `action=add/remove`, there are `key` and `value` keys, indicating what value should be added to, or removed from, the attribute key.
