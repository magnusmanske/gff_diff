extern crate getopts;
extern crate gff_diff;

use getopts::Options;
use gff_diff::CompareGFF;
use std::env;
use std::io::{self};

fn get_usage(program: &str, opts: Options) -> String {
    let brief = format!("Usage: {} FILE [options]", program);
    format!("{}", opts.usage(&brief))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("a", "apollo", "apollo output GFF", "APOLLO");
    opts.optflag("d", "diff", "output diff");
    opts.optflag("x", "apply", "apply diff");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        println!("{}", get_usage(&program, opts));
        return;
    }
    let do_diff = matches.opt_present("d");
    let do_apply = matches.opt_present("x");
    let apollo = matches.opt_str("a");
    let files: Vec<String> = matches.free;

    let mut cg = CompareGFF::new();
    let diff = match files.len() {
        1 => match apollo {
            Some(apollo_file) => {
                cg = CompareGFF::new_from_files(&files[0], &apollo_file).unwrap();
                cg.diff_apollo()
            }
            None => Err(From::from(get_usage(&program, opts))),
        },
        2 => {
            cg = CompareGFF::new_from_files(&files[0], &files[1]).unwrap();
            cg.diff()
        }
        _ => Err(From::from(get_usage(&program, opts))),
    };
    match diff {
        Ok(diff) => match (do_diff, do_apply) {
            (true, false) | (false, false) => {
                println!("{:#}", diff);
            }
            (false, true) => {
                match cg.apply_diff(&diff) {
                    Ok(_) => {
                        cg.write_data1(Box::new(io::stdout())).unwrap();
                    }
                    Err(e) => {
                        println!("{}", e);
                    }
                };
            }
            (true, true) => println!("--diff and --apply are both set, abort"),
        },
        Err(e) => println!("{}", e),
    }
}
