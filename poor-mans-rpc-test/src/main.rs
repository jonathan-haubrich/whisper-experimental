pub fn hello_world(msg: String) -> String {
    format!("Hello, {msg}!")
}

pub trait Dispatcher {
    fn func1();

    fn func2(arg1: String);

    fn func3(arg1: String, arg2: u64) -> String;
}

pub struct Dispatch {}

impl Dispatcher for Dispatch {
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

fn get_random_string(len: usize) -> String {
    use rand::Rng;
    let mut rng = rand::rng();

    // String:
    (&mut rng)
        .sample_iter(rand::distr::Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn __stub_function_template(args: Vec<u8>) -> Vec<u8> {
    let formatted = format!("__stub_function_template called with args: {:?}", args);

    formatted.as_bytes().to_vec()
}

type StubFn = fn(Vec<u8>) -> Vec<u8>;

fn init_dispatch_map() -> std::collections::HashMap<String, StubFn> {
    std::collections::HashMap::from([(get_random_string(16), __stub_function_template as StubFn)])
}

fn get_dispatch_map() -> &'static std::sync::Mutex<std::collections::HashMap<String, StubFn>> {
    static DISPATCH_MAP: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, StubFn>>,
    > = std::sync::OnceLock::new();
    DISPATCH_MAP.get_or_init(|| std::sync::Mutex::new(init_dispatch_map()))
}

enum DispatchError {
    Success = 0,
    UnknownFunction = 1,
    MapMutexFailure = 2,
}

fn dispatch_call(id: String, args: Vec<u8>) -> Vec<u8> {
    let dispatch_map = get_dispatch_map();

    match dispatch_map.lock() {
        Ok(dm) => match dm.get(&id) {
            Some(stub_fn) => stub_fn(args),
            None => vec![DispatchError::UnknownFunction as u8],
        },
        Err(_) => std::vec![DispatchError::MapMutexFailure as u8],
    }
}

fn init() -> Vec<Vec<u8>> {
    let dispatch_map = get_dispatch_map();
    let mut ids = Vec::<Vec<u8>>::new();

    for key in dispatch_map.lock().unwrap().keys() {
        ids.push(key.as_bytes().to_vec());
    }

    ids
}

fn main() {
    println!("{}", hello_world("hello".into()));

    Dispatch::func1();

    Dispatch::func2("func2 arg".to_owned());

    Dispatch::func3("func3 arg".to_owned(), 0x4444);

    let ids = init();
    let ret = dispatch_call(
        String::from_utf8(ids[0].clone()).unwrap(),
        "called with dispatch call".as_bytes().to_vec(),
    );
    println!(
        "return from dispatch_call: {:?}",
        String::from_utf8(ret).unwrap()
    );
}
