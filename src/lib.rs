use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{StdoutLock, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<S, Payload> {
    fn from_init(state: S, init: Init) -> anyhow::Result<Self>
    where
        Self: Sized;

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

// We have different State Machines in Binary Crates
// That execute the main_loop with their States => Eg: Echo, UniqueIds
// Init is state is always executed
pub fn main_loop<S, N, P>(init_state: S) -> anyhow::Result<()>
where
    P: DeserializeOwned,
    N: Node<S, P>,
{
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    // Write Init Msg to the Output
    let init_msg: Message<InitPayload> =
        serde_json::from_reader(&mut stdin).context("init message should be deserialized")?;
    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("first message should be init");
    };

    let mut node: N = Node::from_init(init_state, init).context("node initialization failed")?;

    let reply = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_msg.body.id,
            payload: InitPayload::InitOk,
        },
    };

    serde_json::to_writer(&mut stdout, &reply).context("Serialize response to init")?;
    stdout.write_all(b"\n").context("write trailing newline")?;

    // Listen to stdin and write the Payload for that State
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<P>>();
    for input in inputs {
        let input = input.context("error deserializing input")?;
        node.step(input, &mut stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}
