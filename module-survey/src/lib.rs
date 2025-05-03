use std::str::FromStr;

use pmr;
use windows::{core::PSTR, Win32::System::WindowsProgramming::{GetComputerNameA, MAX_COMPUTERNAME_LENGTH}};

trait Surveyor {
    fn hostname() -> String;
}

struct Survey {}

#[pmr::dispatcher]
impl Surveyor for Survey {
    fn hostname() -> String {
        println!("in get_hostname");
        const COMPUTER_NAME_BUF_CAPACITY: usize = (MAX_COMPUTERNAME_LENGTH + 1) as usize;
        let mut computer_name_buf = [0u8; COMPUTER_NAME_BUF_CAPACITY];
        let mut written = (computer_name_buf.len() - 1) as u32;

        match unsafe { GetComputerNameA(Some(PSTR::from_raw(computer_name_buf.as_mut_ptr())), &mut written) } {
            Ok(_) => {
                let computer_name = unsafe { String::from_utf8_unchecked(computer_name_buf.to_vec()) };
                println!("GetComputerName returned: {computer_name} ({written})");
                computer_name
            },
            Err(err) => {
                println!("GetComputerName failed: {err}");
                String::from("UNKNOWN")
            }
        }
    }
}
