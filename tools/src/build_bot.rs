use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::Path;
use std::process::Command;
use tools::*;

#[derive(Parser)]
#[command(name = "build-bot")]
#[command(about = "Build bot for specified platform")]
struct Cli {
    /// Operating system (linux, windows, macos)
    #[arg(long)]
    os: String,
    /// Architecture (x64, arm64)
    #[arg(long)]
    arch: String,
    /// Output directory for artifacts
    #[arg(long, default_value = "artifacts")]
    output_dir: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Get Minecraft version from bot metadata
    let bot_config = get_bot_config()?;
    let mc_version = &bot_config.package.metadata.mc_version;

    // Construct target from OS and ARCH
    let target = match (cli.os.as_str(), cli.arch.as_str()) {
        ("linux", "x64") => "x86_64-unknown-linux-gnu",
        ("linux", "arm64") => "aarch64-unknown-linux-gnu",
        ("windows", "x64") => "x86_64-pc-windows-msvc",
        ("windows", "arm64") => "aarch64-pc-windows-msvc",
        ("macos", "x64") => "x86_64-apple-darwin",
        ("macos", "arm64") => "aarch64-apple-darwin",
        _ => anyhow::bail!("Unsupported OS/ARCH combination: {}/{}", cli.os, cli.arch),
    };

    // Add target for cross-compilation
    println!("Adding target: {}", target);
    let target_add_output = Command::new("rustup")
        .args(["target", "add", target])
        .output()
        .context("Failed to execute rustup target add")?;

    if !target_add_output.status.success() {
        let stderr = String::from_utf8_lossy(&target_add_output.stderr);
        // Only warn if it's not already installed
        if !stderr.contains("already installed") {
            println!("Warning: Failed to add target: {}", stderr);
        } else {
            println!("Target {} already installed", target);
        }
    } else {
        println!("Successfully added target: {}", target);
    }

    let is_windows = target.contains("windows");
    let exe_extension = if is_windows { ".exe" } else { "" };

    println!("Building for target: {}", target);
    println!("Minecraft version: {}", mc_version);

    // Build the bot
    let mut build_cmd = Command::new("cargo");

    // Setup cross-compilation linker for Linux ARM64
    if target == "aarch64-unknown-linux-gnu" {
        build_cmd.env(
            "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER",
            "aarch64-linux-gnu-gcc",
        );
    }

    build_cmd
        .args(["build", "--release"])
        .current_dir("bot");

    // Always add target
    build_cmd.args(["--target", &target]);

    println!("Executing build command: {:?}", build_cmd);
    let build_output = build_cmd
        .output()
        .context("Failed to execute cargo build")?;

    if !build_output.status.success() {
        println!(
            "Build stdout: {}",
            String::from_utf8_lossy(&build_output.stdout)
        );
        println!(
            "Build stderr: {}",
            String::from_utf8_lossy(&build_output.stderr)
        );
        anyhow::bail!(
            "cargo build failed with exit code: {:?}",
            build_output.status.code()
        );
    } else {
        println!("Build completed successfully");
    }

    // Determine source binary path
    let binary_name = format!("flex-update-mc-bot{}", exe_extension);
    let source_path = format!("bot/target/{}/release/{}", target, binary_name);

    // Create output directory
    fs::create_dir_all(&cli.output_dir).context("Failed to create output directory")?;

    // Determine output filename
    let output_filename = format!(
        "flex-update-mc-bot-{}-{}-{}{}",
        mc_version, cli.os, cli.arch, exe_extension
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
