
use std::ffi::{c_uchar, c_void};
use std::mem::offset_of;
use std::slice;

use windows::Win32::System::SystemServices::{self, IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_SIGNATURE};
use windows::Win32::System::Diagnostics::Debug::{self, ImageNtHeader, IMAGE_NT_HEADERS64, IMAGE_NT_OPTIONAL_HDR64_MAGIC, IMAGE_SECTION_HEADER};

use crate::error::{MemoryModuleError, Result as MMResult};

pub struct PeFile {
    data: Vec<u8>,
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
        let pe = &self.data;
        let nt_header_ptr = unsafe { ImageNtHeader(pe.as_ptr() as *const c_void ) };
        unsafe { &*nt_header_ptr }
    }

    pub fn validate(&self) -> MMResult<()> {

        let pe = &self.data;
    
        if pe.len() < size_of::<IMAGE_DOS_HEADER>() {
            return Err(MemoryModuleError::PeInvalid);
        }
    
        let dos_header = unsafe { *std::mem::transmute::<*const u8, *const IMAGE_DOS_HEADER>(pe.as_ptr()) };
        if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
            return Err(MemoryModuleError::PeInvalid);
        }
    
        if pe.len() < (dos_header.e_lfanew as usize) + size_of::<IMAGE_NT_HEADERS64>() {
            return Err(MemoryModuleError::PeInvalid)
        }
    
        let nt_header_ptr = unsafe { ImageNtHeader(pe.as_ptr() as *const c_void ) };
        let nt_header = unsafe { *nt_header_ptr };
        if nt_header.Signature != IMAGE_NT_SIGNATURE {
            return Err(MemoryModuleError::PeInvalid)
        }
    
        if nt_header.OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC {
            return Err(MemoryModuleError::PeInvalid)
        }
    
        if pe.len() < nt_header.OptionalHeader.SizeOfImage as usize ||
            pe.len() < nt_header.OptionalHeader.SizeOfHeaders as usize
         {
            return Err(MemoryModuleError::PeInvalid)
        }
    
        Ok(())
    }


    pub fn sections(&self) -> MMResult<&[IMAGE_SECTION_HEADER]> {
        let pe = &self.data;
        let nt_header_ptr = self.nt_header_ptr();
        let nt_header = unsafe { *nt_header_ptr };

        if nt_header.Signature != IMAGE_NT_SIGNATURE {
            return Err(MemoryModuleError::PeInvalid)
        }
    
        if nt_header.OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC {
            return Err(MemoryModuleError::PeInvalid)
        }
    
        if pe.len() < nt_header.OptionalHeader.SizeOfImage as usize ||
            pe.len() < nt_header.OptionalHeader.SizeOfHeaders as usize
         {
            return Err(MemoryModuleError::PeInvalid)
        }
    
        let optional_header_offset = offset_of!(IMAGE_NT_HEADERS64, OptionalHeader);
    
        let first_section_ptr: *mut IMAGE_SECTION_HEADER = unsafe { ((nt_header_ptr as *mut u8).
            add(optional_header_offset).
            add(nt_header.FileHeader.SizeOfOptionalHeader as usize)) as *mut IMAGE_SECTION_HEADER };
    
        let num_sections = nt_header.FileHeader.NumberOfSections;
        let sections = unsafe { std::slice::from_raw_parts(first_section_ptr, num_sections as usize) };

        Ok(sections)
    }
}
