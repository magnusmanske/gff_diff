extern crate bio;
extern crate getopts;
extern crate gff_diff;
#[macro_use]
extern crate json;

use getopts::Options;
use json::stringify_pretty;
use std::collections::HashMap;
use std::env;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn diff_files(file1: String, file2: String) -> json::JsonValue {
    let mut data1 = HashMap::new();
    gff_diff::read_gff_into_data(file1, &mut data1);
    let mut data2 = HashMap::new();
    gff_diff::read_gff_into_data(file2, &mut data2);
    let mut result = object! {
        "changes" => array![]
    };
    gff_diff::compare_gff(&data1, &data2, 0, &mut result);
    gff_diff::compare_gff(&data2, &data1, 1, &mut result);
    result
    //        println!("{:#}", stringify_pretty(result, 2));
}

fn apollo_diff(_full_gff: String, _apollo_gff: String) -> json::JsonValue {
    let result = object! {
        "changes" => array![]
    };
    result
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("a", "apollo", "apollo output GFF", "APOLLO");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let apollo = matches.opt_str("a");
    let files: Vec<String> = matches.free;

    match files.len() {
        1 => match apollo {
            Some(apollo_file) => {
                dbg!(apollo_diff(files[0].clone(), apollo_file));
            }
            None => print_usage(&program, opts),
        },
        2 => {
            let diff = diff_files(files[0].clone(), files[1].clone());
            println!("{:#}", stringify_pretty(diff, 2));
        }
        _ => print_usage(&program, opts),
    }
}
