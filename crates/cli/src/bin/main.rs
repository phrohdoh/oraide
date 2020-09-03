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
                .replace("\t", "→") // ␉
                ;

            let fg = ansi_term::Color::Blue;

            let styled = ansi_term::Style::new()
                .fg(fg)
                .paint(txt);

            print!("{}", styled);
        }

        if let Some(key_txt) = opt_key_txt {
            let fg = ansi_term::Color::Green;

            let styleds = {
                let mut ret = vec![];

                match (key_txt.starts_with('-'), key_txt.rfind('@')) {
                    (true, Some(last_at_idx)) => {
                        let rm_styled = ansi_term::Style::new().fg(ansi_term::Color::Red).paint("-");
                        let rest_styled = ansi_term::Style::new().fg(fg).paint(&key_txt[1..last_at_idx]);
                        let suffix_styled = ansi_term::Style::new().fg(ansi_term::Color::Yellow).paint(&key_txt[last_at_idx..]);

                        ret.push(rm_styled);
                        ret.push(rest_styled);
                        ret.push(suffix_styled);
                    },
                    (true, None) => {
                        let rm_styled = ansi_term::Style::new().fg(ansi_term::Color::Red).paint("-");
                        let rest_styled = ansi_term::Style::new().fg(fg).paint(&key_txt[1..]);

                        ret.push(rm_styled);
                        ret.push(rest_styled);
                    },
                    (false, Some(last_at_idx)) => {
                        let suffix_styled = ansi_term::Style::new().fg(ansi_term::Color::Yellow).paint(&key_txt[last_at_idx..]);
                        let rest_styled = ansi_term::Style::new().fg(fg).paint(&key_txt[..last_at_idx]);

                        ret.push(rest_styled);
                        ret.push(suffix_styled);
                    },
                    (false, None) => {
                        let styled = ansi_term::Style::new().fg(fg).paint(key_txt);
                        ret.push(styled);
                    },
                }

                ret
            };

            for styled in styleds {
                print!("{}", styled);
            }
        }

        if let Some(key_sep_txt) = opt_key_sep_txt {
            let fg = ansi_term::Color::RGB(128, 128, 128);

            let styled = ansi_term::Style::new()
                .fg(fg)
                .paint(key_sep_txt);

            print!("{}", styled);
        }

        if let Some(value_txt) = opt_value_txt {
            let fg = ansi_term::Color::RGB(192, 192, 224);

            let styled = ansi_term::Style::new()
                .fg(fg)
                .paint(value_txt);

            print!("{}", styled);
        }

        if let Some(comment_txt) = opt_comment_txt {
            let fg = ansi_term::Color::RGB(96, 96, 96);

            let styled = ansi_term::Style::new()
                .fg(fg)
                .paint(comment_txt);

            print!("{}", styled);
        }

        // /*
        if let Some(term_txt) = opt_term_txt {
            let txt = term_txt
                .replace("\r\n", "␍␊")
                .replace("\n", "␊")
                .replace("\r", "␍") ;

            let fg = ansi_term::Color::Red;

            let styled = ansi_term::Style::new()
                .fg(fg)
                .paint(txt);

            print!("{}", styled);
        }
        // */

        println!();
    }

    Ok(())
}
