
use std::{ffi::CString, marker::PhantomData, os::raw::c_void};
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
use windows::{core::PCSTR, Win32::{Foundation::{GetLastError, HMODULE, WIN32_ERROR}, System::{Diagnostics::Debug::{self, IMAGE_DIRECTORY_ENTRY_IMPORT}, LibraryLoader::{GetProcAddress, LoadLibraryA}, Memory::{VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE}, SystemServices::{IMAGE_DOS_HEADER, IMAGE_IMPORT_BY_NAME, IMAGE_IMPORT_DESCRIPTOR, IMAGE_ORDINAL_FLAG64}}}};
use crate::allocator;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid pe data")]
    PeInvalid,

    #[error("invalid section")]
    PeSectionInvalid,

    #[error("rva out of bounds")]
    RvaOutOfBounds,

    #[error("win32 error")]
    Win32Error(WIN32_ERROR),

    #[error("win32 api error")]
    Win32ApiError(#[from] windows::core::Error)
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

        
        let image_import_descriptors = unsafe { self.pefile.data.as_ptr().add(image_data_directory.VirtualAddress as usize) as *const IMAGE_IMPORT_DESCRIPTOR };
        let mut current_image_import_descriptor = 0;
        loop {

            let image_import_descriptor: &IMAGE_IMPORT_DESCRIPTOR = unsafe { &*image_import_descriptors.add(current_image_import_descriptor) };

            if image_import_descriptor.FirstThunk == 0 {
                break;
            }

            let import_name_rva = image_import_descriptor.Name as usize;  
            let import_name = self.pefile.get_cstring_at_rva(import_name_rva)?;
    
            println!("Import name: {}", import_name);
            
            let original_first_thunk_rva = unsafe { image_import_descriptor.Anonymous.OriginalFirstThunk };

            let thunks_ptr = unsafe { self.pefile.data.as_ptr().add(original_first_thunk_rva as usize) as *mut u64};
            let mut current_thunk = 0;

            let module = unsafe { LoadLibraryA(PCSTR::from_raw(self.pefile.data.as_ptr().add(import_name_rva))) }?;

            loop {
                let thunk_ptr = unsafe { thunks_ptr.add(current_thunk) };
                let thunk = unsafe { *thunk_ptr };

                if thunk == 0 {
                    break;
                }

                let proc_address ;
                if thunk & IMAGE_ORDINAL_FLAG64 as u64 == IMAGE_ORDINAL_FLAG64 {
                    let ordinal = thunk & 0xFFFF;
                    println!("\tFunction imported by ordinal: {}", ordinal);
                    proc_address = unsafe { GetProcAddress(module, PCSTR::from_raw(ordinal as *const u8)).unwrap() }
                } else {
                    let image_import_by_name_rva = thunk & (!IMAGE_ORDINAL_FLAG64);
                    let image_import_by_name: &IMAGE_IMPORT_BY_NAME = self.pefile.get_struct_at_rva(image_import_by_name_rva as usize)?;

                    let import_name = self.pefile.get_cstring_at_rva((image_import_by_name_rva + 2) as usize)?;
                    proc_address = unsafe { GetProcAddress(module, PCSTR::from_raw(import_name.as_ptr())).unwrap() };

                    println!("\tFunction imported by name: {}", import_name);
                }

                println!("\t\t...setting address to: 0x{:x}", proc_address as u64);
                unsafe { *thunk_ptr = proc_address as u64 };

                current_thunk += 1;
            }

            current_image_import_descriptor += 1;
        }


        Ok(())
    }
}