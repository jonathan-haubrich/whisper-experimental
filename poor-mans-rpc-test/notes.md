### Design

- proc macro to decorate functions that should be exported

```
#[pmr::export]
pub fn hello_world(msg: String) -> String {}
```

expands to:

```

// args for func message be (de)serializeable

#[derivce(rmp_serde::Deserialize, rmp_serder::Serialize)]
struct __stub_hello_world_args {
  __msg: String,
}

extern "C" fn __stub_hello_world(args: Vec<u8>) -> Vec<u8> {
  let mut args = 

}
```
