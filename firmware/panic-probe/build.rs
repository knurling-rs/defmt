use std::env;

fn main() {
    // allow all possible configs
    let possible_configs = [
        "cortex_m",
        "armv6m",
        "armv7m",
        "armv7em",
        "armv8m",
        "armv8m_base",
        "armv8m_main",
    ];
    for config in possible_configs {
        println!("cargo:rustc-check-cfg=cfg({config})");
    }

    // enable configs for target
    let target = env::var("TARGET").unwrap();
    let enabled_configs = if target.starts_with("thumbv6m-") {
        ["cortex_m", "armv6m"].as_slice()
    } else if target.starts_with("thumbv7m-") {
        ["cortex_m", "armv7m"].as_slice()
    } else if target.starts_with("thumbv7em-") {
        ["cortex_m", "armv7m", "armv7em"].as_slice()
    } else if target.starts_with("thumbv8m.base") {
        ["cortex_m", "armv8m", "armv8m_base"].as_slice()
    } else if target.starts_with("thumbv8m.main") {
        ["cortex_m", "armv8m", "armv8m_main"].as_slice()
    } else {
        [].as_slice()
    };
    for config in enabled_configs {
        println!("cargo:rustc-cfg={config}");
    }
}
