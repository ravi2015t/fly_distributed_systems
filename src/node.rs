use super::protocol::*;
use anyhow::Context;
use anyhow::Result;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;

pub struct Node {
    node_id: String,
    node_ids: Vec<String>,
    topology: HashMap<String, Vec<String>>,
    messages: Messages,
}
pub struct Messages {
    kwn_msgs: HashSet<u32>,
    nxt_id: u32,
}
impl Messages {
    pub fn new() -> Self {
        Messages {
            kwn_msgs: HashSet::new(),
            nxt_id: 0,
        }
    }
    fn incr_msg_id(&mut self) {
        self.nxt_id += 1;
    }
}
impl Node {
    pub fn new(body: InitBody) -> Self {
        Node {
            node_id: body.node_id,
            node_ids: body.node_ids,
            topology: HashMap::new(),
            messages: Messages::new(),
        }
    }

    pub fn write(&self, message: Message) -> Result<()> {
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

    pub fn gossip(&mut self) -> Result<()> {
        let neighbors = self
            .topology
            .get(&self.node_id)
            .context("Topology for current node is not available")
            .unwrap();
        for node_id in neighbors {
            let gossip_body = MessageType::Gossip(GossipBody {
                messages: self.messages.kwn_msgs.clone(),
                msg_id: self.messages.nxt_id,
            });

            let gossip = Message {
                src: self.node_id.to_owned(),
                dest: node_id.to_owned(),
                body: gossip_body,
            };
            self.write(gossip)
                .context("Writing gossip message to stdout")?;
            self.messages.incr_msg_id();
        }
        Ok(())
    }

    pub fn respond_to_message(&mut self, message: Message) -> Result<()> {
        let response_body: Option<MessageType> = match message.body {
            MessageType::Broadcast(body) => {
                self.messages.kwn_msgs.insert(body.message);
                Some(MessageType::BroadcastOk(MessageResponseBody {
                    msg_id: self.messages.nxt_id,
                    in_reply_to: body.msg_id,
                }))
            }
            MessageType::Topology(body) => {
                self.topology = body.topology;
                Some(MessageType::TopologyOk(MessageResponseBody {
                    msg_id: self.messages.nxt_id,
                    in_reply_to: body.msg_id,
                }))
            }

            MessageType::Read(body) => Some(MessageType::ReadOk(ReadResponseBody {
                in_reply_to: body.msg_id,
                messages: self.messages.kwn_msgs.clone(),
            })),

            MessageType::Gossip(body) => {
                self.messages.kwn_msgs.extend(body.messages);

                Some(MessageType::GossipOk(GossipResponseBody {
                    in_reply_to: body.msg_id,
                }))
            }
            _ => None,
        };
        self.messages.incr_msg_id();
        if response_body.is_some() {
            let response = Message {
                src: message.dest,
                dest: message.src,
                body: response_body.unwrap(),
            };

            self.write(response).context("Writing response to stdout")?;
        }
        Ok(())
    }
}
