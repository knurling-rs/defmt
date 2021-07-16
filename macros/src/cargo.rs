use std::env;

pub(crate) fn package_name() -> String {
    env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "<unknown>".to_string())
}
