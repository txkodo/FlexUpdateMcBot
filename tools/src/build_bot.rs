use anyhow::{Context, Result};
use clap::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use tools::*;

#[derive(Parser)]
#[command(name = "build-bot")]
#[command(about = "Build bot for specified platform")]
struct Cli {
    /// Target platform (e.g., x86_64-unknown-linux-gnu)
    #[arg(long)]
    target: Option<String>,
    /// Output directory for artifacts
    #[arg(long, default_value = "artifacts")]
    output_dir: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get Minecraft version from bot metadata
    let bot_config = get_bot_config()?;
    let mc_version = &bot_config.package.metadata.mc_version;

    // Determine target and file extension
    let target: String = cli.target.unwrap_or_else(|| {
        // Default target based on current platform
        let arch = env::consts::ARCH;
        let os = env::consts::OS;
        match (os, arch) {
            ("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),
            ("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),
            ("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),
            ("macos", "x86_64") => "x86_64-apple-darwin".to_string(),
            ("macos", "aarch64") => "aarch64-apple-darwin".to_string(),
            _ => format!("{}-unknown-{}-gnu", arch, os),
        }
    });

    let is_windows = target.contains("windows");
    let exe_extension = if is_windows { ".exe" } else { "" };

    println!("Building for target: {}", target);
    println!("Minecraft version: {}", mc_version);

    // Build the bot
    let mut build_cmd = Command::new("cargo");
    build_cmd
        .args(["+nightly", "build", "--release"])
        .current_dir("bot");

    // Add target if it's not the default
    if target != format!("{}-unknown-{}-gnu", env::consts::ARCH, env::consts::OS) {
        build_cmd.args(["--target", &target]);
    }

    let build_output = build_cmd
        .output()
        .context("Failed to execute cargo build")?;

    if !build_output.status.success() {
        anyhow::bail!(
            "cargo build failed: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
    }

    // Determine source binary path
    let binary_name = format!("flex-update-mc-bot{}", exe_extension);
    let source_path = if target == format!("{}-unknown-{}-gnu", env::consts::ARCH, env::consts::OS)
    {
        format!("bot/target/release/{}", binary_name)
    } else {
        format!("bot/target/{}/release/{}", target, binary_name)
    };

    // Create output directory
    fs::create_dir_all(&cli.output_dir).context("Failed to create output directory")?;

    // Determine output filename
    let os_name = if target.contains("linux") {
        "linux"
    } else if target.contains("windows") {
        "windows"
    } else if target.contains("darwin") {
        "macos"
    } else {
        "unknown"
    };

    let arch_name = if target.contains("x86_64") {
        "x64"
    } else if target.contains("aarch64") {
        "arm64"
    } else {
        "unknown"
    };

    let output_filename = format!(
        "flex-update-mc-bot-{}-{}-{}{}",
        mc_version, os_name, arch_name, exe_extension
    );
    let output_path = Path::new(&cli.output_dir).join(&output_filename);

    println!("Output binary will be: {}", output_path.display());

    // Copy binary to output directory
    fs::copy(&source_path, &output_path).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            source_path,
            output_path.display()
        )
    })?;

    println!("Build completed: {}", output_path.display());

    Ok(())
}
