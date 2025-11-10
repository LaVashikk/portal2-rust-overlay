fn main() {
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

    match target_env.as_str() {
        "msvc" => {
            println!("cargo:rustc-link-arg=/DEF:d3d9.def");
        }
        "gnu" => {
            println!("cargo:rustc-link-arg=--def=d3d9.def");
        }
        _ => {}
    }
}
