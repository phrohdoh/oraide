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
        span_lines_of,
        AbsByteIdxSpan,
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
        args::Command::CheckSingleFile(path) => _check_single_file(&path),
    }
}

fn _check_single_file(
    path: &Path,
) -> Result<()> {
    let file_contents = fs::read_to_string(path)?;
    let lines = span_lines_of(&file_contents);

    let map_opt_span_to_txt = |opt_span: Option<AbsByteIdxSpan>| -> Option<&str> {
        opt_span.map(|span| &file_contents[span])
    };

    for line in lines {
        let opt_indent_txt = map_opt_span_to_txt(line.indent);
        let opt_key_txt = map_opt_span_to_txt(line.key);
        let opt_key_sep_txt = map_opt_span_to_txt(line.key_sep);
        let opt_value_txt = map_opt_span_to_txt(line.value);
        let opt_comment_txt = map_opt_span_to_txt(line.comment);
        let opt_term_txt = map_opt_span_to_txt(line.term);

        if let Some(indent_txt) = opt_indent_txt {
            let txt = indent_txt
                .replace(" ", "␣")
                .replace("\t", "\\t");

            let fg = ansi_term::Color::Black;
            let bg = ansi_term::Color::Blue;

            let styled = ansi_term::Style::new()
                .fg(fg)
                .on(bg)
                .paint(txt);

            print!("{}", styled);
        }

        if let Some(key_txt) = opt_key_txt {
            let fg = ansi_term::Color::Black;
            let bg = ansi_term::Color::Green;

            let styled = ansi_term::Style::new()
                .fg(fg)
                .on(bg)
                .paint(key_txt);

            print!("{}", styled);
        }

        if let Some(key_sep_txt) = opt_key_sep_txt {
            let fg = ansi_term::Color::Black;
            let bg = ansi_term::Color::Red;

            let styled = ansi_term::Style::new()
                .fg(fg)
                .on(bg)
                .paint(key_sep_txt);

            print!("{}", styled);
        }

        if let Some(value_txt) = opt_value_txt {
            let fg = ansi_term::Color::Black;
            let bg = ansi_term::Color::Purple;

            let styled = ansi_term::Style::new()
                .fg(fg)
                .on(bg)
                .paint(value_txt);

            print!("{}", styled);
        }

        if let Some(comment_txt) = opt_comment_txt {
            let fg = ansi_term::Color::Black;
            let bg = ansi_term::Color::RGB(128, 128, 128);

            let styled = ansi_term::Style::new()
                .fg(fg)
                .on(bg)
                .paint(comment_txt);

            print!("{}", styled);
        }

        if let Some(term_txt) = opt_term_txt {
            let txt = term_txt
                .replace("\r\n", "␍␊")
                .replace("\n", "␊")
                .replace("\r", "␍") ;

            let fg = ansi_term::Color::Black;
            let bg = ansi_term::Color::White;

            let styled = ansi_term::Style::new()
                .fg(fg)
                .on(bg)
                .paint(txt);

            print!("{}", styled);
        }

        println!();
    }

    Ok(())
}
