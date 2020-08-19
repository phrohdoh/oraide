// oraide - tools for OpenRA-based mod/game development
// get the source code at https://github.com/Phrohdoh/oraide
//
// copyright (c)
// - 2020 Taryn "Phrohdoh" Hill

//! command-line parsing for the `ora` cli component of `oraide`

use {
    std::path::PathBuf,
    anyhow::{bail, Result},
    pico_args::Arguments,
};

pub(crate) struct Args {
    pub(crate) command: Command,
}

pub(crate) enum Command {
    Help,
    LexFile(PathBuf),
}

impl Args {
    pub(crate) fn parse() -> Result<Self> {
        let mut matches = Arguments::from_env();

        let is_user_requesting_help = matches.contains(["-h", "--help"]);

        let help = Ok(Self {
            command: Command::Help,
        });

        let cmd = match matches.subcommand()? {
            Some(it) => it,
            _ => {
                print_usage();
                matches.finish()?;
                return help;
            },
        };

        let command = match cmd.as_str() {
            "lex" => {
                if is_user_requesting_help {
                    eprintln!("\
ora lex

USAGE:
    ora lex <file-path-to-lex> [FLAGS]

FLAGS:
    -h, --help        prints help information"
                    );

                    return help;
                }

                let file_path = {
                    let mut trailing = matches.free()?;
                    if trailing.len() != 1 {
                        bail!("must provide a single file-path");
                    }

                    trailing.pop().unwrap().into()
                };

                Command::LexFile(file_path)
            },
            other => bail!("command {:?} not supported", other),
        };

        Ok(Args { command })
    }
}

fn print_usage() {
    eprintln!("\
ora

USAGE:
    ora <one of COMMANDS>

FLAGS:
    -h, --help        prints help information

COMMANDS:
    lex"
    );
}