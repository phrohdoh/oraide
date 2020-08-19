// oraide - tools for OpenRA-based mod/game development
// get the source code at https://github.com/Phrohdoh/oraide
//
// copyright (c)
// - 2020 Taryn "Phrohdoh" Hill

mod args;

use {
    std::{
        fs,
        process,
        path::Path,
    },
    oraide_cli::Result,
    oraide_miniyaml::{
        lex_miniyaml_document,
        AbsByteIdxSpan
    },
};

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        process::exit(101);
    }
}

fn try_main() -> Result<()> {
    let args = args::Args::parse()?;

    match args.command {
        args::Command::Help => /* handled in args.rs */ Ok(()),
        args::Command::LexFile(path) => _lex_file(&path),
    }
}

fn _lex_file(
    path: &Path,
) -> Result<()> {
    let file_contents = fs::read_to_string(path)?;
    let lines = lex_miniyaml_document(&file_contents);

    let map_opt_span_to_txt = |opt_span: Option<AbsByteIdxSpan>| -> Option<&str> {
        opt_span.map(|span| &file_contents[span])
    };

    for line in lines {
        let raw_txt = &file_contents[line.raw];
        let opt_indent_txt = map_opt_span_to_txt(line.indent);
        let opt_key_txt = map_opt_span_to_txt(line.key);
        let opt_key_sep_txt = map_opt_span_to_txt(line.key_sep);
        let opt_value_txt = map_opt_span_to_txt(line.value);
        let opt_comment_txt = map_opt_span_to_txt(line.comment);
        let opt_term_txt = map_opt_span_to_txt(line.term);

        println!("raw     = {:?}", raw_txt);
        println!("indent  = {:?}", opt_indent_txt);
        println!("key     = {:?}", opt_key_txt);
        println!("key_sep = {:?}", opt_key_sep_txt);
        println!("value   = {:?}", opt_value_txt);
        println!("comment = {:?}", opt_comment_txt);
        println!("term    = {:?}", opt_term_txt);
        println!();
    }

    Ok(())
}
