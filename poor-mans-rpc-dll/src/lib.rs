pub trait Dispatcher {
    fn func1();

    fn func2(arg1: String);

    fn func3(arg1: String, arg2: u64) -> String;
}

pub struct Dispatch {}

#[pmr::dispatcher]
impl Dispatcher for Dispatch {
    fn func1() {
        println!("Dispatch::func");
    }

    fn func2(arg1: String) {
        println!("Dispatch::func2, arg1: {arg1}");
    }

    fn func3(arg1: String, arg2: u64) -> String {
        let formatted = format!("Dispatch::func3, arg1: {arg1} arg2: {arg2}");

        println!("{formatted}");

        formatted
    }
}
