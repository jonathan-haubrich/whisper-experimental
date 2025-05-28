fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let module_dll_filepath = std::path::Path::new(&out_dir).join(r"..\..\..\module_survey_dll.dll");
    println!("cargo::rustc-env=MODULE_DLL_FILEPATH={}", module_dll_filepath.to_str().unwrap());
}