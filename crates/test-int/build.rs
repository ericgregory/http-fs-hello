use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{Context as _, Result, ensure};

#[tokio::main]
async fn main() -> Result<()> {
    // Watch the manifest thread
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let component_dir = manifest_dir.join("../../components/http-fs-hello");
    println!("cargo:rerun-if-changed={}", component_dir.display());

    // Build the component
    let component_manifest_path = component_dir.join("Cargo.toml");
    let output = tokio::process::Command::new("cargo")
        .args([
            "build",
            "--manifest-path",
            &format!("{}", component_manifest_path.display()),
            "--target",
            "wasm32-wasip2",
            "--release",
        ])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .output()
        .await
        .context("failed to build component")?;
    ensure!(
        output.status.success(),
        "build command failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let wasm_output_path = component_dir.join("target/wasm32-wasip2/release/http_fs_hello.wasm");
    ensure!(
        tokio::fs::try_exists(&wasm_output_path)
        .await
        .is_ok_and(|v| v),
        "missing expected output @ [{}]",
        wasm_output_path.display()
    );
    println!(
        "cargo:rustc-env=COMPONENT_HTTP_FS_HELLO_PATH={}",
        wasm_output_path.display()
    );

    Ok(())
}
