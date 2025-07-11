use anyhow::Result;
use azalea_client::{Account, Client, ClientInformation, Event};
use azalea_protocol::{
    ServerAddress,
    common::client_information::{ChatVisibility, HumanoidArm, ModelCustomization, ParticleStatus},
    packets::game::ClientboundGamePacket,
};
use common::{StdoutEvent, serialize_stdout_line};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let args = common::parse_args();

    let (client, mut event) = Client::join(
        Account::offline(&args.username),
        ServerAddress {
            host: args.host,
            port: args.port,
        },
    )
    .await?;

    let mut stdout = io::stdout();

    while let Some(e) = event.recv().await {
        match e {
            Event::Spawn => {
                stdout.write(&serialize_stdout_line(&StdoutEvent::Spawn {}))?;
                stdout.write("\n".as_bytes())?;
            }
            Event::Disconnect(reason) => {
                stdout.write(&serialize_stdout_line(&StdoutEvent::Disconnect {
                    reason: reason
                        .map(|x| x.to_string())
                        .unwrap_or("unknown".to_string()),
                }))?;
                stdout.write("\n".as_bytes())?;
                break;
            }
            Event::Packet(packet) => match &*packet {
                ClientboundGamePacket::LevelChunkWithLight(packet) => {
                    stdout.write(&serialize_stdout_line(&StdoutEvent::Chunk {
                        x: packet.x,
                        z: packet.z,
                    }))?;
                    stdout.write("\n".as_bytes())?;
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(())
}
