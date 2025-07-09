use anyhow::Result;
use azalea_client::{Account, Client};
use azalea_protocol::ServerAddress;
use std::env;

struct Args {
    username: String,
    host: String,
    port: u16,
}

impl Args {
    fn parse() -> Result<Self> {
        let args: Vec<String> = env::args().collect();

        if args.len() != 5 {
            eprintln!("Usage: {} <username> <host> <port>", args[0]);
            std::process::exit(1);
        }

        let port = args[3]
            .parse::<u16>()
            .map_err(|_| anyhow::anyhow!("Invalid port number: {}", args[3]))?;

        Ok(Args {
            username: args[1].clone(),
            host: args[2].clone(),
            port
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse()?;

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
