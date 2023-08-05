use std::fs;
use std::io::{self, BufRead, BufReader, Read};
use std::path::PathBuf;
use structopt::StructOpt;

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

fn get_reader<R>(file_name: &PathBuf) -> io::Result<Box<dyn BufRead>> {
    match file_name.to_str() {
        None | Some("-") => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => match fs::File::open(file_name) {
            Ok(fh) => Ok(Box::new(BufReader::new(fh))),
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
        //let mut buf = Vec::new();
        /*
        if let Ok(mut reader) = get_reader(&file) {
            reader.read_to_end(&mut buf);
            print!("{:?}", buf);
        } else {
            eprintln!("Cannot open file {:?}", file);
        }*/
    }
}
