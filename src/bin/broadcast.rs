use distributed_systems::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::StdoutLock,
    time::Duration,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Braodcast {
        message: usize,
    },
    BraodcastOk,
    Read,
    ReadOk {
        message: HashSet<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Gossip {
        seen: HashSet<usize>,
    },
}

enum InjectedPayload {
    Gossip,
}

// Gossip Protocol Optimization
struct BroadcastNode {
    node: String,
    id: usize,
    messages: HashSet<usize>,
    neighborhood: Vec<String>,

    // Node Identifier -> Messages this Node knows that the other Nodes know
    known: HashMap<String, HashSet<usize>>,
}

impl Node<(), Payload, InjectedPayload> for BroadcastNode {
    fn from_init(
        _state: (),
        init: Init,
        tx: std::sync::mpsc::Sender<Event<Payload, InjectedPayload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        std::thread::spawn(move || {
            // generate gossip events
            loop {
                std::thread::sleep(Duration::from_millis(300));
                if let Err(_) = tx.send(Event::Injected(InjectedPayload::Gossip)) {
                    break;
                }
            }
        });

        Ok(Self {
            id: 1,
            node: init.node_id,
            messages: HashSet::new(),
            neighborhood: Vec::new(),

            known: init
                .node_ids
                .into_iter()
                .map(|nid| (nid, HashSet::new()))
                .collect(),
        })
    }

    fn step(
        &mut self,
        input: Event<Payload, InjectedPayload>,
        output: &mut StdoutLock,
    ) -> anyhow::Result<()> {
        match input {
            Event::EOF => {}

            Event::Injected(payload) => match payload {
                InjectedPayload::Gossip => {
                    for n in &self.neighborhood {
                        let known_to_n = &self.known[n];
                        Message {
                            src: self.node.clone(),
                            dst: n.clone(),
                            body: Body {
                                id: None,
                                in_reply_to: None,
                                payload: Payload::Gossip {
                                    seen: self
                                        .messages
                                        .iter()
                                        .copied()
                                        .filter(|m| !known_to_n.contains(m))
                                        .collect(),
                                },
                            },
                        }
                        .send(&mut *output)
                        .with_context(|| format!("gossip to {}", n))?;
                        self.id += 1;
                    }
                }
            },

            Event::Message(input) => {
                let mut reply = input.into_reply(Some(&mut self.id));

                match reply.body.payload {
                    Payload::Gossip { seen } => {
                        self.messages.extend(seen);
                    }

                    Payload::Braodcast { message } => {
                        self.messages.insert(message);
                        reply.body.payload = Payload::BraodcastOk;

                        reply.send(&mut *output).context("reply to broadcast")?;
                    }

                    Payload::Read => {
                        reply.body.payload = Payload::ReadOk {
                            message: self.messages.clone(),
                        };

                        reply.send(&mut *output).context("reply to read")?;
                    }

                    Payload::Topology { mut topology } => {
                        self.neighborhood = topology
                            .remove(&self.node)
                            .unwrap_or_else(|| panic!("no topology given for node {}", self.node));
                        reply.body.payload = Payload::TopologyOk;

                        reply.send(&mut *output).context("reply to topology")?;
                    }

                    Payload::BraodcastOk | Payload::ReadOk { .. } | Payload::TopologyOk => {}
                };
            }
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _, _>(())
}
