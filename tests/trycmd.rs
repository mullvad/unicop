#[test]
fn trycmd() {
    trycmd::TestCases::new()
        .case("README.md")
        .case("example-files/README.md");
}
