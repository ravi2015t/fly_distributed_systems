use anyhow::Context;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
enum ResponseBody {
    EchoOk(EchoResponseBody),
    InitOk(InitResponseBody),
}

#[derive(Debug, Clone, Serialize)]
struct Response {
    src: String,
    dest: String,
    body: ResponseBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body {
    #[serde(rename = "type")]
    message_type: MessageType,
    #[serde(default)]
    msg_id: u32,
    #[serde(default)]
    echo: String,
}

#[derive(Debug, Clone, Serialize)]
struct EchoResponseBody {
    #[serde(rename = "type")]
    message_type: MessageType,
    msg_id: u32,
    in_reply_to: u32,
    echo: String,
}

#[derive(Debug, Clone, Serialize)]
struct InitResponseBody {
    #[serde(rename = "type")]
    message_type: MessageType,
    in_reply_to: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum MessageType {
    Echo,
    EchoOk,
    Init,
    InitOk,
}

fn main() -> Result<()> {
    let stdin = std::io::stdin();
    let mut id = 1;
    for line in stdin.lock().lines() {
        let json_str = line.unwrap();
        let message: Message =
            serde_json::from_str(&json_str).context("converting stdin to message")?;

        let response_body: Option<ResponseBody> = match message.body.message_type {
            MessageType::Init => Some(ResponseBody::InitOk(InitResponseBody {
                message_type: MessageType::InitOk,
                in_reply_to: message.body.msg_id,
            })),
            MessageType::Echo => Some(ResponseBody::EchoOk(EchoResponseBody {
                message_type: MessageType::EchoOk,
                msg_id: id,
                in_reply_to: message.body.msg_id,
                echo: message.body.echo,
            })),
            MessageType::EchoOk => None,
            MessageType::InitOk => None,
        };
        let response = Response {
            src: message.dest,
            dest: message.src,
            body: response_body.unwrap(),
        };

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        let resp = serde_json::to_string(&response);

        handle
            .write_all(resp?.as_bytes())
            .context("writing response to stdout")?;
        handle.write_all(b"\n").context("writing new line chr")?;
        handle.flush().context("flushing it to the std out")?;
        id += 1;
    }
    Ok(())
}
