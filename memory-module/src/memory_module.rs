
use std::{ffi::CString, marker::PhantomData, os::raw::c_void, ptr::addr_eq};
use crate::{memory_module, pe_file, ModuleAllocator, PeFile };

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
use windows::{core::PCSTR, Win32::{self, Foundation::{GetLastError, HMODULE, WIN32_ERROR}, System::{self, Diagnostics::Debug::{self, IMAGE_DIRECTORY_ENTRY_BASERELOC, IMAGE_DIRECTORY_ENTRY_EXPORT, IMAGE_DIRECTORY_ENTRY_IMPORT, IMAGE_DIRECTORY_ENTRY_TLS}, LibraryLoader::{GetProcAddress, LoadLibraryA}, Memory::{LocalAlloc, VirtualAlloc, VirtualFree, LMEM_ZEROINIT, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE, PAGE_READWRITE}, SystemServices::{DLL_PROCESS_ATTACH, IMAGE_BASE_RELOCATION, IMAGE_DOS_HEADER, IMAGE_EXPORT_DIRECTORY, IMAGE_IMPORT_BY_NAME, IMAGE_IMPORT_DESCRIPTOR, IMAGE_ORDINAL_FLAG64, IMAGE_REL_BASED_ABSOLUTE, IMAGE_REL_BASED_HIGHLOW, IMAGE_TLS_DIRECTORY64, PIMAGE_TLS_CALLBACK}, Threading::{TlsAlloc, TlsGetValue, TlsSetValue, TLS_OUT_OF_INDEXES}}}};
use crate::allocator;

#[derive(Error, Debug)]
pub enum Error {
    #[error("invalid pe data")]
    PeInvalid,

    #[error("invalid section")]
    PeSectionInvalid,

    #[error("rva out of bounds")]
    RvaOutOfBounds,

    #[error("mapped memory unallocated")]
    MappedMemoryUnallocated,

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

        if let Err(err) = self.pefile.validate() {
            println!("pefile.validate failed");
            return Err(err);            
        }
        for section in self.pefile.sections()? {
            println!("section.Name: {}", String::from_utf8(section.Name.to_vec()).unwrap());
        }

        if let Err(err) = self.map_structs() {
            println!("map_structs failed");
            return Err(err);            
        }

        if let Err(err) = self.resolve_imports() {
            println!("resolve_imports failed");
            return Err(err);            
        }

        if let Err(err) = self.apply_relocations() {
            println!("apply_relocations failed");
            return Err(err);            
        }

