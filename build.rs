use std::process::Command;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo::rerun-if-changed=assets/");

    Command::new("npm")
        .arg("run")
        .arg("build")
        .output()
        .expect("failed to execute process");
}
