use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use structopt::StructOpt;

use crate::md5::{MD5Machine, MD5Reader};

pub mod md5;

#[derive(StructOpt, Debug)]
#[structopt(name = "rustmd5")]
struct Opt {
    // A flag, true if used in the command line. Note doc comment will
    // be used for the help message of the flag. The name of the
    // argument will be, by default, based on the name of the field.
    /// Activate debug mode
    //#[structopt(short, long)]
    //binary: bool,

    /// Input Files
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

const BUFFERED_READER_CAP: usize = 32768; // 32kB

fn get_reader(file_name: &PathBuf) -> io::Result<Box<dyn BufRead>> {
    match file_name.to_str() {
        //    let reader = BufReader::with_capacity(10, f);
        None | Some("-") => Ok(Box::new(BufReader::with_capacity(
            BUFFERED_READER_CAP,
            io::stdin(),
        ))),
        _ => match fs::File::open(file_name) {
            Ok(fh) => Ok(Box::new(BufReader::with_capacity(BUFFERED_READER_CAP, fh))),
            Err(e) => Err(e),
        },
    }
}

fn main() {
    let opt = Opt::from_args();
    //println!("{:#?}", opt);

    let mut files = opt.files;

    if files.len() == 0 {
        files.push(PathBuf::from("-"));
    }

    for file in files {
        //let mut buf: [u8; 1] = [0];

        if let Ok(mut reader) = get_reader(&file) {
            let mut machine = MD5Machine::new(MD5Reader::new(&mut reader));
            let sum = machine.sum();
            for b in sum {
                print!("{:02x}", b)
            }
            println!("  {}", file.to_str().unwrap());
        } else {
            eprintln!("Cannot open file {:?}", file);
        }
    }
}
