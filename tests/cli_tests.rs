#[test]
fn cli_tests() {
    trycmd::TestCases::new()
        // .case("README.md")
        .case("tests/cmd/*.trycmd");
}

#[test]
fn with_zipkin_tests() {
    trycmd::TestCases::new()
        // .case("README.md")
        .case("tests/with_zipkin/*.trycmd");
}

#[test]
fn without_zipkin_tests() {
    trycmd::TestCases::new()
        // .case("README.md")
        .case("tests/without_zipkin/*.trycmd");
}
