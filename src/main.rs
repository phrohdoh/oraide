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
    Parser,
};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let file_path = env::args().nth(1).expect("Please provide a file path");
    let mut f = std::fs::File::open(&file_path).expect("Failed to open provided file path");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("Failed to read provided file path");

    let mut files = Files::new();
    let file_id = files.add(file_path, s);

    // === lexer

    let file = &files[file_id];
    let mut lexer = Lexer::new(file);
    let tokens = lexer.by_ref().collect::<Vec<_>>();
    log::debug!("Lexed {} token(s)", tokens.len());

    let lexer_diags = lexer.take_diagnostics();

    if !lexer_diags.is_empty() {
        let writer = StandardStream::stdout(ColorArg::from_str("auto").unwrap().into());

        for diag in &lexer_diags {
            language_reporting::emit(
                &mut writer.lock(),
                &files,
                diag,
                &language_reporting::DefaultConfig,
            ).unwrap();

            println!();
        }
    }

    // === parser

    let mut parser = Parser::new(file_id, tokens.into_iter());
    let nodes = parser.by_ref().collect::<Vec<_>>();
    log::debug!("Parsed {} node(s)", nodes.len());

    let parser_diags = parser.take_diagnostics();
    if !parser_diags.is_empty() {
        let writer = StandardStream::stdout(ColorArg::from_str("auto").unwrap().into());

        for diag in &parser_diags {
            language_reporting::emit(
                &mut writer.lock(),
                &files,
                diag,
                &language_reporting::DefaultConfig,
            ).unwrap();

            println!();
        }
    }

    Ok(())
}