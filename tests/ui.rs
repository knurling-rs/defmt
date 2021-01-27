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
        // ( https://github.com/rust-embedded/cortex-m-rt/pull/306
        // https://github.com/rust-embedded/cortex-m-rt/pull/307 )
        // re-enable this test (deleted in commit d20ec32) and remove the macos special casing in `ci.yml`
        // also check out improved approach re: linker sections in the second cortex-m-rt PR
        // t.pass("tests/basic_usage.rs");
    }
}
