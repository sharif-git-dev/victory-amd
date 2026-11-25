fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=src/seh.c");
        cc::Build::new().file("src/seh.c").compile("victory_gpu_seh");
    }
}
