// oraide - tools for OpenRA-based mod/game development
// get the source code at https://github.com/Phrohdoh/oraide
//
// copyright (c)
// - 2020 Taryn "Phrohdoh" Hill

use {
    std::{
        env,
        io::Read,
        fs::File,
    },
    oraide::*,
};

fn main() {
    let mut args = env::args().skip(1);
    let miniyaml_file_path = match args.next() {
        Some(path) => path,
        _ => {
            eprintln!("error: please provide a file path for a MiniYaml file");
            std::process::exit(1);
        },
    };

    let mut miniyaml_file_handle = match File::open(&miniyaml_file_path) {
        Ok(hndl) => hndl,
        Err(e) => {
            eprintln!("error: unable to open '{}'", miniyaml_file_path);
            eprintln!("       because: {}", e.to_string());
            std::process::exit(2);
        },
    };

    let mut miniyaml_text = String::new();
    if let Err(e) = miniyaml_file_handle.read_to_string(&mut miniyaml_text) {
        eprintln!("error: unable to read '{}'", miniyaml_file_path);
        eprintln!("       because: {}", e.to_string());
        std::process::exit(3);
    }

    let lines = lex_miniyaml_document(&miniyaml_text);

    let map_opt_span_to_txt = |opt_span: Option<AbsByteIdxSpan>| -> Option<&str> {
        opt_span.map(|span| &miniyaml_text[span])
    };

    for line in lines {
        let raw_txt = &miniyaml_text[line.raw];
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
}
