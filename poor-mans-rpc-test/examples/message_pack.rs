fn test_message_pack(arg1: String, arg2: u64, arg3: (i32, i32)) -> String {
    let message = format!("arg1: {arg1} arg2: {arg2}: arg3: ({},{})", arg3.0, arg3.1);

    println!("test_message_pack message: {message}");

    message
}

fn __stub_test_message_pack(args: Vec<u8>) -> Vec<u8> {
    let (arg1, arg2, arg3) = rmp_serde::decode::from_slice(args.as_slice()).unwrap();

    let ret = test_message_pack(arg1, arg2, arg3);

    rmp_serde::encode::to_vec(&ret).unwrap()
}

fn main() {
    let args =
        rmp_serde::encode::to_vec(&("arg1_string".to_owned(), 22u64, (33i32, 44i32))).unwrap();

    let ret = __stub_test_message_pack(args);

    println!("__stub_test_message_pack ret: {ret:#?}");

    let decoded: String = rmp_serde::decode::from_slice(&ret).unwrap();

    println!("__stub_test_message_pack ret decoded: {decoded}");
}
