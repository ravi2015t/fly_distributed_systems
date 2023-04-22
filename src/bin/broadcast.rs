use anyhow::Context;
use anyhow::Error;
use anyhow::Result;
use core::panic;
use fly_exercise::node::*;
use fly_exercise::protocol::*;
use std::io::BufRead;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

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
                    let mut node = n.lock().unwrap();
                    node.gossip().context("Starting gossip")?;
                }
            }

            Ok(())
        });

    //read input from stdin and process events
    let mut threads: Vec<thread::JoinHandle<std::result::Result<(), Error>>> = vec![];
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let json_str = line.unwrap();
        let node = Arc::clone(&node);
        let message: Message =
            serde_json::from_str(&json_str).context("converting stdin to message")?;
        threads.push(thread::spawn(move || {
            let mut node = node.lock().unwrap();
            node.respond_to_message(message)
                .context("responding to an input message")?;
            Ok(())
        }));
    }
    for thread in threads {
        thread.join().unwrap()?;
    }

    tx.send(()).context("terminate gossip").unwrap();
    gossip_thread.join().unwrap()?;

    Ok(())
}
