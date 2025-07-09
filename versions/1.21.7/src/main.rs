use azalea::prelude::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let account = Account::offline("bot");
    
    azalea::ClientBuilder::new()
        .set_handler(handle)
        .start(account, "localhost")
        .await?;
        
    Ok(())
}

async fn handle(bot: Client, event: Event, state: State) -> anyhow::Result<()> {
    match event {
        Event::Login => {
            println!("Bot logged in!");
        }
        Event::Chat(m) => {
            println!("Chat: {}", m.message().to_ansi());
        }
        _ => {}
    }
    
    Ok(())
}