use base64::prelude::*;
use libcommon::newerr;

pub struct Message {
    pub command: String,
    pub payload: Vec<u8>,
}

impl Message {
    pub fn new(command: impl ToString, payload: impl ToString) -> Self {
        Self {
            command: command.to_string(),
            payload: BASE64_STANDARD.encode(payload.to_string()).into(),
        }
    }
}

impl From<Message> for String {
    fn from(value: Message) -> Self {
        let mut s = String::new();
        s.push_str(&value.command);
        s.push('|');
        s.push_str(&BASE64_STANDARD.encode(value.payload));
        s
    }
}

impl TryFrom<&str> for Message {
    type Error = libcommon::prelude::Err;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let value = value.split("|").collect::<Vec<&str>>();
        if value.len() != 2 {
            return Err(newerr!("invalid message format"));
        }
        let command = value[0].trim().to_string();
        let payload = if value[1].is_empty() {
            vec![]
        } else {
            BASE64_STANDARD.decode(value[1].trim())?
        };
        Ok(Self { command, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let msg = Message {
            command: "test".to_string(),
            payload: "Ok".as_bytes().to_vec(),
        };

        let s = String::from(msg);
        assert_eq!(s, "test|T2s=");

        let msg: Result<Message, _> = s.as_str().try_into();
        assert!(msg.is_ok());
    }
}
