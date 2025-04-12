mod allocator;

#[cfg(test)]
mod tests {
    use crate::allocator::{self, ModuleAllocator};

    #[test]
    fn test_alloc() {
        let mem = allocator::VirtualAlloc::mem_alloc(0x1000, None);

        dbg!(mem);

        if let Some(m) = mem {
            allocator::VirtualAlloc::mem_free(m);
        }
    }
}
