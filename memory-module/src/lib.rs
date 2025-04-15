#![allow(dead_code, unused_variables, unused_imports)]

mod allocator;
mod pe_file;
mod error;
mod memory_module;

use std::{marker::PhantomData, mem::transmute};

use allocator::ModuleAllocator;

use pe_file::PeFile;
use windows::Win32::System::{Diagnostics::Debug::IMAGE_SECTION_HEADER, SystemServices::{self, IMAGE_DOS_HEADER}};

use memory_module::{Error, MemoryModule};


#[cfg(test)]
mod tests {
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
        
        let dll = std::fs::read(r#"C:\Windows\System32\ws2_32.dll"#).unwrap();
        
        let mut memory_module = MemoryModule::<allocator::VirtualAlloc>::new(dll);

        match memory_module.load_library() {
            Ok(_) => println!("Loaded successfully"),
            Err(error) => println!("Got an error :(  {error:#?}")
        }
    }
}
