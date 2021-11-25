fn main() {
    println!("cargo:rerun-if-env-changed=DEFMT_RTT_BUFFER_SIZE");

    let size = option_env!("DEFMT_RTT_BUFFER_SIZE")
        .map(|s| s.parse().ok())
        .flatten()
        .unwrap_or(1024_usize);

    let non_power_of_two = (size & (size - 1)) != 0;

    if non_power_of_two {
        println!("cargo:warning=RTT buffer size of {} is not a power of two, performance will be degraded.", size)
    }

    let out_dir_path = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let out_file_path = out_dir_path.join("consts.rs");

    std::fs::write(
        out_file_path,
        format!("pub(crate) const BUF_SIZE: usize = {};", size),
    )
    .unwrap();
}
