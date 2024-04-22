use distributed_systems::*;

use anyhow::Context;
use rand::prelude::*;
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
                        let (already_known, mut notify_of): (HashSet<_>, HashSet<_>) = self
                            .messages
                            .iter()
                            .copied()
                            .partition(|m| !known_to_n.contains(m));

                        //------- SMARTER GOSSIP -------- //

                        /*
                        If we know that n knows m, we don't tell n that _we_ know m, so n will
                        send us m for all eternity. so, we include a couple of extra `m`s so
                        they gradually know all the things that we know without sending lots of
                        extra stuff each time.

                        We cap the number of extraneous `m`s we include to be at most 10% of the
                        number of `m`s` we _have_ to include to avoid excessive overhead
                        */

                        let mut rng = rand::thread_rng();
                        let additional_cap = (10 * notify_of.len() / 100) as u32;
                        notify_of.extend(already_known.iter().filter(|_| {
                            rng.gen_ratio(
                                additional_cap.min(already_known.len() as u32),
                                already_known.len() as u32,
                            )
                        }));

                        Message {
                            src: self.node.clone(),
                            dst: n.clone(),
                            body: Body {
                                id: None,
                                in_reply_to: None,
                                payload: Payload::Gossip { seen: notify_of },
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
                        self.known
                            .get_mut(&reply.dst)
                            .expect("got gossip from unknown node")
                            .extend(seen.iter().copied());
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
