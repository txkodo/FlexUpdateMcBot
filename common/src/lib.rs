use serde::{Deserialize, Serialize};

///! このボットは標準入出力でJSON RPCするのでその型定義

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StdinEvent {}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StdoutEvent {
    #[serde(rename = "spawn")]
    Spawn {},
    #[serde(rename = "disconnect")]
    Disconnect {
        reason: String,
    },
    #[serde(rename = "chunk")]
    Chunk {
        x: i32,
        z: i32,
    },
}

/// StdoutEventを標準出力用にシリアライズ
/// 改行は含まない
pub fn serialize_stdout_line(event: &StdoutEvent) -> Vec<u8> {
    serde_json::to_vec(event).unwrap()
}

/// StdinEventを標準入力からデシリアライズ
pub fn deserialize_stdin_line(line: &[u8]) -> Option<StdinEvent> {
    serde_json::from_slice(&line).ok()
}

pub struct Args {
    pub username: String,
    pub host: String,
    pub port: u16,
    pub view_distance: u8,
}

pub fn parse_args() -> Args {
    let mut args = pico_args::Arguments::from_env();
    let username: String = args
        .value_from_str("--username")
        .expect("--username is required");
    let host: String = args.value_from_str("--host").expect("--host is required");
    let port: u16 = args.value_from_str("--port").expect("--port is required");
    let view_distance: u8 = args
        .value_from_str("--view-distance")
        .expect("--view_distance must be u8");
    Args {
        username,
        host,
        port,
        view_distance,
    }
}
