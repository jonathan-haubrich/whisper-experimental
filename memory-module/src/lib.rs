#![allow(dead_code, unused_variables, unused_imports)]

mod allocator;
mod pe_file;
mod error;

use std::{marker::PhantomData, mem::transmute};

use allocator::ModuleAllocator;

use pe_file::PeFile;
use windows::Win32::System::{Diagnostics::Debug::IMAGE_SECTION_HEADER, SystemServices::{self, IMAGE_DOS_HEADER}};

use error::Result;

pub struct MemoryModule<T: ModuleAllocator> {
    allocator: PhantomData<T>,

    pefile: PeFile,
}

impl<'pe, T: ModuleAllocator> MemoryModule<T> {
    fn new(pe: Vec<u8>) -> Self {
        MemoryModule::<T>{
            allocator: PhantomData,
            pefile: PeFile::new(pe),
        }
    }

    fn load_library(&mut self) -> Result<bool>  {

        let dos_header = self.pefile.dos_header();
        println!("dos_header: {:x}", dos_header.e_magic);

        let _ = self.pefile.validate();

        for section in self.pefile.sections()? {
            println!("section.Name: {}", String::from_utf8(section.Name.to_vec()).unwrap());
        }

        let mem = T::mem_alloc(0x1000, None);

        dbg!(mem);
        println!("mem: {:?}", mem);

        if let Some(m) = mem {
            allocator::VirtualAlloc::mem_free(m);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use crate::{allocator::{self, ModuleAllocator}, MemoryModule};

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
        
        let dll = std::fs::read(r#"C:\Windows\System32\ntdll.dll"#).unwrap();
        
        let mut memory_module = MemoryModule::<allocator::VirtualAlloc>::new(dll);

        match memory_module.load_library() {
            Ok(_) => println!("Loaded successfully"),
            Err(error) => println!("Got an error :(  {error:#?}")
        }
    }
}
