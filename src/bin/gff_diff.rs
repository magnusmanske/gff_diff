extern crate getopts;
extern crate gff_diff;

use getopts::Options;
use gff_diff::CompareGFF;
use std::env;
use std::io::{self};

fn get_usage(program: &str, opts: Options) -> String {
    let brief = format!("Usage: {} [options] FILE FILE2", program);
    format!("{}", opts.usage(&brief))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("a", "apollo", "Second file is Apollo-style GFF");
    opts.optflag("d", "diff", "output diff");
    opts.optflag("x", "apply", "apply diff");
    opts.optflag("i", "issues", "record issues");
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
    let do_record_issues = matches.opt_present("i");
    let do_apply = matches.opt_present("x");
    let apollo = matches.opt_present("a");
    let files: Vec<String> = matches.free;

    if files.len() != 2 {
        println!("{}", get_usage(&program, opts));
        return;
    }

    let mut cg = CompareGFF::new_from_files(&files[0], &files[1]).unwrap();
    cg.record_issues(do_record_issues);
    let diff = match apollo {
        true => cg.diff_apollo(),
        false => cg.diff(),
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
