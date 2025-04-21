use binrw::BinRead;

#[derive(BinRead)]
#[br(big)]
pub(crate) enum Message {
    #[br(magic(0u8))] Load{
        len: u32,
        #[br(count = len)]
        message: Vec<u8>
    },
    #[br(magic(1u8))] Command{
        id: u32,
        args_len: u32,
        #[br(count = args_len)]
        args: Vec<u8>
    }
}

