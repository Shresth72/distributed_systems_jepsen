#![allow(unused)]

use distributed_systems::*;

use anyhow::Context;
use core::panic;
use serde::{de::value, Deserialize, Serialize};
use std::{
    io::StdoutLock,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Add { delta: usize },
    AddOk,
    Read,
    ReadOk { value: usize },
}

struct GrowCounterNode {
    node: String,
    id: usize,
}

impl Node<(), Payload> for GrowCounterNode {
    fn from_init(
        _state: (),
        init: Init,
        _tx: std::sync::mpsc::Sender<Event<Payload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(GrowCounterNode {
            id: 1,
            node: init.node_id,
        })
    }

    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input {
            Event::EOF => {}
            Event::Injected(..) => {
                panic!("got injected event when there's no event injection");
            }

            Event::Message(input) => {
                let mut reply = input.into_reply(Some(&mut self.id));

                match reply.body.payload {
                    Payload::Add { delta } => {
                        // Atomically increment the counter
                        // This is not global to all nodes and only within the node
                        let mut counter = GLOBAL_COUNTER.lock().unwrap();
                        counter.fetch_add(delta, Ordering::SeqCst);

                        reply.body.payload = Payload::AddOk;
                        reply
                            .send(&mut *output)
                            .context("reply to grow counter add")?;
                    }

                    Payload::Read => {
                        // Automatically read the current value of the counter
                        let counter = GLOBAL_COUNTER.lock().unwrap();
                        let current_value = counter.load(Ordering::SeqCst);

                        reply.body.payload = Payload::ReadOk {
                            value: current_value,
                        };
                        reply
                            .send(&mut *output)
                            .context("reply to grow counter read")?;
                    }

                    Payload::AddOk | Payload::ReadOk { .. } => {}
                }
            }
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, GrowCounterNode, _, _>(())
}
