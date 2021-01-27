#[test]
fn ui() {
    // only test error messages on the stable channel (nightly may change too often)
    if rustc_version::version_meta()
        .map(|meta| meta.channel == rustc_version::Channel::Stable)
        .unwrap_or(false)
    {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/*.rs");

        // TODO once the corresponding fix in cortex-m-rt has been released,
        // ()
        // re-enable this test and remove the macos special casing in `ci.yml`
        // t.pass("tests/basic_usage.rs");
    }
}
