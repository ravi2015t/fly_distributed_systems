use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use core::panic;
use fly_exercise::protocol::*;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

struct Node {
    node_id: String,
    node_ids: Vec<String>,
    topology: HashMap<String, Vec<String>>,
    messages: Vec<u32>,
}

impl Node {
    pub fn new(body: InitBody) -> Self {
        Node {
            node_id: body.node_id,
            node_ids: body.node_ids,
            topology: HashMap::new(),
            messages: vec![],
        }
    }

    fn write(&self, message: Message) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        let resp = serde_json::to_string(&message);

        handle
            .write_all(resp?.as_bytes())
            .context("writing response to stdout")?;
        handle.write_all(b"\n").context("writing new line chr")?;
        handle.flush().context("flushing it to the std out")?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let stdin = std::io::stdin();
    dbg!("Got init message");
    let mut first_message = String::new();
    stdin
        .lock()
        .read_line(&mut first_message)
        .context("reading first message from std input")?;

    let init_message: Message =
        serde_json::from_str(&first_message).context("converting stdin to message")?;

    let init_body = match init_message.body {
        MessageType::Init(body) => Some(body),
        _ => None,
    };
    if init_body.is_none() {
        panic!("First message should be Init")
    }
    let init_body = init_body.unwrap();

    let msg_id = init_body.msg_id;
    let node = Node::new(init_body);

    let response = Message {
        src: init_message.dest,
        dest: init_message.src,
        body: MessageType::InitOk(InitResponseBody {
            in_reply_to: msg_id,
        }),
    };
    dbg!("Got init message");
    node.write(response)
        .context("Writing init ok message to std out")?;

    let node = Arc::new(Mutex::new(node));
    let mut id = 2;
    let mut threads: Vec<thread::JoinHandle<std::result::Result<(), Error>>> = vec![];
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let json_str = line.unwrap();
        let node = Arc::clone(&node);
        let message: Message =
            serde_json::from_str(&json_str).context("converting stdin to message")?;
        threads.push(thread::spawn(move || {
            let mut node = node.lock().unwrap();
            let response_body: Option<MessageType> = match message.body {
                MessageType::Broadcast(body) => {
                    node.messages.push(body.message);
                    Some(MessageType::BroadcastOk(MessageResponseBody {
                        msg_id: id,
                        in_reply_to: body.msg_id,
                    }))
                }
                MessageType::Topology(body) => {
                    node.topology = body.topology;
                    Some(MessageType::TopologyOk(MessageResponseBody {
                        msg_id: id,
                        in_reply_to: body.msg_id,
                    }))
                }

                MessageType::Read(body) => Some(MessageType::ReadOk(ReadResponseBody {
                    in_reply_to: body.msg_id,
                    messages: node.messages.clone(),
                })),
                _ => None,
            };
            let response = Message {
                src: message.dest,
                dest: message.src,
                body: response_body.unwrap(),
            };

            node.write(response).context("Writing to stdout")?;
            Ok(())
        }));
        id += 1;
    }
    for thread in threads {
        thread.join().unwrap()?;
    }
    Ok(())
}
