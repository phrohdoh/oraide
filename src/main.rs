use std::error::Error;

use mltt_span::{Files};
use unindent::unindent;

use oraml2::{
    Lexer,
};

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init()?;

    let mut files = Files::new();
    let file_id = files.add("test", unindent(r##"
        ^BasePlayer: # a
        # b
            AlwaysVisible: # c
            # d
                # e
                DoTheThing: true # f
                    # g
            # h
    "##));

    let lexer = Lexer::new(&files[file_id]);
    let tokens = lexer.collect::<Vec<_>>();

    let s: String = tokens.iter().map(|tok| tok.slice).collect();
    print!("{}", s);

    Ok(())
}