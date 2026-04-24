use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../original-c/source/crx2rnx.c");

    let out_dir = match env::var("OUT_DIR") {
        Ok(v) => PathBuf::from(v),
        Err(_) => return,
    };

    let exe_name = if cfg!(windows) {
        "crx2rnx_helper.exe"
    } else {
        "crx2rnx_helper"
    };
    let exe_path = out_dir.join(exe_name);

    let status = Command::new("cc")
        .arg("-O2")
        .arg("-std=c99")
        .arg("../original-c/source/crx2rnx.c")
        .arg("-o")
        .arg(&exe_path)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-env=CRX2RNX_HELPER={}", exe_path.display());
        }
        Ok(_) | Err(_) => {
            // Dejar que la librería emita un error claro en runtime si falta helper.
            println!("cargo:rustc-env=CRX2RNX_HELPER=");
        }
    }
}