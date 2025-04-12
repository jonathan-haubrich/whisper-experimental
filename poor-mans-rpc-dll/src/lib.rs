pub trait Dispatcher {
    fn func1();

    fn func2(arg1: String);

    fn func3(arg1: String, arg2: u64) -> String;

    fn func4(arg1: String, arg2: String) -> DirInfo;
}

pub struct Dispatch {}

#[derive(serde::Serialize)]
pub struct DirOwner {
    name: String,
    id: usize,
}

#[derive(serde::Serialize)]
pub struct DirInfo {
    path: String,
    permissions: u64,
    owner: DirOwner,
}

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

    fn func4(arg1: String, arg2: String) -> DirInfo {
        DirInfo {
            path: arg2.clone(),
            permissions: 777,
            owner: DirOwner {
                name: arg1.clone(),
                id: 42,
            },
        }
    }
}
