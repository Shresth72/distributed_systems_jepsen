use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug)]
enum ServerRequest {
    StoreMessage { key: String, message: usize },
    RetrieveMessage { offsets: HashMap<String, usize> },
    CommitOffsets { offsets: HashMap<String, usize> },
    ListCommittedOffsets { keys: Vec<String> },
}

#[derive(Serialize, Deserialize, Debug)]
enum ServerResponse {
    StoreResponseOk {
        offset: usize,
    },
    RetrieveMessageOk {
        messages: HashMap<String, Vec<Vec<usize>>>,
    },
    CommitOffsetsOk,
    ListCommittedOffsetsOk {
        offsets: HashMap<String, usize>,
    },
}

fn handle_client(
    mut stream: TcpStream,
    log_msgs: &mut HashMap<String, Vec<Vec<usize>>>,
    committed_msgs: &mut HashMap<String, usize>,
) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    let amt = stream.read(&mut buf)?;

    let request: ServerRequest =
        serde_json::from_slice(&buf[..amt]).unwrap_or_else(|_| ServerRequest::StoreMessage {
            key: String::new(),
            message: 0,
        });

    let response: ServerResponse = match request {
        ServerRequest::StoreMessage { key, message } => {
            let entry = log_msgs.entry(key.clone()).or_insert_with(|| Vec::new());
            let new_offset = match entry.iter().map(|v| v[0]).max() {
                Some(max_offset) => max_offset + 1,
                None => {
                    let num_start = key.find(char::is_numeric).unwrap();
                    let num_str = &key[num_start..];
                    let num: usize = num_str.parse().unwrap();
                    let offset = num * 1000;
                    offset
                }
            };

            let msg = vec![new_offset, message];
            entry.push(msg);

            ServerResponse::StoreResponseOk { offset: new_offset }
        }

        ServerRequest::RetrieveMessage { offsets } => {
            let mut messages = HashMap::new();
            for (key, start_offset) in offsets {
                if let Some(vecs) = log_msgs.get(&key) {
                    let fmsgs: Vec<Vec<usize>> = vecs
                        .iter()
                        .filter(|v| v[0] > start_offset)
                        .cloned()
                        .collect();
                    messages.insert(key, fmsgs);
                } else {
                    messages.insert(key, vec![]);
                }
            }

            ServerResponse::RetrieveMessageOk { messages }
        }

        ServerRequest::CommitOffsets { offsets } => {
            for (key, offset) in offsets {
                committed_msgs.insert(key, offset);
            }
            ServerResponse::CommitOffsetsOk
        }

        ServerRequest::ListCommittedOffsets { keys } => {
            let mut offsets = HashMap::new();
            for key in keys {
                if let Some(offset) = committed_msgs.get(&key) {
                    offsets.insert(key, *offset);
                }
            }
            ServerResponse::ListCommittedOffsetsOk { offsets }
        }
    };

    let response_bytes = serde_json::to_vec(&response).expect("Failed to serialize response");
    stream.write_all(&response_bytes)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5141")?;
    let mut log_msgs: HashMap<String, Vec<Vec<usize>>> = HashMap::new();
    let mut committed_msgs: HashMap<String, usize> = HashMap::new();

    println!("TCP Server listening on 127.0.0.1:5141");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let start_time = Instant::now();
                let log_msgs = &mut log_msgs;
                let committed_msgs = &mut committed_msgs;
                handle_client(stream, log_msgs, committed_msgs)?;
                let duration = start_time.elapsed();
                println!("Time consumed by request: {:?}", duration);
            }
            Err(e) => {
                println!("Error occurred: {}", e);
            }
        }
    }

    Ok(())
}
