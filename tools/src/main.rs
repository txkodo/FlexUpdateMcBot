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

#[derive(Debug, Deserialize)]
struct AzaleaCargoToml {
    workspace: WorkspaceInfo,
}

#[derive(Debug, Deserialize)]
struct WorkspaceInfo {
    package: PackageInfo,
}
#[derive(Debug, Deserialize)]
struct PackageInfo {
    #[serde(default)]
    version: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct BotCargoToml {
    dependencies: BotDependencies,
    package: BotPackageInfo,
    #[serde(flatten)]
    others: HashMap<String, toml::Value>,
}
#[derive(Debug, Deserialize, Serialize)]
struct BotDependencies {
    azalea: AzaleaDependency,
    #[serde(flatten)]
    others: HashMap<String, toml::Value>,
}
#[derive(Debug, Deserialize, Serialize)]
struct AzaleaDependency {
    git: String,
    rev: String,
}
#[derive(Debug, Deserialize, Serialize)]
struct BotPackageInfo {
    metadata: BotMetadata,
    #[serde(flatten)]
    others: HashMap<String, toml::Value>,
}
#[derive(Debug, Deserialize, Serialize)]
struct BotMetadata {
    mc_version: String,
    #[serde(flatten)]
    others: HashMap<String, toml::Value>,
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
    let mut bot_config = get_bot_config()?;
    println!(
        "Current azalea rev: {}",
        &bot_config.dependencies.azalea.rev
    );

    // 2. Clone azalea repository
    let azalea_path = "./azalea-temp";
    clone_azalea_repo(azalea_path).await?;

    // 3. Find next commit after current rev
    let next_rev = find_next_commit(azalea_path, &bot_config.dependencies.azalea.rev)?;

    if let Some(next_rev) = next_rev {
        println!("Next azalea rev: {}", next_rev);

        // Checkout the next revision
        checkout_revision(azalea_path, &next_rev)?;

        // 4. Get Minecraft version from azalea Cargo.toml
        let mc_version = get_minecraft_version(azalea_path)?;
        println!("Minecraft version: {}", mc_version);

        // 5. Update bot/Cargo.toml
        bot_config.dependencies.azalea.rev = next_rev.clone();
        bot_config.package.metadata.mc_version = mc_version.clone();
        update_bot_config(&bot_config)?;

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

fn get_bot_config() -> Result<BotCargoToml> {
    let cargo_toml_content =
        fs::read_to_string("bot/Cargo.toml").context("Failed to read bot/Cargo.toml")?;
    Ok(toml::from_str(&cargo_toml_content).context("Failed to parse bot/Cargo.toml")?)
}

fn update_bot_config(bot_config: &BotCargoToml) -> Result<()> {
    fs::write("bot/Cargo.toml", toml::to_string(bot_config)?)
        .context("Failed to write updated bot/Cargo.toml")?;
    Ok(())
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

    // Parse current_rev to Oid and hide it (and all older commits)
    let current_oid =
        git2::Oid::from_str(current_rev).context("Failed to parse current revision")?;
    revwalk.hide(current_oid)?;

    // Collect all commits newer than current_rev
    let commits: Result<Vec<String>, _> = revwalk
        .collect::<Result<Vec<_>, _>>()
        .map(|oids| oids.into_iter().map(|oid| oid.to_string()).collect());

    let commits = commits?;

    // The next commit is the oldest among the newer commits (last in the list)
    if !commits.is_empty() {
        Ok(Some(commits.last().unwrap().clone()))
    } else {
        Ok(None) // current_rev is the latest commit
    }
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

    let cargo_toml: AzaleaCargoToml =
        toml::from_str(&cargo_toml_content).context("Failed to parse azalea/Cargo.toml")?;

    let full_version = cargo_toml.workspace.package.version;

    // Extract Minecraft version from format like "0.13.0+mc1.21.7" -> "1.21.7"
    let re = regex::Regex::new(r"\+mc(.+)$").unwrap();
    if let Some(caps) = re.captures(&full_version) {
        Ok(caps[1].to_string())
    } else {
        anyhow::bail!("Could not extract Minecraft version from: {}", full_version);
    }
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
