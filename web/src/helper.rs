use super::*;
use bincode::deserialize_from;
use std::fs::File;
use std::io::BufReader;

pub fn get_filename() -> String {
    // check if arguments are right
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} pbf-file", args[0]);
        std::process::exit(1);
    }

    // get filename
    return std::env::args_os().nth(1).unwrap().into_string().unwrap();
}

pub fn read_from_disk(filename: String) -> FmiFile {
    let mut reader = BufReader::new(File::open(filename).unwrap());
    return deserialize_from(&mut reader).unwrap();
}
