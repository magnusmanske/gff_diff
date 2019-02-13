extern crate bio;
#[macro_use]
extern crate json;

use json::stringify_pretty;
use std::collections::HashMap;
use std::env;
mod gff_diff;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} old_gff_file new_gff_file\n", args[0]);
        return;
    }

    let mut data1 = HashMap::new();
    gff_diff::read_gff_into_data(args[1].clone(), &mut data1);
    let mut data2 = HashMap::new();
    gff_diff::read_gff_into_data(args[2].clone(), &mut data2);
    let mut result = object! {
        "changes" => array![]
    };
    gff_diff::compare_gff(&data1, &data2, 0, &mut result);
    gff_diff::compare_gff(&data2, &data1, 1, &mut result);
    println!("{:#}", stringify_pretty(result, 2));
}
