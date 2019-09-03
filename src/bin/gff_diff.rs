#[macro_use]
extern crate serde_json;
extern crate bio;
extern crate getopts;
extern crate gff_diff;

use getopts::Options;
use regex::Regex;
use serde_json::value::Value;
use std::collections::HashMap;
use std::env;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn diff_files(file1: String, file2: String) -> Value {
    let mut data1 = HashMap::new();
    gff_diff::read_gff_into_data(file1, &mut data1);
    let mut data2 = HashMap::new();
    gff_diff::read_gff_into_data(file2, &mut data2);
    let mut result = json!( {
        "changes" :[]
    });
    gff_diff::compare_gff(&data1, &data2, 0, &mut result);
    gff_diff::compare_gff(&data2, &data1, 1, &mut result);
    result
    //        println!("{:#}", stringify_pretty(result, 2));
}

fn apollo_diff(full_gff: String, apollo_gff: String) -> Value {
    let mut full = HashMap::new();
    gff_diff::read_gff_into_data(full_gff, &mut full);
    let mut apollo = HashMap::new();
    gff_diff::read_gff_into_data(apollo_gff, &mut apollo);

    let result = json! ({
        "changes" :[]
    });

    let re = Regex::new(r"-\d+$").unwrap();

    for (id, apollo_row) in &apollo {
        let attrs = apollo_row.attributes();
        if attrs.contains_key("Parent") {
            let parent = attrs["Parent"].clone();
            if !apollo.contains_key(&parent) {
                panic!("Parent {} of {} not in Apollo dataset!", &parent, &id);
            }
            let parent_row = apollo.get(&parent).unwrap();
            if !parent_row.attributes().contains_key("Name") {
                panic!("Parent {} of {} has no 'Name' attribute", &parent, &id);
            }
            let parent_id = parent_row.attributes().get("Name").unwrap();
            let parent_id = re.replace(parent_id, "");
            println!("Original parent ID of {} is {}", &id, &parent_id);
        } else {
            let row_id = attrs.get("Name").unwrap();
            let row_id = re.replace(row_id, "");
            println!("Top-level row {} is {}", &id, &row_id);
        }
    }

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
                apollo_diff(files[0].clone(), apollo_file);
            }
            None => print_usage(&program, opts),
        },
        2 => {
            let diff = diff_files(files[0].clone(), files[1].clone());
            println!("{:#}", diff);
        }
        _ => print_usage(&program, opts),
    }
}
