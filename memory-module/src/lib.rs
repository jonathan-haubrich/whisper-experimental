#![allow(dead_code, unused_variables, unused_imports)]

pub mod pe_file;
pub mod error;
pub mod allocator;
mod memory_module;
pub use memory_module::*;

use std::{marker::PhantomData, mem::transmute};

use allocator::ModuleAllocator;

use pe_file::PeFile;
use windows::Win32::System::{Diagnostics::Debug::IMAGE_SECTION_HEADER, SystemServices::{self, IMAGE_DOS_HEADER}};

pub type FnDispatch = unsafe extern "C" fn(id: usize, arg_ptr: *mut u8, arg_len: usize, ret_ptr: &mut *mut u8, ret_len: &mut usize);

#[cfg(test)]
mod tests {
    use windows::{core::PCSTR, Win32::System::LibraryLoader::GetProcAddress};

    use crate::{allocator::{self, ModuleAllocator}, memory_module::MemoryModule};

    #[test]
    fn test_alloc() {
        let mem = allocator::VirtualAlloc::mem_alloc(0x1000, None);

        dbg!(mem);
        println!("mem: {:?}", mem);

        if let Some(m) = mem {
            allocator::VirtualAlloc::mem_free(m);
        }
    }

    #[test]
    fn test_load_library() {
        pub type FnDispatch = unsafe extern "C" fn(id: usize, arg_ptr: *mut u8, arg_len: usize, ret_ptr: &mut *mut u8, ret_len: &mut usize);
        println!("cwd: {:#?}", std::path::absolute(".").unwrap());

        let dll = std::fs::read(r#"..\target\debug\pmr_dll.dll"#).unwrap();
        
        let mut memory_module = MemoryModule::<allocator::VirtualAlloc>::new(dll);

        match memory_module.load_library() {
            Ok(_) => println!("Loaded successfully"),
            Err(err) => panic!("Got an error :(  {err:#?}"),
        }

        match memory_module.call_entry_point() {
            Ok(_) => println!("Called successfully!"),
            Err(err) => panic!("Nope :( {err:#?}"),
        }

        let hmodule = memory_module.hmodule().unwrap();

        println!("hmodule: {:#?}", hmodule);

        let Some(dispatch_ptr) = memory_module.get_proc_address("dispatch") else {
            panic!("Couldn't find dispatch");
        };

        println!("dispatch_ptr: {:#?}", dispatch_ptr);
        let dispatch_ptr: FnDispatch = unsafe { std::mem::transmute(dispatch_ptr) };

        let mut input: Vec<u8> = Vec::new();

        let mut output: *mut u8 = std::ptr::null_mut();
        let mut output_len = 0usize;
        unsafe { dispatch_ptr(0, input.as_mut_ptr(), 0, &mut output, &mut output_len) };

    }
}
