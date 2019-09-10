// This file is part of oraide.  See <https://github.com/Phrohdoh/oraide>.
// 
// oraide is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3
// as published by the Free Software Foundation.
// 
// oraide is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with oraide.  If not, see <https://www.gnu.org/licenses/>.

// invoke like so:
// cargo run -- __novcs/medium.yaml

use std::error::Error;
use std::str::FromStr;
use std::env;
use std::io::Read;

use mltt_span::{
    Span,
    Files,
    FileId,
};

use language_reporting::{
    Diagnostic,
    ColorArg,
    termcolor::{
        StandardStream,
        StandardStreamLock,
    },
};

use oraide_miniyaml::{
    Lexer,
    Parser,
    Arborist,
};

fn write_diags<'files>(
    mut w: &mut StandardStreamLock,
    files: &'files Files,
    diags: Vec<Diagnostic<Span<FileId>>>,
) {
    for diag in &diags {
        language_reporting::emit(
            &mut w,
            files,
            diag,
            &language_reporting::DefaultConfig
        ).unwrap();

        println!();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let mut args = env::args();
    args.next();

    let file_path = args.next().expect("Please provide a file path");
    let mut f = std::fs::File::open(&file_path).expect("Failed to open provided file path");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("Failed to read provided file path");

    let mut files = Files::new();
    let file_id = files.add(file_path, s);

    let writer = StandardStream::stdout(ColorArg::from_str("auto").unwrap().into());

    // === lexer

    let file = &files[file_id];
    let mut lexer = Lexer::new(file);
    let tokens = lexer.by_ref().collect::<Vec<_>>();
    log::debug!("Lexed {} token(s)", tokens.len());

    let lexer_diags = lexer.take_diagnostics();

    if !lexer_diags.is_empty() {
        write_diags(
            &mut writer.lock(),
            &files,
            lexer_diags
        );
    }

    // === parser

    let mut parser = Parser::new(tokens.into_iter());
    let nodes = parser.by_ref().collect::<Vec<_>>();
    log::debug!("Parsed {} node(s)", nodes.len());

    let parser_diags = parser.take_diagnostics();
    if !parser_diags.is_empty() {
        write_diags(
            &mut writer.lock(),
            &files,
            parser_diags
        );
    }

    // === arborist

    let mut arborist = Arborist::new(nodes.into_iter());
    let tree = arborist.build_tree();

    let arborist_diags = arborist.take_diagnostics();
    if !arborist_diags.is_empty() {
        write_diags(
            &mut writer.lock(),
            &files,
            arborist_diags
        );
    }

    let top_level_slices = tree.top_level_nodes()
        .filter_map(|(_nid, shrd_node)| shrd_node.key_slice(&files))
        .collect::<Vec<_>>();

    println!("{:?}", top_level_slices);

    Ok(())
}