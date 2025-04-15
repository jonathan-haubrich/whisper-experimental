
use std::{marker::PhantomData, os::raw::c_void};
use crate::{ModuleAllocator, PeFile };

///
/// Steps to load:
/// 1. alloc memory
/// 2. copy structs to correct alignment
/// 3. fix up iat
/// 4. do relocs
/// 5. fix permissions
/// 6. TODO: do tls
/// 7. call entry point

use thiserror::Error;
use windows::Win32::{Foundation::{GetLastError, WIN32_ERROR}, System::{Diagnostics::Debug::{self, IMAGE_DIRECTORY_ENTRY_IMPORT}, Memory::{VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE}, SystemServices::IMAGE_DOS_HEADER}};
use crate::allocator;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid pe data")]
    PeInvalid,

    #[error("invalid section")]
    PeSectionInvalid,

    #[error("win32 error")]
    Win32Error(WIN32_ERROR),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct MemoryModule<T: ModuleAllocator> {
    allocator: PhantomData<T>,

    pefile: PeFile,
    memory: Option<*mut c_void>,
}

impl<'pe, T: ModuleAllocator> MemoryModule<T> {
    pub fn new(pe: Vec<u8>) -> Self {
        MemoryModule::<T>{
            allocator: PhantomData,
            pefile: PeFile::new(pe),
            memory: None,
        }
    }

    pub fn load_library(&mut self) -> Result<bool>  {

        let dos_header = self.pefile.dos_header();
        println!("dos_header: {:x}", dos_header.e_magic);

        let _ = self.pefile.validate();

        for section in self.pefile.sections()? {
            println!("section.Name: {}", String::from_utf8(section.Name.to_vec()).unwrap());
        }

        self.map_structs()?;

        self.resolve_imports()?;

        let mem = T::mem_alloc(0x1000, None);

        dbg!(mem);
        println!("mem: {:?}", mem);

        if let Some(m) = mem {
            allocator::VirtualAlloc::mem_free(m);
        }

        Ok(true)
    }

    fn map_structs(&mut self) -> Result<()> {
        let nt_header = self.pefile.nt_header();

        let size_of_image = nt_header.OptionalHeader.SizeOfImage as usize; 

        // try at preferred base first
        let mem = unsafe {
            VirtualAlloc(Some(nt_header.OptionalHeader.ImageBase as *const c_void), 
                size_of_image,
                MEM_RESERVE | MEM_COMMIT,
                PAGE_READWRITE)
        };

        if mem.is_null() {
            // fallback to whatever
            let mem = unsafe {
                VirtualAlloc(None, 
                    size_of_image,
                    MEM_RESERVE | MEM_COMMIT,
                    PAGE_READWRITE)
            };
        }

        // if still null, can't continue
        if mem.is_null() {
            return Err(Error::Win32Error(unsafe { GetLastError()}));
        }

        unsafe { std::ptr::write_bytes(mem, 0, size_of_image) };
        self.memory = Some(mem);

        unsafe { std::ptr::copy_nonoverlapping(self.pefile.data.as_ptr() as *const c_void,
            mem, 
            nt_header.OptionalHeader.SizeOfHeaders as usize) };

        for (i, section) in self.pefile.sections()?.iter().enumerate() {
            let section_data = self.pefile.section_data(i)?;

            let virtual_address = section.VirtualAddress as usize;

            println!("section: {}\n\tvirtual address: 0x{:x}\n\tsection_data size: 0x{:x}\n\tsection_data raw size: 0x{:x}\n\tsection virtual size: 0x{:x}",
                String::from_utf8(section.Name.to_vec()).unwrap(),
                section.VirtualAddress,
                section_data.len(),
                section.SizeOfRawData,
                unsafe { section.Misc.PhysicalAddress },
            );

            let dst = unsafe { mem.add(virtual_address) as *mut u8 };
            unsafe { std::ptr::copy_nonoverlapping(section_data.as_ptr(), dst, section_data.len()) }
        }

        Ok(())
    }

    fn resolve_imports(&mut self) -> Result<()> {

        let nt_header = self.pefile.nt_header();
        let Some(image_data_directory) = nt_header.OptionalHeader.DataDirectory.get(IMAGE_DIRECTORY_ENTRY_IMPORT.0 as usize) else {
            return Err(Error::PeInvalid);
        };

        println!("image_data_directory.Size: 0x{:x}", image_data_directory.Size);
        println!("image_data_directory.VirtualAddress: 0x{:x}", image_data_directory.VirtualAddress);

        Ok(())
    }
}