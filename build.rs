use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=templates/**");
    println!("cargo:rerun-if-changed=tailwind.config.js");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=package-lock.json");

    let status = Command::new("npm")
        .arg("run")
        .arg("build:css")
        .status()
        .expect("Failed to execute npm. Is Node.js installed?");

    if !status.success() {
        panic!("Tailwind CSS build failed");
    }
}
