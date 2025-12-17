use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/**");

    let status = Command::new("npm")
        .arg("run")
        .arg("build:css")
        .current_dir("frontend")
        .status()
        .expect("Failed to execute npm. Is Node.js installed?");

    if !status.success() {
        panic!("Tailwind CSS build failed");
    }
}