        if let Err(err) = self.init_tls_callbacks() {
            println!("init_tls_callbacks failed");
            return Err(err);

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
                PAGE_EXECUTE_READWRITE)
        };

        if mem.is_null() {
            // fallback to whatever
            let mem = unsafe {
                VirtualAlloc(None, 
                    size_of_image,
                    MEM_RESERVE | MEM_COMMIT,
                    PAGE_EXECUTE_READWRITE)
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
        let Some(mem) = self.memory else {
            return Err(memory_module::Error::MappedMemoryUnallocated);
        };

        let nt_header = self.pefile.nt_header();
        let Some(image_data_directory) = nt_header.OptionalHeader.DataDirectory.get(IMAGE_DIRECTORY_ENTRY_IMPORT.0 as usize) else {
            return Err(Error::PeInvalid);
        };

        println!("image_data_directory.Size: 0x{:x}", image_data_directory.Size);
        println!("image_data_directory.VirtualAddress: 0x{:x}", image_data_directory.VirtualAddress);

        let image_import_descriptors = unsafe { mem.add(image_data_directory.VirtualAddress as usize) as *const IMAGE_IMPORT_DESCRIPTOR };
        let mut current_image_import_descriptor = 0;
        loop {

            let image_import_descriptor: &IMAGE_IMPORT_DESCRIPTOR = unsafe { &*image_import_descriptors.add(current_image_import_descriptor) };

            if image_import_descriptor.FirstThunk == 0 {
                break;
            }

            let import_name_rva = image_import_descriptor.Name as usize;  
            println!("import_name_rva: 0x{:x}", import_name_rva);

            let import_name = PeFile::get_cstring_from_mem_at_rva(mem as *const u8, import_name_rva)?;
    
            println!("Import name: {}", import_name);
            
            let first_thunk_rva = image_import_descriptor.FirstThunk;

            let thunks_ptr = unsafe { mem.add(first_thunk_rva as usize) as *mut u64};
            let mut current_thunk = 0;

            let module = unsafe { LoadLibraryA(PCSTR::from_raw(import_name.as_ptr())) }?;

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

                    let import_name = PeFile::get_cstring_from_mem_at_rva(mem as *const u8, (image_import_by_name_rva + 2) as usize)?;
                    proc_address = unsafe { GetProcAddress(module, PCSTR::from_raw(import_name.as_ptr())).unwrap() };

                    println!("\tFunction imported by name: {}", import_name);
                }

                print!("\t\t...setting address to: 0x{:x}, thunk_ptr: {:#?}, address before: 0x{:x}", proc_address as u64, thunk_ptr, unsafe {*thunk_ptr as u64});
                unsafe { *thunk_ptr = proc_address as u64 };
                println!(", address after: 0x{:x}", unsafe {*thunk_ptr as u64});

                current_thunk += 1;
            }

            current_image_import_descriptor += 1;
        }

        Ok(())
    }

    fn apply_relocations(&self) -> Result<()> {
        let Some(mem) = self.memory else {
            return Err(memory_module::Error::MappedMemoryUnallocated);
        };

        let nt_header = self.pefile.nt_header();
        let Some(image_data_directory) = nt_header.OptionalHeader.DataDirectory.get(IMAGE_DIRECTORY_ENTRY_BASERELOC.0 as usize) else {
            return Err(Error::PeInvalid);
        };

        println!("image_data_directory.Size: 0x{:x}", image_data_directory.Size);
        println!("image_data_directory.VirtualAddress: 0x{:x}", image_data_directory.VirtualAddress);

        let mut image_base_relocation_ptr = unsafe { mem.byte_add(image_data_directory.VirtualAddress as usize) as *const IMAGE_BASE_RELOCATION };

        let mut processed = 0;

        println!("mem: {:#?} ImageBase: 0x{:x}", mem, nt_header.OptionalHeader.ImageBase as isize);
        let reloc_delta = mem.wrapping_byte_sub(nt_header.OptionalHeader.ImageBase as usize) as usize;

        loop {
            let image_base_relocation = unsafe { *image_base_relocation_ptr };

            println!("processed: 0x{:x} image_data_directory.Size: 0x{:x} image_base_relocation.SizeOfBlock: 0x{:x}", processed, image_data_directory.Size, image_base_relocation.SizeOfBlock);

            if processed >= image_data_directory.Size || image_base_relocation.SizeOfBlock == 0 {
                break;
            }
            
            // println!("relocations rva: 0x{:x}", image_base_relocation.VirtualAddress);
            let num_relocs = (image_base_relocation.SizeOfBlock as usize - size_of::<IMAGE_BASE_RELOCATION>()) / 2;
            // println!("size of block: 0x{:x} (0x{:x})", image_base_relocation.SizeOfBlock, num_relocs);


            let relocs_ptr = unsafe { image_base_relocation_ptr.add(1) as *const u16 };

            for i in 0..num_relocs {
                let reloc_ptr = unsafe { *relocs_ptr.add(i) };

                let reloc_type = (reloc_ptr & 0xF000) >> 12;
                let reloc_offset = reloc_ptr & 0x0FFF;

                let reloc_rva = image_base_relocation.VirtualAddress + reloc_offset as u32;
                
                let addr = unsafe { mem.add(reloc_rva as usize) as *mut u64 };
                let addr_before = unsafe { *addr as u64 };
                unsafe { *addr = *addr.wrapping_byte_add(reloc_delta) as u64 };
                let addr_after = unsafe { *addr as u64 };
                // println!("\treloc_type: {:x} reloc_rva: {:x} reloc_delta: {:x} addr_before: {:x} addr_after: {:x}", reloc_type, reloc_offset, reloc_delta, addr_before, addr_after);
            }

            processed += image_base_relocation.SizeOfBlock;

            image_base_relocation_ptr = unsafe { image_base_relocation_ptr.byte_add(image_base_relocation.SizeOfBlock as usize) };
        }

        Ok(())
    }

    pub fn init_tls_callbacks(&self) -> Result<()> {
        let Some(mem) = self.memory else {
            return Err(memory_module::Error::MappedMemoryUnallocated);
        };

        let nt_header = self.pefile.nt_header();
        let Some(image_data_directory) = nt_header.OptionalHeader.DataDirectory.get(IMAGE_DIRECTORY_ENTRY_TLS.0 as usize) else {
            return Err(Error::PeInvalid);
        };

        let image_tls_directory_ptr = unsafe { (mem.byte_add(image_data_directory.VirtualAddress as usize)) as *const IMAGE_TLS_DIRECTORY64 };
        let image_tls_directory = unsafe { *image_tls_directory_ptr };

        println!("image_tls_directory.AddressOfIndex: 0x{:x}", image_tls_directory.AddressOfIndex as u64);
        println!("image_tls_directory.StartAddressOfRawData: 0x{:x}", image_tls_directory.StartAddressOfRawData as u64);
        println!("image_tls_directory.EndAddressOfRawData: 0x{:x}", image_tls_directory.EndAddressOfRawData as u64);
        println!("image_tls_directory.SizeOfZeroFill: 0x{:x}", image_tls_directory.SizeOfZeroFill as u64);

        let tls_index = unsafe { TlsAlloc() };
        if tls_index == TLS_OUT_OF_INDEXES {
            return Err(Error::Win32Error(unsafe { GetLastError() }));
        }

        let alloc_size = image_tls_directory.EndAddressOfRawData as usize - 
            image_tls_directory.StartAddressOfRawData as usize +
            image_tls_directory.SizeOfZeroFill as usize;

        let tls_memory = unsafe { LocalAlloc(LMEM_ZEROINIT, alloc_size)?};

        unsafe { std::ptr::copy(image_tls_directory.StartAddressOfRawData as *const u8, 
            tls_memory.0 as *mut u8,
            alloc_size); }

        
        let mut teb_ptr: *mut Win32::System::Threading::TEB = std::ptr::null_mut();
        unsafe { core::arch::asm!(
            "mov {teb_ptr}, gs:[0x30]",
            teb_ptr = out(reg) teb_ptr
        ) }
        // println!("Got TEB: {:#?}", unsafe { *teb_ptr });

        let tls_storage_slots = unsafe { teb_ptr.byte_add(0x58) as *mut *const c_void };

        if tls_storage_slots.is_null() {
            println!("Need to allocate TlsData array");
        } else {
            let tls_storage_slots = unsafe { (*tls_storage_slots) as *mut *const c_void };
            let mut next_slot = 0usize;
    
            println!("tls_storage_slots starts at: {tls_storage_slots:#?}");
            while unsafe { *tls_storage_slots.add(next_slot) } != std::ptr::null() {
                println!("tls_storage_slots[{next_slot}]: {:#?}", unsafe { *tls_storage_slots.add(next_slot)});
                next_slot += 1;
            }
    
            println!("Found next empty slot: {next_slot}");

            let tls_index = next_slot as u32;

            let address_of_index_ptr = image_tls_directory.AddressOfIndex as *mut u32;
            unsafe { *address_of_index_ptr = tls_index };
            println!("Setting {:#?} to index {:#?}", address_of_index_ptr, tls_index);

            unsafe { *tls_storage_slots.add(tls_index as usize) = tls_memory.0 };
        }

        let mut current_callback = 0;
        
        let mut address_of_callback = image_tls_directory.AddressOfCallBacks as *mut u64;

        loop {

            let callback = unsafe { *address_of_callback.add(current_callback) };

            if callback == 0 {
                break;
            }

            let callback: PIMAGE_TLS_CALLBACK = unsafe { std::mem::transmute(callback) };
            let callback = callback.unwrap();

            println!("callback: {:#?}", callback);
            unsafe { callback(mem, DLL_PROCESS_ATTACH, std::ptr::null_mut()) };

            current_callback += 1;
            address_of_callback = unsafe { address_of_callback.add(current_callback) };
        }


        Ok(())
    }

    pub fn call_entry_point(&self) -> Result<()> {
        let Some(mem) = self.memory else {
            return Err(memory_module::Error::MappedMemoryUnallocated);
        };

        let nt_header = self.pefile.nt_header();

        let entry_point_addr = unsafe { mem.add(nt_header.OptionalHeader.AddressOfEntryPoint as usize) };

        println!("would call: {:#?}", entry_point_addr);

        type FnDllMain = unsafe extern "C" fn(HMODULE, u32, *const c_void) -> bool;

        let entry_point: FnDllMain = unsafe { std::mem::transmute(entry_point_addr) };

        unsafe { entry_point(HMODULE(mem.clone()), DLL_PROCESS_ATTACH, std::ptr::null()) };

        Ok(())
    }

    pub fn get_proc_address(&self, proc: &str) -> Option<*const c_void> {
        let Some(mem) = self.memory else {
            return None;
        };

        let nt_header = self.pefile.nt_header();
        let Some(image_data_directory) = nt_header.OptionalHeader.DataDirectory.get(IMAGE_DIRECTORY_ENTRY_EXPORT.0 as usize) else {
            return None;
        };

        let image_export_directory_ptr = unsafe { mem.byte_add(image_data_directory.VirtualAddress as usize) };
        let image_export_directory: IMAGE_EXPORT_DIRECTORY = unsafe { *(image_export_directory_ptr as *const IMAGE_EXPORT_DIRECTORY) };

        println!("image_export_directory.AddressOfNames: 0x{:x}", image_export_directory.AddressOfNames);

        let address_of_names: *const u32 = unsafe { (mem.byte_add(image_export_directory.AddressOfNames as usize)) as *const u32 };

        println!("address_of_names: {:#?}", address_of_names);

        for i in 0..image_export_directory.NumberOfNames {
            let address_of_name_rva = unsafe { *address_of_names.add(i as usize) };
            println!("address_of_name_rva[{}]: {:x}", i, address_of_name_rva);

            let Ok(export_name) = PeFile::get_cstring_from_mem_at_rva(mem as *const u8, address_of_name_rva as usize) else {
                return None;
            };

            println!("export_name: {}", export_name);

            if export_name == proc {
                let address_of_name_ordinals: *const u16 = unsafe { (mem.byte_add(image_export_directory.AddressOfNameOrdinals as usize)) as *const u16 };
                println!("address_of_name_ordinals: {:#?}", address_of_name_ordinals);

                let export_name_ordinal = unsafe { *address_of_name_ordinals.add(i as usize) };
                println!("export_name_ordinal[{}]: 0x{:x}", i, export_name_ordinal);

                let address_of_functions: *const u32 = unsafe { (mem.byte_add(image_export_directory.AddressOfFunctions as usize)) as *const u32 };
                println!("address_of_functions: {:#?}", address_of_functions);

                let export_function = unsafe { *address_of_functions.add(export_name_ordinal as usize) };
                println!("address_of_functions[{}]: 0x{:x}", export_name_ordinal, export_function);

                let export_function = unsafe { mem.byte_add(export_function as usize) };
                println!("export_function: {:#?}", export_function);

                return Some(export_function);
            }
        }

        None
    }

    pub fn hmodule(&self) -> Option<HMODULE> {
        if let Some(mem) = self.memory {
            Some(HMODULE(mem))
        } else {
            None
        }
    }
}