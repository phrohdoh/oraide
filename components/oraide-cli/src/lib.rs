use std::{
    env,
    path::{
        PathBuf,
    },
};

mod commands;
use commands::{
    Parse,
};

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
            let file_paths = args.into_iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>();

            let file_count = file_paths.len();

            let parse = Parse::new(file_paths)
                .expect("Failed to setup parsing");

            let start = std::time::Instant::now();
            parse.run();
            println!("[info] took {:?} to parse {} file(s)", start.elapsed(), file_count);
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