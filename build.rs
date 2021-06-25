use std::{env, error::Error, fs::File, io::Write, path::PathBuf};

use cc::Build;

fn main() -> Result<(), Box<dyn Error>> {
    // build directory for this crate
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // extend the library search path
    println!("cargo:rustc-link-search={}", out_dir.display());

    // put `linker.ld` in the build directory
    File::create(out_dir.join("linker.ld"))?.write_all(include_bytes!("linker.ld"))?;

    // assemble the assembly file
    Build::new().file("src/entry.S").compile("asm");

    // rebuild if `entry.s` changed
    println!("cargo:rerun-if-changed=src/entry.S");

    Ok(())
}
