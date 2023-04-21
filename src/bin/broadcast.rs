use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use core::panic;
use fly_exercise::protocol::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::BufRead;
use std::io::Write;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

struct Node {
    node_id: String,
    node_ids: Vec<String>,
    topology: HashMap<String, Vec<String>>,
    messages: HashSet<u32>,
}

impl Node {
    pub fn new(body: InitBody) -> Self {
        Node {
            node_id: body.node_id,
            node_ids: body.node_ids,
            topology: HashMap::new(),
            messages: HashSet::new(),
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
    node.write(response)
        .context("Writing init ok message to std out")?;

    let node = Arc::new(Mutex::new(node));

    //thread to take care of gossip

    let mut id = 2;
    let n = Arc::clone(&node);
    let (tx, rx) = channel();
    let gossip_thread: thread::JoinHandle<std::result::Result<(), Error>> =
        thread::spawn(move || {
            let park_timeout = Duration::from_millis(500);
            let mut terminated = false;

            while !terminated {
                if rx.try_recv().is_ok() {
                    terminated = true;
                } else {
                    thread::park_timeout(park_timeout);

                    let node = n.lock().unwrap();
                    let neighbors = node
                        .topology
                        .get(&node.node_id)
                        .context("Topology for current node is not available")
                        .unwrap();
                    for node_id in neighbors {
                        let gossip_body = MessageType::Gossip(GossipBody {
                            messages: node.messages.clone(),
                            msg_id: id,
                        });

                        let gossip = Message {
                            src: node.node_id.to_owned(),
                            dest: node_id.to_owned(),
                            body: gossip_body,
                        };
                        node.write(gossip)
                            .context("Writing gossip message to stdout")?;
                        id += 1;
                    }
                }
            }

            Ok(())
        });

    //read input from stdin and process events
    let mut threads: Vec<thread::JoinHandle<std::result::Result<(), Error>>> = vec![];
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let json_str = line.unwrap();
        // println!("Message received on std in {}", &json_str);
        let node = Arc::clone(&node);
        let message: Message =
            serde_json::from_str(&json_str).context("converting stdin to message")?;
        threads.push(thread::spawn(move || {
            let mut node = node.lock().unwrap();
            let response_body: Option<MessageType> = match message.body {
                MessageType::Broadcast(body) => {
                    node.messages.insert(body.message);
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

                MessageType::Gossip(body) => {
                    node.messages.extend(body.messages);

                    Some(MessageType::GossipOk(GossipResponseBody {
                        in_reply_to: body.msg_id,
                    }))
                }
                _ => None,
            };
            if response_body.is_some() {
                let response = Message {
                    src: message.dest,
                    dest: message.src,
                    body: response_body.unwrap(),
                };

                node.write(response).context("Writing response to stdout")?;
            }
            Ok(())
        }));
        id += 1;
    }
    for thread in threads {
        thread.join().unwrap()?;
    }

    tx.send(()).context("terminate gossip").unwrap();
    gossip_thread.join().unwrap()?;

    Ok(())
}
