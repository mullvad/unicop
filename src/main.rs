use std::env;
use std::fs;

use miette::{miette, LabeledSpan, NamedSource, Severity};
use unic_ucd_name::Name;

fn main() {
    for arg in env::args().skip(1) {
        check_file(&arg);
    }
}

fn check_file(arg: &str) {
    let src = fs::read_to_string(arg).unwrap();
    let nsrc = NamedSource::new(arg, src.clone());
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_javascript::language())
        .expect("Error loading JavaScript grammar");
    // parser
    //     .set_language(&tree_sitter_python::language())
    //     .expect("Error loading Python grammar");
    let tree = parser.parse(&src, None).unwrap();
    if tree.root_node().has_error() {
        println!(
            "{:?}",
            miette!(severity = Severity::Warning, "{}: parse error", arg).with_source_code(nsrc)
        );
    }
    for (off, ch) in src.char_indices() {
        if ch.is_ascii() {
            continue;
        }
        let node = tree
            .root_node()
            .named_descendant_for_byte_range(off, off + ch.len_utf8())
            .unwrap();
        let kind = node.kind();
        if kind == "comment" || kind == "string_fragment" {
            continue;
        }
        let chname = Name::of(ch).unwrap();
        let report = miette!(
            labels = vec![LabeledSpan::at(
                off..off + ch.len_utf8(),
                chname.to_string()
            )],
            "found non-ascii character {} in {}",
            chname,
            node.kind()
        )
        .with_source_code(NamedSource::new(arg, src.clone()));
        println!("{:?}", report);
    }
}
