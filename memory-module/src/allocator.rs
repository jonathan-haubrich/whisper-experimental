use std::ffi::c_void;

use windows::Win32::System::Memory::{
    self, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE, PAGE_READWRITE,
};

pub trait ModuleAllocator {
    fn mem_alloc(size: usize, permissions: Option<u32>) -> Option<*mut c_void>;

    fn mem_free(mem: *mut c_void);

    fn mem_protect(mem: *mut u8, permissions: usize);
}

pub struct VirtualAlloc {}

impl ModuleAllocator for VirtualAlloc {
    fn mem_alloc(size: usize, permissions: Option<u32>) -> Option<*mut c_void> {
        let allocation_type = MEM_RESERVE | MEM_COMMIT;
        let protect = match permissions {
            Some(protect) => Memory::PAGE_PROTECTION_FLAGS(protect),
            None => PAGE_READWRITE | PAGE_EXECUTE,
        };

        let mem = unsafe { Memory::VirtualAlloc(None, size, allocation_type, protect) };

        if mem.is_null() { None } else { Some(mem) }
    }

    fn mem_free(mem: *mut c_void) {
        let _ = unsafe { Memory::VirtualFree(mem, 0, MEM_RELEASE) };
    }

    fn mem_protect(mem: *mut u8, permissions: usize) {
        todo!()
    }
}
