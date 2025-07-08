use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(name = "tools")]
#[command(about = "FlexUpdateMcBot tools for updating azalea dependencies")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update bot dependencies to next azalea revision
    Update,
}

#[derive(Debug, Deserialize, Serialize)]
struct CargoToml {
    dependencies: HashMap<String, DependencyValue>,
    #[serde(default)]
    package: Option<PackageInfo>,
    #[serde(default)]
    workspace: Option<WorkspaceInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PackageInfo {
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkspaceInfo {
    #[serde(default)]
    package: Option<PackageInfo>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum DependencyValue {
    Simple(String),
    Detailed(DetailedDependency),
}

#[derive(Debug, Deserialize, Serialize)]
struct DetailedDependency {
    #[serde(default)]
    git: Option<String>,
    #[serde(default)]
    rev: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Update => update_bot().await?,
    }

    Ok(())
}

async fn update_bot() -> Result<()> {
    // 1. Parse bot/Cargo.toml to get current azalea rev
    let current_rev = get_current_azalea_rev()?;
    println!("Current azalea rev: {}", current_rev);

    // 2. Clone azalea repository
    let azalea_path = "./azalea-temp";
    clone_azalea_repo(azalea_path).await?;

    // 3. Find next commit after current rev
    let next_rev = find_next_commit(azalea_path, &current_rev)?;

    if let Some(next_rev) = next_rev {
        println!("Next azalea rev: {}", next_rev);

        // Checkout the next revision
        checkout_revision(azalea_path, &next_rev)?;

        // 4. Get Minecraft version from azalea Cargo.toml
        let mc_version = get_minecraft_version(azalea_path)?;
        println!("Minecraft version: {}", mc_version);

        // 5. Update bot/Cargo.toml
        update_bot_cargo_toml(&next_rev)?;

        // 6. Copy azalea/Cargo.lock to bot/Cargo.lock
        copy_cargo_lock(azalea_path)?;

        // 7. Run cargo update in bot directory
        run_cargo_update()?;

        // 8. Create git commit
        create_git_commit(&next_rev, &mc_version)?;

        println!("Bot updated successfully to revision: {}", next_rev);
    } else {
        println!("Already at latest revision");
    }

    Ok(())
}

fn get_current_azalea_rev() -> Result<String> {
    let cargo_toml_content =
        fs::read_to_string("bot/Cargo.toml").context("Failed to read bot/Cargo.toml")?;

    let cargo_toml: CargoToml =
        toml::from_str(&cargo_toml_content).context("Failed to parse bot/Cargo.toml")?;

    if let Some(DependencyValue::Detailed(azalea_dep)) = cargo_toml.dependencies.get("azalea") {
        if let Some(rev) = &azalea_dep.rev {
            return Ok(rev.clone());
        }
    }

    anyhow::bail!("Could not find azalea revision in bot/Cargo.toml");
}

async fn clone_azalea_repo(path: &str) -> Result<()> {
    // Remove existing directory if it exists
    if Path::new(path).exists() {
        fs::remove_dir_all(path).context("Failed to remove existing azalea directory")?;
    }

    Repository::clone("https://github.com/azalea-rs/azalea", path)
        .context("Failed to clone azalea repository")?;

    Ok(())
}

fn find_next_commit(repo_path: &str, current_rev: &str) -> Result<Option<String>> {
    let repo = Repository::open(repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    let mut found_current = false;

    for oid in revwalk {
        let oid = oid?;
        let commit_hash = oid.to_string();

        if found_current {
            return Ok(Some(commit_hash));
        }

        if commit_hash.starts_with(current_rev) {
            found_current = true;
        }
    }

    Ok(None)
}

fn checkout_revision(repo_path: &str, rev: &str) -> Result<()> {
    let repo = Repository::open(repo_path)?;
    let oid = git2::Oid::from_str(rev)?;
    let commit = repo.find_commit(oid)?;

    repo.checkout_tree(commit.as_object(), None)?;
    repo.set_head_detached(oid)?;

    Ok(())
}

fn get_minecraft_version(azalea_path: &str) -> Result<String> {
    let cargo_toml_path = format!("{}/Cargo.toml", azalea_path);
    let cargo_toml_content =
        fs::read_to_string(&cargo_toml_path).context("Failed to read azalea/Cargo.toml")?;

    let cargo_toml: CargoToml =
        toml::from_str(&cargo_toml_content).context("Failed to parse azalea/Cargo.toml")?;

    if let Some(workspace) = &cargo_toml.workspace {
        if let Some(package) = &workspace.package {
            if let Some(version) = &package.version {
                return Ok(version.clone());
            }
        }
    }

    anyhow::bail!("Could not find version in azalea/Cargo.toml workspace.package.version");
}

fn update_bot_cargo_toml(new_rev: &str) -> Result<()> {
    let cargo_toml_path = "bot/Cargo.toml";
    let cargo_toml_content =
        fs::read_to_string(cargo_toml_path).context("Failed to read bot/Cargo.toml")?;

    let mut cargo_toml: CargoToml =
        toml::from_str(&cargo_toml_content).context("Failed to parse bot/Cargo.toml")?;

    if let Some(DependencyValue::Detailed(azalea_dep)) = cargo_toml.dependencies.get_mut("azalea") {
        azalea_dep.rev = Some(new_rev.to_string());
    }

    let updated_content =
        toml::to_string_pretty(&cargo_toml).context("Failed to serialize updated Cargo.toml")?;

    fs::write(cargo_toml_path, updated_content)
        .context("Failed to write updated bot/Cargo.toml")?;

    Ok(())
}

fn copy_cargo_lock(azalea_path: &str) -> Result<()> {
    let source = format!("{}/Cargo.lock", azalea_path);
    let dest = "bot/Cargo.lock";

    fs::copy(&source, dest).context("Failed to copy Cargo.lock from azalea to bot")?;

    Ok(())
}

fn run_cargo_update() -> Result<()> {
    let output = Command::new("cargo")
        .args(["update"])
        .current_dir("bot")
        .output()
        .context("Failed to execute cargo update")?;

    if !output.status.success() {
        anyhow::bail!(
            "cargo update failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}

fn create_git_commit(rev: &str, mc_version: &str) -> Result<()> {
    // Add all changes
    let add_output = Command::new("git")
        .args(["add", "."])
        .output()
        .context("Failed to add changes to git")?;

    if !add_output.status.success() {
        anyhow::bail!(
            "git add failed: {}",
            String::from_utf8_lossy(&add_output.stderr)
        );
    }

    // Create commit
    let commit_message = format!("Update azalea to {} (MC {})", &rev[..8], mc_version);
    let commit_output = Command::new("git")
        .args(["commit", "-m", &commit_message])
        .output()
        .context("Failed to create git commit")?;

    if !commit_output.status.success() {
        anyhow::bail!(
            "git commit failed: {}",
            String::from_utf8_lossy(&commit_output.stderr)
        );
    }

    Ok(())
}
