//! Example file that the current Rust grammars cannot fully parse.
//! Used to test the --deny-parse-errors flag.

fn function_with_parse_errors() {
    match 123 {
        ..0 => (),
    }
}
