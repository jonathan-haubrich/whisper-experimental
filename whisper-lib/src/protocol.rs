use prost::Message;

include!("codegen/whisper.protocol.rs");

impl TryFrom<crate::envelope::Envelope> for Request {
    type Error = std::io::Error;

    fn try_from(value: crate::envelope::Envelope) -> Result<Self, Self::Error> {
        let data = value.data;

        //let message = crate::request::Request::decode(data.as_slice())?;
        let request = Request::decode(data.as_slice())?;

        Ok(request)
    }
}

impl TryFrom<crate::envelope::Envelope> for Response {
    type Error = std::io::Error;

    fn try_from(value: crate::envelope::Envelope) -> Result<Self, Self::Error> {
        let data = value.data;

        //let message = crate::request::Request::decode(data.as_slice())?;
        let request = Response::decode(data.as_slice())?;

        Ok(request)
    }
}