use cmake::Config;

fn main() {
    let dst = Config::new("../..")
        .define("BUILD_RAT_CLI", "OFF")
        .define("BUILD_TESTS", "OFF")
        .build_target("rat")
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=rat");
}
