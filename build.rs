use std::process::{exit, Command};

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=assets/");

    let output = Command::new("npm").arg("run").arg("build").output();
    match output {
        Ok(out) if out.status.success() => {
            exit(0);
        }
        Ok(out) => {
            let out_string = String::from_utf8_lossy(&out.stdout);
            panic!("Failed to build assets: {out_string}");
        }
        Err(e) => {
            panic!("Failed to build assets: {e}");
        }
    }
}
