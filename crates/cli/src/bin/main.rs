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
    codespan_reporting::{
        term::{
            self,
            ColorArg,
            termcolor::StandardStream,
        },
    },
    oraide_cli::Result,
    oraide_miniyaml::{
        lex_miniyaml_document,
        AbsByteIdxSpan
    },
};

type File = codespan_reporting::files::SimpleFile<String, String>;
type Reportable = codespan_reporting::diagnostic::Diagnostic<()>;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        process::exit(101);
    }
}

fn try_main() -> Result<()> {
    let args = args::Args::parse()?;

    if let args::Command::Help = args.command {
        // already handled in args.rs
        return Ok(());
    }

    let ora_report = match args.command {
        args::Command::CheckSingleFile(path) => _check_single_file(&path),
        _ => unreachable!(),
    }?;

    let report = ora_report_to_report(ora_report);

    let writer = StandardStream::stderr(
        "auto".parse::<ColorArg>().unwrap().into()
    );

    let config = codespan_reporting::term::Config::default();

    for diag in report.1 {
        term::emit(&mut writer.lock(), &config, &report.0, &diag)?;
    }

    Ok(())
}

type FileReport<R> = (File, Vec<R>);
type OraReport = FileReport<Diagnostic>;
type Report = FileReport<Reportable>;

fn ora_report_to_report(
    (file, diags): OraReport,
) -> Report {
    let mut ret_reportables = vec![];

    for diag in diags {
        let reportable = match diag {
            Diagnostic::CrOnlyLineTermNotPermitted {
                line_num,
                term_span,
            } => {
                Reportable::error()
                    .with_message("'\\r'-only line-terminator is not permitted")
                    .with_notes(vec![
                        format!(
                            "{file_name}:{line_num} {span:?}",
                            file_name = file.name(),
                            line_num = line_num,
                            span = term_span,
                        ),
                    ])
            },
        };

        ret_reportables.push(reportable);
    }

    (file, ret_reportables)
}

/// `oraide`-specific diagnostics
enum Diagnostic {
    CrOnlyLineTermNotPermitted {
        line_num: u32,
        term_span: AbsByteIdxSpan,
    },
}

fn _check_single_file(
    path: &Path,
) -> Result<OraReport> {
    let file_contents = fs::read_to_string(path)?;

    let file = File::new(
        path.to_string_lossy().into_owned(),
        file_contents.clone(),
    );

    let mut ret_diags = vec![];

    let lines = lex_miniyaml_document(&file_contents);

    let map_opt_span_to_txt = |opt_span: Option<AbsByteIdxSpan>| -> Option<&str> {
        opt_span.map(|span| &file_contents[span])
    };

    let line_iter = lines.into_iter().enumerate().map(|(line_idx, line)| (
        line_idx as u32 + 1,
        line,
    ));

    for (line_num, line) in line_iter {
        if let Some("\r") = map_opt_span_to_txt(line.term) {
            ret_diags.push(Diagnostic::CrOnlyLineTermNotPermitted {
                line_num,
                term_span: line.term.unwrap(),
            });
        }
    }

    let report = (
        file,
        ret_diags,
    );

    Ok(report)
}
