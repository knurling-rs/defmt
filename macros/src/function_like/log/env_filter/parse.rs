use defmt_parser::Level;
use proc_macro_error::abort_call_site;
use syn::Ident;

// None = "off" pseudo-level
pub(crate) type LogLevelOrOff = Option<Level>;

// NOTE this is simpler than `syn::Path`; we do not want to accept e.g. `Vec::<Ty>::new`
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ModulePath(String);

/// Parses the contents of the `DEFMT_LOG` env var
pub(crate) fn defmt_log(input: &str) -> impl Iterator<Item = Entry> + '_ {
    input.rsplit(',').map(|entry| {
        if let Some((path, log_level)) = entry.rsplit_once('=') {
            let module_path = ModulePath::parse(path);
            let log_level = parse_log_level(log_level)
                .unwrap_or_else(|_| abort_call_site!("unknown log level `{}`", log_level));

            Entry::ModulePathLogLevel {
                module_path,
                log_level,
            }
        } else if let Ok(log_level) = parse_log_level(entry) {
            Entry::LogLevel(log_level)
        } else {
            Entry::ModulePath(ModulePath::parse(entry))
        }
    })
}

#[derive(Debug, PartialEq)]
pub(crate) enum Entry {
    LogLevel(LogLevelOrOff),
    ModulePath(ModulePath),
    ModulePathLogLevel {
        module_path: ModulePath,
        log_level: LogLevelOrOff,
    },
}

impl ModulePath {
    pub(crate) fn from_crate_name(input: &str) -> Self {
        if input.is_empty() && input.contains("::") {
            abort_call_site!("crate name cannot be an empty string or contain path separators")
        }
        Self::parse(input)
    }

    fn parse(input: &str) -> Self {
        if input.is_empty() {
            abort_call_site!("module path cannot be an empty string")
        }

        input.split("::").for_each(validate_identifier);

        Self(input.to_string())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn crate_name(&self) -> &str {
        self.0
            .split_once("::")
            .map(|(crate_name, _rest)| crate_name)
            .unwrap_or(&self.0)
    }
}

fn parse_log_level(input: &str) -> Result<LogLevelOrOff, ()> {
    Ok(Some(match input {
        "debug" => Level::Debug,
        "error" => Level::Error,
        "info" => Level::Info,
        "off" => return Ok(None),
        "trace" => Level::Trace,
        "warn" => Level::Warn,
        _ => return Err(()),
    }))
}

fn validate_identifier(input: &str) {
    syn::parse_str::<Ident>(input)
        .unwrap_or_else(|_| abort_call_site!("`{}` is not a valid identifier", input));
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    use super::*;

    #[test]
    fn parses_from_the_right() {
        let entries = defmt_log("krate=info,krate,info").collect::<Vec<_>>();
        assert_eq!(
            [
                Entry::LogLevel(Some(Level::Info)),
                Entry::ModulePath(ModulePath("krate".to_string())),
                Entry::ModulePathLogLevel {
                    module_path: ModulePath("krate".to_string()),
                    log_level: Some(Level::Info)
                },
            ],
            entries.as_slice()
        );
    }

    #[test]
    fn accepts_raw_identifier() {
        ModulePath::parse("krate::r#mod");
    }

    #[rstest]
    #[case::has_module("krate::module")]
    #[case::no_module("krate")]
    fn modpath_crate_name(#[case] input: &str) {
        let modpath = ModulePath::parse(input);
        assert_eq!("krate", modpath.crate_name());
    }

    #[rstest]
    #[case::crate_name_is_invalid("some-crate::module")]
    #[case::module_name_is_invalid("krate::some-module")]
    #[case::with_level("krate::some-module=info")]
    #[should_panic = "not a valid identifier"]
    fn rejects_invalid_identifier(#[case] input: &str) {
        defmt_log(input).next();
    }

    #[test]
    #[should_panic = "unknown log level"]
    fn rejects_unknown_log_level() {
        defmt_log("krate=module").next();
    }

    #[test]
    #[should_panic = "module path cannot be an empty string"]
    fn rejects_empty_module_path() {
        defmt_log("=info").next();
    }
}
