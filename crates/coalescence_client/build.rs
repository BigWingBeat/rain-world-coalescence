use std::path::PathBuf;

fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let Ok(out_dir) = std::env::var("CBINDGEN_OUT_DIR") else {
        return;
    };

    // let out_dir = ".";

    let mut crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
    crate_name.push_str(".h");

    let out_file: PathBuf = [out_dir, crate_name].iter().collect();

    cbindgen::generate(crate_dir)
        .expect("Unable to generate bindings")
        .write_to_file(out_file);
}
