
use std::ffi::{c_uchar, c_void};
use std::mem::offset_of;
use std::slice;

use windows::Win32::System::SystemServices::{self, IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE};
use windows::Win32::System::Diagnostics::Debug::{self, ImageNtHeader, IMAGE_NT_HEADERS64, IMAGE_NT_OPTIONAL_HDR64_MAGIC, IMAGE_SECTION_HEADER};

use crate::memory_module;



pub struct PeFile {
    pub data: Vec<u8>,
}

impl PeFile {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
        }
    }

    pub fn dos_header(&self) -> &IMAGE_DOS_HEADER {
        unsafe { &*std::mem::transmute::<*const u8, *const IMAGE_DOS_HEADER>(self.data.as_ptr()) }
    }

    pub fn nt_header_ptr(&self) -> *const IMAGE_NT_HEADERS64 {
        let pe = &self.data;
        unsafe { ImageNtHeader(pe.as_ptr() as *const c_void ) }
    }

    pub fn nt_header(&self) -> &IMAGE_NT_HEADERS64 {
        unsafe { &*self.nt_header_ptr() }
    }

    pub fn validate(&self) -> memory_module::Result<()> {

        let pe = &self.data;
    
        if pe.len() < size_of::<IMAGE_DOS_HEADER>() {
            return Err(memory_module::Error::PeInvalid);
        }
    
        let dos_header = unsafe { *std::mem::transmute::<*const u8, *const IMAGE_DOS_HEADER>(pe.as_ptr()) };
        if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
            return Err(memory_module::Error::PeInvalid);
        }
    
        if pe.len() < (dos_header.e_lfanew as usize) + size_of::<IMAGE_NT_HEADERS64>() {
            return Err(memory_module::Error::PeInvalid);
        }
    
        let nt_header_ptr = unsafe { ImageNtHeader(pe.as_ptr() as *const c_void ) };
        let nt_header = unsafe { *nt_header_ptr };
        if nt_header.Signature != IMAGE_NT_SIGNATURE {
            return Err(memory_module::Error::PeInvalid);
        }
    
        if nt_header.OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC {
            return Err(memory_module::Error::PeInvalid);
        }
    
        if pe.len() < nt_header.OptionalHeader.SizeOfImage as usize ||
            pe.len() < nt_header.OptionalHeader.SizeOfHeaders as usize
         {
            return Err(memory_module::Error::PeInvalid);
        }
    
        Ok(())
    }


    pub fn sections(&self) -> memory_module::Result<&[IMAGE_SECTION_HEADER]> {
        let pe = &self.data;
        let nt_header_ptr = self.nt_header_ptr();
        let nt_header = unsafe { *nt_header_ptr };

        if nt_header.Signature != IMAGE_NT_SIGNATURE {
            return Err(memory_module::Error::PeInvalid);
        }
    
        if nt_header.OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC {
            return Err(memory_module::Error::PeInvalid);
        }
    
        if pe.len() < nt_header.OptionalHeader.SizeOfImage as usize ||
            pe.len() < nt_header.OptionalHeader.SizeOfHeaders as usize
         {
            return Err(memory_module::Error::PeInvalid);
        }
    
        let optional_header_offset = offset_of!(IMAGE_NT_HEADERS64, OptionalHeader);
    
        let first_section_ptr: *mut IMAGE_SECTION_HEADER = unsafe { ((nt_header_ptr as *mut u8).
            add(optional_header_offset).
            add(nt_header.FileHeader.SizeOfOptionalHeader as usize)) as *mut IMAGE_SECTION_HEADER };
    
        let num_sections = nt_header.FileHeader.NumberOfSections;
        let sections = unsafe { std::slice::from_raw_parts(first_section_ptr, num_sections as usize) };

        Ok(sections)
    }

    pub fn section(&self, section: usize) -> Option<(&IMAGE_SECTION_HEADER, &[u8])> {
        if let Ok(sections) = self.sections() {
            if section < sections.len() {
                if let Ok(section_data) = self.section_data(section) {
                    return Some((&sections[section], section_data));
                }
            }
        }

        None
    }

    pub fn section_by_name(&self, section_name: &[u8; 8]) -> Option<(&IMAGE_SECTION_HEADER, &[u8])> {
        if let Ok(sections) = self.sections() {
            for (i, section) in sections.iter().enumerate() {
                if section.Name == *section_name {
                    if let Ok(section_data) = self.section_data(i) {
                        return Some((section, section_data));
                    }
                }
            }
        }

        None
    }

    pub fn section_data(&self, section: usize) -> memory_module::Result<&[u8]> {
        let sections = self.sections()?;

        if section > sections.len() {
            return Err(memory_module::Error::PeSectionInvalid);
        }

        let section = sections[section];

        let base = self.data.as_ptr();

        let ptr_to_raw_data = section.PointerToRawData as usize;
        let virtual_size = unsafe { section.Misc.VirtualSize as usize };

        if ptr_to_raw_data > self.data.len() ||
        (ptr_to_raw_data + virtual_size) > self.data.len() {
            return Err(memory_module::Error::PeInvalid);
        }
        let raw_data = unsafe { base.add(ptr_to_raw_data) };

        let section_data = unsafe { slice::from_raw_parts(raw_data, virtual_size) };

        Ok(section_data)
    }

    pub fn get_struct_at_rva<T>(&self, rva: usize) -> memory_module::Result<&T> {
        if rva + size_of::<T>() > self.data.len()  {
            return Err(memory_module::Error::RvaOutOfBounds);
        }

        let data_ptr = unsafe { self.data.as_ptr().add(rva) };

        let t_ref = unsafe { &*(data_ptr as *const T) };

        Ok(t_ref)
    }

    pub fn get_cstring_at_rva(&self, rva: usize) -> memory_module::Result<&str> {
        if rva > self.data.len()  {
            return Err(memory_module::Error::RvaOutOfBounds);
        }

        let ptr = unsafe { self.data.as_ptr().add(rva) };
        let mut len = 0usize;

        loop {
            if rva + len > self.data.len() {
                break;
            }

            if unsafe { *ptr.add(len) } == 0 {
                break;
            }

            len += 1;
        }

        let s = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };

        Ok(s)
    }
}
