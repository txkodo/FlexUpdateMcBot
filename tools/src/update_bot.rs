use anyhow::Result;
use clap::Parser;
use tools::*;

#[derive(Parser)]
#[command(name = "update-bot")]
#[command(about = "Update bot dependencies to next azalea revision")]
struct Cli {
    #[arg(long, help = "Specific azalea revision to update to")]
    next_rev: Option<String>,
    #[arg(long, help = "Minecraft version")]
    mc_version: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Parse bot/Cargo.toml to get current azalea rev
    let mut bot_config = get_bot_config()?;
    println!(
        "Current azalea rev: {}",
        &bot_config.dependencies.azalea_protocol.rev
    );

    // 2. Clone azalea repository
    let azalea_path = "./azalea-temp";
    clone_azalea_repo(azalea_path).await?;

    // 3. Get latest commit (or use provided rev)
    let next_rev = if let Some(specified_rev) = cli.next_rev {
        println!("Using specified revision: {}", specified_rev);
        specified_rev
    } else {
        get_latest_commit(azalea_path)?
    };

    // Check if we need to update (compare with current rev)
    if next_rev != bot_config.dependencies.azalea_protocol.rev {
        println!("Latest azalea rev: {}", next_rev);

        // Checkout the latest revision
        checkout_revision(azalea_path, &next_rev)?;

        // 4. Get Minecraft version from azalea Cargo.toml or use provided version
        let mc_version = if let Some(provided_version) = cli.mc_version {
            println!("Using provided Minecraft version: {}", provided_version);
            provided_version
        } else {
            let version = get_minecraft_version(azalea_path)?;
            println!("Minecraft version: {}", version);
            version
        };

        // 5. Update bot/Cargo.toml
        bot_config.dependencies.azalea_protocol.rev = next_rev.clone();
        bot_config.dependencies.azalea_client.rev = next_rev.clone();

        let cargo_lock = get_cargo_lock(azalea_path)?;

        // Find anyhow and tokio versions from Cargo.lock
        let anyhow_version = cargo_lock
            .package
            .iter()
            .find(|p| p.name == "anyhow")
            .map(|p| p.version.clone())
            .unwrap();

        let tokio_version = cargo_lock
            .package
            .iter()
            .find(|p| p.name == "tokio")
            .map(|p| p.version.clone())
            .unwrap();

        bot_config.dependencies.anyhow = anyhow_version;
        bot_config.dependencies.tokio = tokio_version;

        bot_config.package.metadata.mc_version = mc_version.clone();

        // 7. Sync edition with azalea-client
        let azalea_edition = get_azalea_client_edition(azalea_path)?;
        println!("Syncing edition to: {}", azalea_edition);
        bot_config.package.edition = azalea_edition;

        update_bot_config(&bot_config)?;

        // 6. Copy azalea/Cargo.lock to bot/Cargo.lock
        copy_cargo_lock(azalea_path)?;

        // 8. Create git commit
        create_git_commit(&next_rev, &mc_version)?;

        println!("Bot updated successfully to revision: {}", next_rev);
        println!("has_changes=true");
    } else {
        println!("Already at latest revision");
        println!("has_changes=false");
    }

    Ok(())
}
