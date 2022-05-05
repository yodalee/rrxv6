use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use cc::Build;

fn main() -> Result<(), Box<dyn Error>> {
    // build directory for this crate
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // extend the library search path
    println!("cargo:rustc-link-search={}", out_dir.display());

    // put `linker.ld` in the build directory
    File::create(out_dir.join("linker.ld"))?.write_all(include_bytes!("linker.ld"))?;

    // assemble the assembly file
    Build::new()
        //.debug(true)
        .file("src/asm/entry.S")
        .file("src/asm/kernelvec.S")
        .file("src/asm/trampoline.S")
        .file("src/asm/switch.S")
        .compile("asm");

    // rebuild if assembly target changed
    println!("cargo:rerun-if-changed=src/entry.S");
    println!("cargo:rerun-if-changed=src/kernelvec.S");
    println!("cargo:rerun-if-changed=src/trampoline.S");
    println!("cargo:rerun-if-changed=src/switch.S");

    // write byte data of initcode to file
    let dest_path = Path::new(&out_dir).join("initcode.rs");
    let buf = Command::new("stat")
        .args(&["-c", "%s", "user/initcode"])
        .output().unwrap();
    let len = String::from_utf8_lossy(&buf.stdout)
        .trim()
        .parse::<u32>().unwrap();
    let buf = Command::new("xxd")
        .args(&["-p", "-c", "1", "user/initcode"])
        .output().unwrap();
    let out = String::from_utf8_lossy(&buf.stdout)
        .trim()
        .split('\n')
        .map(|num| format!("0x{}", num))
        .collect::<Vec<String>>()
        .join(",");
    let mut f = File::create(&dest_path).unwrap();
    writeln!(f, "/// The first user program that run infinite loop").unwrap();
    writeln!(f, "/// od -An -t x1 initcode").unwrap();
    writeln!(f, "pub static INITCODE: [u8;{}] = [", len).unwrap();
    writeln!(f, "  {}", out).unwrap();
    writeln!(f, "];").unwrap();

    Ok(())
}
