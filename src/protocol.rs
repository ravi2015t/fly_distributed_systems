use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct EchoBody {
    pub msg_id: u32,
    pub echo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct BroadcastBody {
    pub message: u32,
    pub msg_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct InitBody {
    pub msg_id: u32,
    pub node_id: String,
    pub node_ids: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct TopologyBody {
    pub topology: HashMap<String, Vec<String>>,
    pub msg_id: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct ReadBody {
    pub msg_id: u32,
}
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EchoResponseBody {
    pub msg_id: u32,
    pub in_reply_to: u32,
    pub echo: String,
}
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct MessageResponseBody {
    pub msg_id: u32,
    pub in_reply_to: u32,
}
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct ReadResponseBody {
    pub messages: Vec<u32>,
    pub in_reply_to: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct InitResponseBody {
    pub in_reply_to: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum MessageType {
    Echo(EchoBody),
    EchoOk(EchoResponseBody),
    Init(InitBody),
    InitOk(InitResponseBody),
    Broadcast(BroadcastBody),
    BroadcastOk(MessageResponseBody),
    Read(ReadBody),
    ReadOk(ReadResponseBody),
    Topology(TopologyBody),
    TopologyOk(MessageResponseBody),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_init() {
        // {"src":"n0","dest":"n3","body":{"type":"init","msg_id":1,"node_id":"n3","node_ids":["n1","n2","n3"]}}
        let json_str= "{\"src\":\"n0\",\"dest\":\"n3\",\"body\":{\"type\":\"init\",\"msg_id\":1,\"node_id\":\"n3\",\"node_ids\":[\"n1\",\"n2\",\"n3\"]}}";

        let expected = Message {
            src: "n0".to_string(),
            dest: "n3".to_string(),
            body: MessageType::Init(InitBody {
                msg_id: 1,
                node_id: "n3".to_string(),
                node_ids: vec!["n1".to_string(), "n2".to_string(), "n3".to_string()],
            }),
        };

        let got: Message = serde_json::from_str(&json_str).unwrap();

        assert_eq!(expected, got);
    }

    #[test]
    fn check_echo() {
        // {"src":"n0","dest":"n3","body":{"type":"init","msg_id":1,"node_id":"n3","node_ids":["n1","n2","n3"]}}
        let json_str= "{\"src\":\"n0\",\"dest\":\"n3\",\"body\":{\"type\":\"echo\",\"msg_id\":1,\"echo\":\"echo message\"}}";

        let expected = Message {
            src: "n0".to_string(),
            dest: "n3".to_string(),
            body: MessageType::Echo(EchoBody {
                msg_id: 1,
                echo: "echo message".to_string(),
            }),
        };

        let got: Message = serde_json::from_str(&json_str).unwrap();

        assert_eq!(expected, got);
    }
    #[test]
    fn check_topology() {
        //{"src":"n0","dest":"n3","body":{"type":"broadcast","message":1}}

        // {"src":"n0","dest":"n3","body":{"type":"read"}}
        // {"id":2,"src":"c1","dest":"n0","body":{"type":"topology","topology":{"n0":[]},"msg_id":1}}
        let json_str= "{\"src\":\"n0\",\"dest\":\"n3\",\"body\":{\"type\":\"topology\",\"msg_id\":1,\"topology\":{\"n1\":[\"n2\",\"n3\"],\"n2\":[\"n1\"],\"n3\":[\"n1\"]}}}";
        let mut map = HashMap::new();
        map.insert("n1".to_string(), vec!["n2".to_string(), "n3".to_string()]);

        map.insert("n2".to_string(), vec!["n1".to_string()]);
        map.insert("n3".to_string(), vec!["n1".to_string()]);
        let expected = Message {
            src: "n0".to_string(),
            dest: "n3".to_string(),
            body: MessageType::Topology(TopologyBody {
                topology: map,
                msg_id: 1,
            }),
        };

        let got: Message = serde_json::from_str(&json_str).unwrap();

        assert_eq!(expected, got);
    }
    #[test]
    fn check_topology_empty() {
        let json_str = "{\"id\":2,\"src\":\"c1\",\"dest\":\"n0\",\"body\":{\"type\":\"topology\",\"topology\":{\"n0\":[]},\"msg_id\":1}}";
        let mut map = HashMap::new();
        map.insert("n0".to_string(), vec![]);

        let expected = Message {
            src: "c1".to_string(),
            dest: "n0".to_string(),
            body: MessageType::Topology(TopologyBody {
                topology: map,
                msg_id: 1,
            }),
        };

        let got: Message = serde_json::from_str(&json_str).unwrap();

        assert_eq!(expected, got);
    }
}
