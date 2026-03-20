fn main() {
    println!("cargo:rerun-if-changed=c_wrapper/jsbsim_wrapper.h");
    println!("cargo:rerun-if-changed=c_wrapper/jsbsim_wrapper.cpp");
    println!("cargo:rerun-if-env-changed=JSBSIM_STATIC");
    println!("cargo:rerun-if-env-changed=JSBSIM_INCLUDE_DIR");

    // Should we link JSBSim statically?
    //
    //   JSBSIM_STATIC=1 cargo build   → links libJSBSim.a into the binary
    //                                    (no runtime .so dependency)
    //
    // Default is dynamic linking.  When dynamic, we embed the RPATH so
    // the resulting binary can find libJSBSim.so at runtime without the
    // user having to set LD_LIBRARY_PATH.
    let use_static = std::env::var("JSBSIM_STATIC").is_ok();

    // Discover JSBSim via pkg-config.
    // probe() automatically emits the necessary cargo:rustc-link-lib and
    // cargo:rustc-link-search directives.
    let jsbsim = pkg_config::Config::new()
        .statik(use_static)
        .probe("JSBSim")
        .expect(
            "JSBSim not found via pkg-config!\n\
             \n\
             Make sure JSBSim is installed and its .pc file is discoverable.\n\
             Common fixes:\n\
             \n\
               • Install JSBSim from source:\n\
                   cd jsbsim/build && cmake .. && make && sudo make install\n\
             \n\
               • Ensure the .pc file is on PKG_CONFIG_PATH:\n\
                   export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH\n\
             \n\
               • Register the shared library with the linker cache:\n\
                   sudo ldconfig\n",
        );

    // Compile the C++ wrapper that bridges JSBSim's C++ API to a C ABI.
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("c_wrapper/jsbsim_wrapper.cpp")
        .flag_if_supported("-std=c++17");

    // Use the include paths reported by pkg-config
    // (e.g. /usr/local/include/JSBSim where FGFDMExec.h and simgear/ live).
    for inc in &jsbsim.include_paths {
        build.include(inc);
    }

    // Allow an extra include path for non-standard installs.
    if let Ok(extra) = std::env::var("JSBSIM_INCLUDE_DIR") {
        build.include(&extra);
    }

    build.compile("jsbsim_wrapper");

    // When dynamically linking, embed RPATH so the final binary can locate
    // libJSBSim.so at runtime without LD_LIBRARY_PATH.
    if !use_static {
        for path in &jsbsim.link_paths {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
        }
    }
}
