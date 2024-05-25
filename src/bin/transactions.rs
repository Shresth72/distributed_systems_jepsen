use anyhow::Context;
use distributed_systems::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::StdoutLock, sync::Mutex};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
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
        txn: Vec<Vec<TxnElement>>,
    },
    TxnOk {
        msg_id: usize,
        in_reply_to: usize,
        txn: Vec<Vec<TxnElement>>,
    },
}

// {"src":"n1","dest":"n2","body":{"type":"init","node_id":"n1","node_ids":["n1","n2"]}}
// {"src":"n1","dest":"n1","body":{"type":"txn","msg_id":3,"txn":[["r",1,null],["w",1,6],["w",2,9]]}}
// {"src":"n1","dest":"n1","body":{"type":"txn","msg_id":3,"txn":[["w",8,2],["r",9,null],["r",9,null],["r",8,null]]}}

struct TransactionNode {
    _node: String,
    id: usize,
    store: Mutex<HashMap<usize, TxnElement>>,
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
            _node: init.node_id,
            store: Mutex::new(HashMap::new()),
        })
    }

    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("got injected event when there's no event injection");
        };

        let mut reply = input.into_reply(Some(&mut self.id));

        match reply.body.payload {
            Payload::Txn { ref mut txn } => {
                let mut store = self.store.lock().unwrap();
                for operation in txn.iter_mut() {
                    match operation.as_slice() {
                        [TxnElement::Str(ref op), TxnElement::USize(key), TxnElement::None]
                            if op == "r" =>
                        {
                            let value = store.get(key).cloned().unwrap_or(TxnElement::None);
                            *operation = vec![
                                TxnElement::Str("r".to_string()),
                                TxnElement::USize(*key),
                                value,
                            ];
                        }

                        [TxnElement::Str(ref op), TxnElement::USize(key), TxnElement::USize(value)]
                            if op == "w" =>
                        {
                            store.insert(*key, TxnElement::USize(*value));
                        }

                        _ => {}
                    }
                }
                drop(store);

                reply.body.payload = Payload::TxnOk {
                    msg_id: self.id,
                    in_reply_to: self.id,
                    txn: txn.clone(),
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
