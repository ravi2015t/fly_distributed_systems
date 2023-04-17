use serde::{Deserialize, Serialize};

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
pub struct InitBody {
    pub msg_id: u32,
    pub node_id: String,
    pub node_ids: Vec<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize, Eq, PartialEq)]
pub struct EchoResponseBody {
    pub msg_id: u32,
    pub in_reply_to: u32,
    pub echo: String,
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
}
