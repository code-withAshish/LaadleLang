use std::env;
use std::process::Command;

fn main() {
    // Only run this build script if we are *not* compiling for wasm32-unknown-unknown
    // to avoid infinite recursion when wasm-pack invokes cargo build internally.
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default() == "wasm32" {
        return;
    }

    // Re-run the build script if any frontend assets or Rust source files change
    println!("cargo:rerun-if-changed=playground/src");
    println!("cargo:rerun-if-changed=playground/package.json");
    println!("cargo:rerun-if-changed=src");

    let is_release = env::var("PROFILE").unwrap_or_default() == "release";

    // 1. Compile LaadleLang to WASM
    // We isolate the target dir setting custom CARGO_TARGET_DIR for wasm-pack
    // to avoid deadlock with the active cargo build process.
    let mut wasm_pack = Command::new("wasm-pack");
    wasm_pack.args(&[
        "build",
        "--target",
        "web",
        "--out-dir",
        "playground/src/wasm",
    ]);
    if !is_release {
        wasm_pack.arg("--dev");
    }
    wasm_pack.env("CARGO_TARGET_DIR", "target/wasm-pack");

    println!("cargo:warning=Compiling WASM via wasm-pack...");
    let status = wasm_pack.status().expect("Failed to execute wasm-pack");
    if !status.success() {
        panic!("wasm-pack build failed");
    }

    // Provide the correct npm executable based on the host OS
    let npm_cmd = if cfg!(windows) { "npm.cmd" } else { "npm" };

    // 2. Install NPM Dependencies locally in `playground/`
    println!("cargo:warning=Installing NPM dependencies...");
    let status = Command::new(npm_cmd)
        .arg("install")
        .current_dir("playground")
        .status()
        .expect("Failed to execute npm install");
    if !status.success() {
        panic!("npm install failed");
    }

    // 3. Bundle Playground with Esbuild
    println!("cargo:warning=Bundling playground and minifying assets...");
    let status = Command::new(npm_cmd)
        .arg("run")
        .arg("build")
        .current_dir("playground")
        .status()
        .expect("Failed to execute npm run build");
    if !status.success() {
        panic!("npm run build failed");
    }
}
