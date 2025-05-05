use prost::Message;


impl TryFrom<crate::Envelope> for crate::protocol::Msg {
    type Error = std::io::Error;

    fn try_from(value: crate::Envelope) -> Result<Self, Self::Error> {
        let data = value.data;

        let message = crate::Protocol::decode(data.as_slice())?;

        let Some(msg) = message.msg else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "no `msg` found after decoding... unknown error"));
        };

        Ok(msg)
    }
}