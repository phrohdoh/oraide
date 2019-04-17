use std::{
    env,
    path::{
        PathBuf,
    },
};

mod commands;

// NOTE: This looks an awful lot like a binary package, but is indeed a library.
//       This will be invoked by a top-level bin target (the overarching `oraide` crate),
//       see `<root>/src/main.rs` for more context.
pub fn main() {
    let mut args = env::args().skip(1);

    let cmd = match args.next() {
        Some(cmd) => cmd,
        _ => {
            eprintln!("Please provide a command");
            return;
        },
    };

    match cmd.as_ref() {
        "parse" => {
            match args.next() {
                Some(file_path) => {
                    let file_path = PathBuf::from(file_path);
                    let parse = commands::Parse::new(file_path).expect("Failed to setup parsing");
                    let start = std::time::Instant::now();
                    parse.run();
                    println!("[info] took {:?}", start.elapsed());
                },
                _ => eprintln!("Please provide a file path to parse"),
            }
        },
        "lint" => {
            match args.next() {
                Some(_file_path) => {
                    unimplemented!("linting")
                },
                _ => eprintln!("Please provide a file path to lint"),
            }
        },
        _ => eprintln!("Please provide a command"),
    }
}