use std::{env, error::Error, fs, path::PathBuf};

fn sub(linker_script: String) -> String {
    #[cfg(not(feature = "avoid-default-panic"))]
    {
        linker_script
    }
    #[cfg(feature = "avoid-default-panic")]
    {
        linker_script.replacen("PROVIDE(_defmt_panic = __defmt_default_panic);", "", 1)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var("OUT_DIR")?);
    let linker_script = fs::read_to_string("defmt.x.in")?;
    fs::write(out.join("defmt.x"), sub(linker_script))?;
    println!("cargo:rustc-link-search={}", out.display());
    let target = env::var("TARGET")?;

    // `"atomic-cas": false` in `--print target-spec-json`
    // last updated: rust 1.48.0
    match &target[..] {
        "avr-gnu-base"
        | "msp430-none-elf"
        | "riscv32i-unknown-none-elf"
        | "riscv32imc-unknown-none-elf"
        | "thumbv4t-none-eabi"
        | "thumbv6m-none-eabi" => {
            println!("cargo:rustc-cfg=no_cas");
        }
        _ => {}
    }
    Ok(())
}
