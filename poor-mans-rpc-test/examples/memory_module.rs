use memory_module::{MemoryModule, allocator};

fn main() {
    println!("cwd: {:#?}", std::path::absolute(".").unwrap());

    let dll = std::fs::read(r#".\target\debug\pmr_dll.dll"#).unwrap();
    
    let mut memory_module = MemoryModule::<allocator::VirtualAlloc>::new(dll);

    match memory_module.load_library() {
        Ok(_) => println!("Loaded successfully"),
        Err(err) => panic!("Got an error :(  {err:#?}"),
    }

    // match memory_module.call_entry_point() {
    //     Ok(_) => println!("Called successfully!"),
    //     Err(err) => panic!("Nope :( {err:#?}"),
    // }

    type FnDispatch = unsafe extern "C" fn(id: usize, arg_ptr: *mut u8, arg_len: usize, ret_ptr: &mut *mut u8, ret_len: &mut usize);

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
    println!("output: {:#?} output_len: 0x{:x}", output, output_len);

    let func2_input = String::from("testing func 2");
    let mut packed = rmp_serde::to_vec(&func2_input).unwrap();
    unsafe { dispatch_ptr(1, packed.as_mut_ptr(), packed.len(), &mut output, &mut output_len) };
    println!("output: {:#?} output_len: 0x{:x}", output, output_len);

    let func3_input = String::from("testing func 3");
    let func3_input2 = 42u64;
    let mut packed = rmp_serde::to_vec(&(func3_input, func3_input2)).unwrap();
    unsafe { dispatch_ptr(2, packed.as_mut_ptr(), packed.len(), &mut output, &mut output_len) };
    println!("output: {:#?} output_len: 0x{:x}", output, output_len);

    let func4_input = String::from("testing func 4");
    let func4_input2 = String::from("testing func 4 (2)");
    let mut packed = rmp_serde::to_vec(&(func4_input, func4_input2)).unwrap();
    unsafe { dispatch_ptr(3, packed.as_mut_ptr(), packed.len(), &mut output, &mut output_len) };
    println!("output: {:#?} output_len: 0x{:x}", output, output_len);

}