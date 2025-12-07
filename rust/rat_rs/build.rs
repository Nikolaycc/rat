use meson_next as meson;

use std::env;
use std::path::PathBuf;
use std::collections::HashMap;

fn main() {
    let mut build_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    build_path = build_path.join("../../build");
    let build_path = build_path.to_str().unwrap();

    let mut options = HashMap::new();
    options.insert("allbuild", "false");
    
    let config = meson::Config::new().options(options);

    println!("cargo:rustc-link-lib=rat");
    println!("cargo:rustc-link-search=native={}", build_path);
    meson::build("../../", build_path, config);
}
