use base64::prelude::*;
use libcommon::newerr;

pub struct Message {
    pub command: String,
    pub payload: Vec<u8>,
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

impl TryFrom<String> for Message {
    type Error = libcommon::prelude::Err;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.split("|").collect::<Vec<&str>>();
        if value.len() != 2 {
            return Err(newerr!("invalid message format"));
        }
        let command = value[0].trim().to_string();
        let payload = BASE64_STANDARD.decode(value[1].trim())?;
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

        let msg: Result<Message, _> = s.try_into();
        assert!(msg.is_ok());
    }
}
