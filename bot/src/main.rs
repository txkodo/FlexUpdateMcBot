use anyhow::Result;
use azalea_client::{Account, Client};
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

    let (_client, mut event) = Client::join(
        &Account::offline(&args.username),
        ServerAddress {
            host: args.host,
            port: args.port,
        },
    )
    .await?;

    while let Some(e) = event.recv().await {
        match e {
            azalea_client::Event::Login => {
                println!(r#"{{"event":"login"}}"#);
            }
            _ => {}
        }
    }
    Ok(())
}
