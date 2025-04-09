#![allow(warnings)]
pub fn hello_world(msg: String) -> String {
    format!("Hello, {msg}!")
}

pub trait Dispatcher {
    fn func1();

    fn func2(arg1: String);

    fn func3(arg1: String, arg2: u64) -> String;
}

pub struct TestDispatch {}

#[pmr::dispatcher]
impl Dispatcher for TestDispatch {
    fn func1() {
        println!("Dispatch::func1")
    }

    fn func2(arg1: String) {
        println!("Dispatch::func2, arg1: {arg1}");
    }

    fn func3(arg1: String, arg2: u64) -> String {
        let formatted = format!("Dispatch::func3, arg1: {arg1}, arg2: {arg2}");

        println!("{formatted}");

        formatted
    }
}

#[allow(non_snake_case)]
pub mod __Dispatch_mod {

    fn get_random_alphanum(len: usize) -> Vec<u8> {
        use rand::Rng;
        let mut rng = rand::rng();

        // String:
        (&mut rng)
            .sample_iter(rand::distr::Alphanumeric)
            .take(len)
            .collect()
    }

    fn __stub_function_template(args: Vec<u8>) -> Vec<u8> {
        let formatted = format!("__stub_function_template called with args: {:?}", args);

        formatted.as_bytes().to_vec()
    }

    type StubFn = fn(Vec<u8>) -> Vec<u8>;
    fn init_dispatch_map() -> std::vec::Vec<StubFn> {
        std::vec![__stub_function_template as StubFn]
    }

    pub fn get_dispatch_map() -> &'static std::sync::Mutex<std::vec::Vec<StubFn>> {
        static DISPATCH_MAP: std::sync::OnceLock<std::sync::Mutex<std::vec::Vec<StubFn>>> =
            std::sync::OnceLock::new();
        DISPATCH_MAP.get_or_init(|| std::sync::Mutex::new(init_dispatch_map()))
    }

    pub enum DispatchError {
        Success = 0,
        UnknownFunction = 1,
        MapMutexFailure = 2,
    }
}

pub extern "C" fn dispatch_call(id: usize, args: Vec<u8>) -> Vec<u8> {
    println!("id in dispatch_call: {:?}", id);
    let dispatch_map = __Dispatch_mod::get_dispatch_map();

    let ret = match dispatch_map.lock() {
        Ok(dm) => match dm.get(id) {
            Some(stub_fn) => stub_fn(args),
            None => vec![__Dispatch_mod::DispatchError::UnknownFunction as u8],
        },
        Err(_) => std::vec![__Dispatch_mod::DispatchError::MapMutexFailure as u8],
    };

    ret
}

pub extern "C" fn init() -> Vec<usize> {
    let dispatch_map = __Dispatch_mod::get_dispatch_map();
    let mut ids = Vec::<Vec<u8>>::new();

    match dispatch_map.lock() {
        Ok(v) => (0..v.len()).collect(),
        Err(_) => Vec::new(),
    }
}

fn main() {
    println!("{}", hello_world("hello".into()));

    TestDispatch::func1();

    TestDispatch::func2("func2 arg".to_owned());

    TestDispatch::func3("func3 arg".to_owned(), 0x4444);

    let ret = dispatch_call(0, "called with dispatch call".as_bytes().to_vec());
    println!(
        "return from dispatch_call: {:?}",
        String::from_utf8(ret).unwrap()
    );
}
