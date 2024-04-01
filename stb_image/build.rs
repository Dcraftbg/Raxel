fn main() {
    cc::Build::new()
        .file("build/stb_image.c")
        .include("vendor")
        .static_crt(true)
        .compile("stb_image");
    println!("cargo:rustc-link-lib=static=stb_image");
}
