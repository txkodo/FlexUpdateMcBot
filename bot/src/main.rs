use anyhow::Result;
use azalea_client::Account;
use azalea_protocol::ServerAddress;
use clap::Parser;

#[derive(Parser)]
#[command(name = "minecraft-bot")]
#[command(about = "A Minecraft bot that connects to servers using ViaVersion")]
struct Args {
    /// Username for the bot
    #[arg(short, long)]
    username: String,

    /// Server host address
    #[arg(short = 'H', long)]
    host: String,

    /// Server port
    #[arg(short, long)]
    port: u16,

    /// Minecraft version to use
    #[arg(short, long)]
    version: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!(
        "Connecting to {}:{} with username {} using Minecraft version {}",
        args.host, args.port, args.username, args.version
    );

    let account = Account::offline("bot");
    let mut client = account
        .join(&ServerAddress {
            host: args.host,
            port: args.port,
        })
        .await
        .unwrap();

    while let Some(e) = client.next().await {
        match e {
            _ => {}
        };
    }
    Ok(())
}
