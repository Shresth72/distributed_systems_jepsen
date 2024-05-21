#![allow(unused)]

use distributed_systems::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, fmt::Debug, io::StdoutLock, usize};

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

struct KafkaNode {
    node: String,
    id: usize,
    log_msgs: HashMap<String, Vec<Vec<usize>>>,
    committed_msgs: HashMap<String, usize>,
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
            log_msgs: HashMap::new(),
            committed_msgs: HashMap::new(),
        })
    }

    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("got injected event when there's no event injection");
        };

        let mut reply = input.into_reply(Some(&mut self.id));

        match reply.body.payload {
            Payload::Send { key, message } => {
                let entry = self
                    .log_msgs
                    .entry(key.clone())
                    .or_insert_with(|| Vec::new());
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

                reply.body.payload = Payload::SendOk { offset: new_offset };
                reply.send(&mut *output).context("reply to broadcast")?;
            }

            Payload::Poll { offsets } => {
                let log_msgs: HashMap<String, Vec<Vec<usize>>> = self
                    .log_msgs
                    .iter()
                    .filter_map(|(key, vecs)| {
                        if let Some(&start_offset) = offsets.get(key) {
                            let fvecs: Vec<Vec<usize>> = vecs
                                .iter()
                                .filter(|v| v[0] > start_offset)
                                .cloned()
                                .collect();

                            if !fvecs.is_empty() {
                                Some((key.clone(), fvecs))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect();

                reply.body.payload = Payload::PollOk { messages: log_msgs };
                reply.send(&mut *output).context("reply to broadcast")?;
            }

            Payload::CommitOffsets { offsets } => {
                for (key, value) in offsets {
                    self.committed_msgs.insert(key, value);
                }

                reply.body.payload = Payload::CommitOffsetsOk;
                reply.send(&mut *output).context("reply to broadcast")?;
            }

            Payload::ListCommittedOffsets { keys } => {
                let mut offsets = HashMap::new();
                for key in keys {
                    if let Some(offset) = self.committed_msgs.get(&key) {
                        offsets.insert(key, *offset);
                    }
                }

                reply.body.payload = Payload::ListCommittedOffsetsOk { offsets };
                reply.send(&mut *output).context("reply to broadcast")?;
            }

            _ => {}
        };

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, KafkaNode, _, _>(())
}
