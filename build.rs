fn main() {
    println!("cargo:rerun-if-changed=c_wrapper/jsbsim_wrapper.h");
    println!("cargo:rerun-if-changed=c_wrapper/jsbsim_wrapper.cpp");

    let jsbsim = pkg_config::Config::new()
        .probe("JSBSim")
        .expect("JSBSim not found via pkg-config!");

    let jsbsim_src = "/home/joni/jsbsim/src";

    // Compile the C++ wrapper
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("c_wrapper/jsbsim_wrapper.cpp")
        .flag_if_supported("-std=c++17")
        .include(jsbsim_src);

    for inc in &jsbsim.include_paths {
        build.include(inc);
    }
    build.compile("jsbsim_wrapper");

    // Link JSBSim
    for path in &jsbsim.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }
    println!("cargo:rustc-link-lib=dylib=JSBSim");
}
