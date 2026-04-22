//! Example file that the Rust grammars cannot fully parse.
//! Used to test the --deny-parse-errors flag.

// Made up "pub(slighly)" syntax to fail the parse.
// This does not have to contain valid Rust
pub(slighly) fn counter() -> u32 {
    0
}
