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

    let lexer = MiniYamlLexer::new_from_str(&miniyaml_text);
    let comp_lines = lexer.lex();

    for comp_line in comp_lines {
        let opt_indent_txt = comp_line.indent.map(|span| &miniyaml_text[span]);
        let opt_key_txt = comp_line.key.map(|span| &miniyaml_text[span]);
        let opt_key_sep_txt = comp_line.key_sep.map(|span| &miniyaml_text[span]);
        let opt_value_txt = comp_line.value.map(|span| &miniyaml_text[span]);
        let opt_comment_txt = comp_line.comment.map(|span| &miniyaml_text[span]);
        let opt_term_txt = comp_line.term.map(|span| &miniyaml_text[span]);

        println!("indent  = {:?}", opt_indent_txt);
        println!("key     = {:?}", opt_key_txt);
        println!("key_sep = {:?}", opt_key_sep_txt);
        println!("value   = {:?}", opt_value_txt);
        println!("comment = {:?}", opt_comment_txt);
        println!("term    = {:?}", opt_term_txt);
        println!();
    }
}
