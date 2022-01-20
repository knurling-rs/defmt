use serde::{Deserialize, Serialize};

pub mod v1 {
    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct JsonFrame {
        pub data: String,
        pub decoder_version: &'static str,
        /// Unix timestamp in nanoseconds
        pub host_timestamp: i64,
        pub level: String,
        pub location: Location,
        pub target_timestamp: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Location {
        pub file: Option<String>,
        pub line: Option<u32>,
        pub module_path: Option<ModulePath>,
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ModulePath {
        pub crate_name: String,
        pub modules: Vec<String>,
        pub function: String,
    }
}
