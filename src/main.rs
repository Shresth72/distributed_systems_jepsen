fn main() -> anyhow::Result<()> {
    // TODO 1: Create the main worker to receive TCP requests or Create Messages
    // TODO 2: Spawm child processes from binaries
    // TODO 3: Pipe requests/messages from TCP to either
    // Binary Channels OR Stdin Lines
    Ok(())
}

/*
fn main() {
    let init = Init {
        node_id: String::from("node_1"),
        node_ids: vec![String::from("node_1"), String::from("node_2")],
    };

    let (tx, _rx) = std::sync::mpsc::channel();

    let mut node = GrowCounterNode::from_init((), init, tx).expect("Failed to initialize node");

    // Example event
    let event = Event::Message(Message {
        body: Body {
            payload: Payload::Add { delta: 5 },
        },
    });

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    node.step(event, &mut handle).expect("Failed to process event");

    // Print the global counter
    println!("Global counter value: {}", GLOBAL_COUNTER.load(Ordering::SeqCst));
}

*/
