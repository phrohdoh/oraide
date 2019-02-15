use std::error::Error;
use std::str::FromStr;
use std::env;
use std::io::Read;

use mltt_span::{Files};
use language_reporting::{
    ColorArg,
    termcolor::{
        StandardStream,
    },
};

use oraml::{
    Lexer,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Change the level to `Debug` to see the log messages.
    simple_logger::init_with_level(log::Level::Error)?;

    let file_path = env::args().nth(1).expect("Please provide a file path");
    let mut f = std::fs::File::open(&file_path).expect("Failed to open provided file path");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("Failed to read provided file path");

    let mut files = Files::new();
    let file_id = files.add(file_path, s);

    let mut lexer = Lexer::new(&files[file_id]);
    let tokens = lexer.by_ref().collect::<Vec<_>>();
    println!("Lexed {} token(s)", tokens.len());

    let diags = lexer.take_diagnostics();

    if diags.is_empty() {
        println!("Everything is a-ok!");
    } else {
        let writer = StandardStream::stdout(ColorArg::from_str("auto").unwrap().into());

        for diag in diags {
            language_reporting::emit(
                &mut writer.lock(),
                &files,
                &diag,
                &language_reporting::DefaultConfig,
            ).unwrap();

            println!();
        }
    }

    Ok(())
}