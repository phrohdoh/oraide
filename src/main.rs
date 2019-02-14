use mltt_span::{Files};

use oraml2::{
    Lexer,
};

fn main() {
    let mut files = Files::new();
    let file_id = files.add("test", "^BasePlayer:\n\tAlwaysVisible: # some comment\n\t\tDoTheThing: true\n");

    let lexer = Lexer::new(&files[file_id]);
}