#![allow(unused)]

use distributed_systems::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Debug,
    io::{self, StdoutLock},
    net::UdpSocket,
    usize,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Send {
        key: String,
        #[serde(rename = "msg")]
        message: usize,
    },
    SendOk {
        offset: usize,
    },
    Poll {
        offsets: HashMap<String, usize>,
    },
    PollOk {
        #[serde(rename = "msgs")]
        messages: HashMap<String, Vec<Vec<usize>>>,
    },
    CommitOffsets {
        offsets: HashMap<String, usize>,
    },
    CommitOffsetsOk,
    ListCommittedOffsets {
        keys: Vec<String>,
    },
    ListCommittedOffsetsOk {
        offsets: HashMap<String, usize>,
    },
}

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

struct KafkaNode {
    node: String,
    id: usize,
}

impl Node<(), Payload> for KafkaNode {
    fn from_init(
        _state: (),
        init: Init,
        _tx: std::sync::mpsc::Sender<Event<Payload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(KafkaNode {
            id: 1,
            node: init.node_id,
        })
    }

    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("got injected event when there's no event injection");
        };

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect("127.0.0.1:5140")?;

        let mut reply = input.into_reply(Some(&mut self.id));

        match reply.body.payload {
            Payload::Send { key, message } => {
                let request = ServerRequest::StoreMessage {
                    key: key.clone(),
                    message,
                };
                self.send_request(&request, &socket)?;
                let response: ServerResponse = self.receive_response(&socket)?;

                if let ServerResponse::StoreResponseOk { offset } = response {
                    reply.body.payload = Payload::SendOk { offset };

                    reply.send(&mut *output).context("reply to send")?;
                }
            }

            Payload::Poll { offsets } => {
                let request = ServerRequest::RetrieveMessage { offsets };
                self.send_request(&request, &socket)?;
                let response: ServerResponse = self.receive_response(&socket)?;

                if let ServerResponse::RetrieveMessageOk { messages } = response {
                    reply.body.payload = Payload::PollOk { messages };
                    reply.send(&mut *output).context("reply to poll")?;
                }
            }

            Payload::CommitOffsets { offsets } => {
                let request = ServerRequest::CommitOffsets { offsets };
                self.send_request(&request, &socket)?;

                let response: ServerResponse = self.receive_response(&socket)?;

                if let ServerResponse::CommitOffsetsOk = response {
                    reply.body.payload = Payload::CommitOffsetsOk;
                    reply
                        .send(&mut *output)
                        .context("reply to commit_offsets")?;
                }
            }

            Payload::ListCommittedOffsets { keys } => {
                let request = ServerRequest::ListCommittedOffsets { keys };
                self.send_request(&request, &socket)?;

                let response: ServerResponse = self.receive_response(&socket)?;

                if let ServerResponse::ListCommittedOffsetsOk { offsets } = response {
                    reply.body.payload = Payload::ListCommittedOffsetsOk { offsets };
                    reply
                        .send(&mut *output)
                        .context("reply to list_committed_offsets")?;
                }
            }

            _ => {}
        };

        Ok(())
    }
}

impl KafkaNode {
    fn send_request(&self, request: &ServerRequest, socket: &UdpSocket) -> io::Result<()> {
        let request_bytes = serde_json::to_vec(&request).expect("Failed to serialize request");
        socket.send(&request_bytes)?;

        Ok(())
    }

    fn receive_response(&self, socket: &UdpSocket) -> io::Result<ServerResponse> {
        let mut buf = [0u8; 4096];
        let (amt, _) = socket.recv_from(&mut buf)?;

        // Attempt to deserialize the response
        let response: Result<ServerResponse, _> = serde_json::from_slice(&buf[..amt]);

        match response {
            Ok(res) => Ok(res),
            Err(e) => {
                // Print the raw bytes and string representation on error
                let raw_response = String::from_utf8_lossy(&buf[..amt]);
                println!(
                    "Failed to deserialize response: {:?} 
                    | Raw Response bytes: {:?} 
                    | Raw Response String: {:?}",
                    e,
                    &buf[..amt],
                    raw_response,
                );
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to deserialize response",
                ))
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, KafkaNode, _, _>(())
}
