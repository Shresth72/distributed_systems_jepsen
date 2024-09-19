use anyhow::Context;
use distributed_systems::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::StdoutLock;
use std::sync::Arc;

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

/*
* network partition:  2  2n  70.44s user 10.20s system 190% cpu 42.384 total
* without partition:  2  2n  68.57s user 11.85s system 198% cpu 40.499 total
*
 *** --consistency-models read-uncommitted:
 - Specifies the consistency model for the test. read-uncommitted means that transactions can read data that has been written but not yet committed by other transactions. This allows for weaker consistency guarantees and is useful for testing systems that support eventual consistency.

 - Replaced Mutex with RwLock: We're now using a RwLock instead of a Mutex. This allows multiple readers to access the store simultaneously, which can improve performance in read-heavy scenarios.

 - Used parking_lot::RwLock: The parking_lot crate provides a more efficient implementation of RwLock than the standard library. This can lead to better performance under high concurrency.

 - Optimized transaction processing: Instead of modifying the input txn vector in-place, we now create a new updated_txn vector. This avoids potential issues with borrowing and allows for more efficient processing.

 - Improved error handling: We're now using the msg_id from the input message in the reply, which ensures correct message tracking.

 - Reduced locking scope: The locks are now held for shorter durations, which should reduce contention and improve responsiveness.

 - Used Arc for shared ownership: This allows for more efficient cloning of the store reference if needed in the future.
*/

struct TransactionNode {
    node: String,
    id: usize,
    store: Arc<RwLock<HashMap<usize, TxnElement>>>,
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
            store: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    fn step(&mut self, input: Event<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let Event::Message(input) = input else {
            panic!("got injected event when there's no event injection");
        };

        let mut reply = input.into_reply(Some(&mut self.id));

        if let Payload::Txn { ref mut txn } = reply.body.payload {
            let mut updated_txn = Vec::new();

            for operation in txn.iter() {
                match operation.as_slice() {
                    [TxnElement::Str(ref op), TxnElement::USize(key), TxnElement::None]
                        if op == "r" =>
                    {
                        let store = self.store.read();
                        let value = store.get(key).cloned().unwrap_or(TxnElement::None);
                        updated_txn.push(vec![
                            TxnElement::Str("r".to_string()),
                            TxnElement::USize(*key),
                            value,
                        ]);
                    }

                    [TxnElement::Str(ref op), TxnElement::USize(key), TxnElement::USize(value)]
                        if op == "w" =>
                    {
                        let mut store = self.store.write();
                        store.insert(*key, TxnElement::USize(*value));
                        updated_txn.push(operation.clone());
                    }

                    _ => {
                        updated_txn.push(operation.clone());
                    }
                }
            }

            reply.body.payload = Payload::TxnOk {
                msg_id: self.id,
                in_reply_to: reply.body.in_reply_to.expect(""),
                txn: updated_txn,
            };
            reply.send(&mut *output).context("reply to txn")?;
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, TransactionNode, _, _>(())
}
