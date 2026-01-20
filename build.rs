use std::{fs, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=frontend/css");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/ts");
    println!("cargo:rerun-if-changed=frontend/tsconfig.json");

    let generated = Path::new("frontend/dist/generated");
    if generated.exists() && !cfg!(debug_assertions) {
        fs::remove_dir_all(generated).expect("Failed to clean `frontend/dist/generated`");
    }

    let status = Command::new("npm")
        .arg("run")
        .arg("build:css")
        .current_dir("frontend")
        .status()
        .expect("Failed to execute npm. Is Node.js installed?");

    if !status.success() {
        panic!("Tailwind CSS build failed");
    }

    let js_script = if cfg!(debug_assertions) {
        "build:js-sourceMap"
    } else {
        "build:js"
    };

    let js_status = Command::new("npm")
        .arg("run")
        .arg(js_script)
        .current_dir("frontend")
        .status()
        .expect("Failed to execute npm. Is Node.js installed?");

    if !js_status.success() {
        panic!("TypeScript build failed");
    }
}
