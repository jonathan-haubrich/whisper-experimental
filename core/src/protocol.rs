use bytemuck::{AnyBitPattern, Contiguous, NoUninit, Zeroable};
use binrw::binrw;

#[binrw::binrw]
#[brw(big)]
#[repr(u8)]
#[derive(Clone, Copy, Default, Debug)]
pub(crate) enum Type {
    #[brw(magic = 0u8)]
    Load = 0,
    Command = 1,
    Response = 2,
    #[default] Invalid = 255,
}

#[binrw]
#[brw(big)]
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Header {
    pub r#type: Type,
    pub len: u64,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub(crate) struct Load {
    
    #[br(temp)]
    #[bw(calc = data.len() as u64)]
    pub len: u64,

    #[br(count = len)]
    pub data: Vec<u8>
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub(crate) struct Command {
    pub module_id: u64,

    pub id: u64,
    
    #[br(temp)]
    #[bw(calc = data.len() as u64)]
    pub len: u64,

    #[br(count = len)]
    pub data: Vec<u8>
}

#[binrw]
#[bw(big)]
pub(crate) struct Response {
    #[bw(calc = data.len() as u64)]
    pub len: u64,

    #[br(count = len)]
    pub data: Vec<u8>
}

pub(crate) enum Message {
    Load(Load),
    Command(Command),
}