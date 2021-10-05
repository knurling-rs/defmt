#[test]
fn ui() {
    // only test error messages on the stable channel (nightly may change too often)
    if rustc_version::version_meta()
        .map(|meta| meta.channel == rustc_version::Channel::Stable)
        .unwrap_or(false)
    {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*.rs");

        t.pass("tests/basic_usage.rs");
    }
}
