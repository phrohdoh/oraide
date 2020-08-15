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

    // print!("[dbg] linting {} ... ", miniyaml_file_path);

    let mut codespan_files = codespan_reporting::files::SimpleFiles::new();
    let diag_file_id = codespan_files.add(&miniyaml_file_path, &miniyaml_text);

    let lexer = MiniYamlLexer::new_from_str(&miniyaml_text);
    let detailed_comp_lines = lexer.lex();

    let mut diagnostics = vec![];

    for (line_idx, (line_start_abs_bx, comp_line)) in detailed_comp_lines.into_iter().enumerate() {
        // let opt_indent_txt = comp_line.indent.map(|span| &miniyaml_text[span]);
        // let _opt_key_txt = comp_line.key.map(|span| &miniyaml_text[span]);
        // let _opt_key_sep_txt = comp_line.key_sep.map(|span| &miniyaml_text[span]);
        // let _opt_value_txt = comp_line.value.map(|span| &miniyaml_text[span]);
        // let _opt_comment_txt = comp_line.comment.map(|span| &miniyaml_text[span]);
        // let _opt_term_txt = comp_line.term.map(|span| &miniyaml_text[span]);

        // ----- helpers -------------------------------------------------------
        let _diag_note_file_path_and_line_num_and_col_nums = |span: ByteIdxSpan| -> String {
            let line_start_abs_idx = line_start_abs_bx.as_inner();
            let (span_start_abs_idx, _span_end_abs_idx) = Into::<(_, _)>::into(span);
            let col_num = span_start_abs_idx - line_start_abs_idx + 1;

            // let display_file_path = ansi_term::Style::new().italic().paint(&miniyaml_file_path);
            let display_file_path = ansi_term::Color::Blue.paint(&miniyaml_file_path);

            format!(
                "{file_path}:{line_num}:{col_num}",
                file_path = display_file_path,
                line_num = line_idx + 1,
                col_num = col_num,
            )
        };

        let _diag_note_diag_code_explanation_url = |code: &'_ str| -> String {
            let url = format!("https://github.com/Phrohdoh/oraide/wiki/diagnostic-explanations#{}", code);
            let display_url = ansi_term::Style::new().underline().paint(url);

            format!(
                "read an explanation of this diagnostic at {url}",
                url = display_url,
            )
        };
        // - end helpers -------------------------------------------------------

        if let Some(indent_span) = comp_line.indent {
            let indent_txt = &miniyaml_text[indent_span];
            let opt_first_tab_rel_idx = indent_txt.find('\t');
            let opt_first_space_rel_idx = indent_txt.find(' ');

            // check for mixed indentation
            match (opt_first_tab_rel_idx, opt_first_space_rel_idx) {
                (Some(_first_tab_rel_idx), Some(_first_space_rel_idx)) => {
                    // let first_offender_rel_idx = first_space_rel_idx.max(first_tab_rel_idx);
                    // let first_offender_display_type = if first_space_rel_idx < first_tab_rel_idx { "tab" } else { "space" };
                    // eprintln!(
                    //     "[err] {file_path}:{line_num}:{col_num}: indentation must be all spaces or all tabs, not a combination of both, found offending <{offender_display_ty}> in {indent_txt}",
                    //     file_path = miniyaml_file_path,
                    //     line_num = line_idx + 1,
                    //     col_num = first_offender_rel_idx + 1,
                    //     offender_display_ty = first_offender_display_type,
                    //     indent_txt = indent_txt,
                    // );

                    let diag = codespan_reporting::diagnostic::Diagnostic::error()
                        .with_message("indentation must be all spaces or all tabs, not a combination of both")
                        .with_code("syn0001")
                        .with_labels(vec![
                            codespan_reporting::diagnostic::Label::primary(diag_file_id, indent_span), ])
                        .with_notes(vec![
                            format!("{:?}", indent_txt.replace(" ", "␣")),
                            // _diag_note_diag_code_explanation_url("syn0001"),
                        ]);

                    diagnostics.push(diag);

                    // eprintln!(
                    //     "error at {file_path}:{line_num}: indentation must all be spaces or all tabs, not mixed; found {indent_txt:#?}",
                    //     file_path = miniyaml_file_path,
                    //     line_num = line_idx + 1,
                    //     indent_txt = indent_txt,
                    // );
                },
                _ => { /* TODO */ },
            }

            // check for indentation-only lines (ignoring terms)
            if comp_line.key.is_none() && comp_line.key_sep.is_none() && comp_line.value.is_none() && comp_line.comment.is_none() {
                let diag = codespan_reporting::diagnostic::Diagnostic::note()
                    .with_message("whitespace-only line probably not desired")
                    .with_code("syn0003")
                    // .with_labels(vec![
                    //     codespan_reporting::diagnostic::Label::primary(diag_file_id, indent_span), ])
                    .with_notes(vec![
                        _diag_note_file_path_and_line_num_and_col_nums(indent_span),
                        // _diag_note_diag_code_explanation_url("syn0003"),
                        // format!("{:?}", indent_txt.replace(" ", "␣")),
                    ])
                    ;

                diagnostics.push(diag);
            }
        }

        /*
        let _diag_note_file_path_and_line_num = || -> String {
            format!(
                "{file_path}:{line_num}",
                file_path = miniyaml_file_path,
                line_num = line_idx + 1,
            )
        };
        */

        if let Some(term_span) = comp_line.term {
            let term_txt = &miniyaml_text[term_span];
            if term_txt == "\r" {
                let diag = codespan_reporting::diagnostic::Diagnostic::error()
                    .with_message("'\\r' (CR) alone is not a valid line-terminator sequence, please use '\\r\\n' (CRLF) or '\\n' (LF) exclusively")
                    .with_code("syn0002")
                    .with_notes(vec![
                        _diag_note_file_path_and_line_num_and_col_nums(term_span),
                        // format!(
                        //     "{file_path}:{line_num}:{col_num}",
                        //     file_path = miniyaml_file_path,
                        //     line_num = line_idx + 1,
                        //     col_num = {
                        //         let line_start_abs_idx = line_start_abs_bx.as_inner();
                        //         let term_start_abs_idx = Into::<(_, _)>::into(term_span).0;
                        //         (term_start_abs_idx - line_start_abs_idx) + 1 // col idx -> col num
                        //     },
                        // ),
                        format!(
                            "if using VSCode, see {url}",
                            url = "https://qvault.io/2020/06/18/how-to-get-consistent-line-breaks-in-vs-code-lf-vs-crlf",
                        ),
                        // _diag_note_diag_code_explanation_url("syn0002"),
                    ])
                    // .with_labels(vec![
                    //     codespan_reporting::diagnostic::Label::primary(diag_file_id, term_span),
                    // ])
                    ;

                diagnostics.push(diag);
            }
        }

        // println!("indent  = {:?}", opt_indent_txt);
        // println!("key     = {:?}", opt_key_txt);
        // println!("key_sep = {:?}", opt_key_sep_txt);
        // println!("value   = {:?}", opt_value_txt);
        // println!("comment = {:?}", opt_comment_txt);
        // println!("term    = {:?}", opt_term_txt);
        // println!();
    }

    // println!("done");

    let writer = codespan_reporting::term::termcolor::StandardStream::stderr(codespan_reporting::term::termcolor::ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    for diag in diagnostics {
        codespan_reporting::term::emit(&mut writer.lock(), &config, &codespan_files, &diag).unwrap();
    }
}
