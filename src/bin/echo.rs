use distributed_systems::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

// State Machine for the ECHO Node of the Distributed System
struct EchoNode {
    id: usize,
}

// Echo Message RPC: {"src":"n1","dest":"n2","body":{"type":"echo","msg_id":1,"echo":"Hello World"}}
impl Node<(), Payload> for EchoNode {
    fn from_init(
        _state: (),
        _init: Init,
        _tx: std::sync::mpsc::Sender<Event<Payload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(EchoNode { id: 1 })
    }

    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("got injected event when there's no event injection");
        };

        let mut reply = input.into_reply(Some(&mut self.id));

        match reply.body.payload {
            Payload::Echo { echo } => {
                reply.body.payload = Payload::EchoOk { echo };

                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to init")?;
                output.write_all(b"\n").context("write trailing newline")?;
            }
            Payload::EchoOk { .. } => {}
        };

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, EchoNode, _, _>(())
}
