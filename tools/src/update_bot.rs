use anyhow::Result;
use clap::Parser;
use tools::*;

#[derive(Parser)]
#[command(name = "update-bot")]
#[command(about = "Update bot dependencies to next azalea revision")]
struct Cli {}

#[tokio::main]
async fn main() -> Result<()> {
    let _cli = Cli::parse();

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