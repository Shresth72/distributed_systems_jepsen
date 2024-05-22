#![allow(unused)]

use distributed_systems::*;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::StdoutLock};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum TxnElement {
    Str(String),
    USize(usize),
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Txn {
        msg_id: usize,
        txn: Vec<Vec<TxnElement>>,
    },
    TxnOk {
        msg_id: usize,
        in_reply_to: usize,
        txn: Vec<Vec<TxnElement>>,
    },
}

struct TransactionNode {
    node: String,
    id: usize,
    msgs: HashMap<usize, Vec<Vec<TxnElement>>>,
}

impl Node<(), Payload> for TransactionNode {
    fn from_init(
        _state: (),
        init: Init,
        _tx: std::sync::mpsc::Sender<Event<Payload>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(TransactionNode {
            id: 1,
            node: init.node_id,
            msgs: HashMap::new(),
        })
    }

    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("got injected event when there's no event injection");
        };

        let mut reply = input.into_reply(Some(&mut self.id));

        match reply.body.payload {
            Payload::Txn { msg_id, mut txn } => {
                for elem in &mut txn {
                    match elem.get_mut(0) {
                        Some(e) if *e == TxnElement::Str("r".to_string()) => {
                            elem[2] = TxnElement::USize(msg_id);
                        }
                        _ => {}
                    }
                }

                self.msgs.insert(msg_id, txn.clone());

                reply.body.payload = Payload::TxnOk {
                    msg_id: self.id,
                    in_reply_to: msg_id,
                    txn,
                };
                reply.send(&mut *output).context("reply to txn")?;
            }

            Payload::TxnOk { .. } => {}
        };

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, TransactionNode, _, _>(())
}
