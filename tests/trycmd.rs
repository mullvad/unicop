#[test]
fn trycmd() {
    trycmd::TestCases::new()
        .case("README.md")
        .case("tests/examples.trycmd");
}
