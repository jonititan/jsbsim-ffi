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

    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR");

    // Discover JSBSim via pkg-config.  We emit the link directives ourselves
    // (see emit_link_directives) rather than letting probe() do it, because
    // JSBSim's installed JSBSim.pc is wrong: it lists -lAtmosphere, -lModels,
    // -lXml, ... which are CMake object libraries already bundled into
    // libJSBSim and never installed, so passing them on makes linking fail.
    let jsbsim = pkg_config::Config::new()
        .statik(use_static)
        .cargo_metadata(false)
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

    emit_link_directives(&jsbsim, use_static);

    // Compile the C++ wrapper that bridges JSBSim's C++ API to a C ABI.
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("c_wrapper/jsbsim_wrapper.cpp")
        .flag_if_supported("-std=c++17");

    // Use the include paths reported by pkg-config as *system* includes
    // (-isystem) so that warnings inside JSBSim's own headers are suppressed.
    for inc in &jsbsim.include_paths {
        build.flag(format!("-isystem{}", inc.display()));
    }

    // Allow an extra include path for non-standard installs (also as system).
    if let Ok(extra) = std::env::var("JSBSIM_INCLUDE_DIR") {
        build.flag(format!("-isystem{}", extra));
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

/// Emit `cargo:rustc-link-search` / `cargo:rustc-link-lib` for the libraries
/// pkg-config reported, skipping any that do not exist on disk.
///
/// JSBSim's JSBSim.pc lists its internal CMake object libraries (Atmosphere,
/// Models, Xml, ...) as if they were installed.  A library is dropped only if
/// it is absent from both the pkg-config `-L` paths and the compiler's default
/// library search dirs; if those dirs cannot be determined, every library is
/// kept so behaviour falls back to plain pkg-config.
fn emit_link_directives(lib: &pkg_config::Library, use_static: bool) {
    for path in &lib.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }

    let system_dirs = compiler_library_dirs();
    let mut skipped = Vec::new();

    for name in &lib.libs {
        let in_link_paths = |ext: &str| {
            lib.link_paths
                .iter()
                .any(|dir| dir.join(format!("lib{name}.{ext}")).exists())
        };
        let exists = ["a", "so", "dylib", "tbd", "lib"].iter().any(|ext| {
            in_link_paths(ext)
                || system_dirs
                    .iter()
                    .any(|dir| dir.join(format!("lib{name}.{ext}")).exists())
        });

        if !exists && !system_dirs.is_empty() {
            skipped.push(format!("-l{name}"));
            continue;
        }

        if use_static && in_link_paths("a") {
            println!("cargo:rustc-link-lib=static={name}");
        } else {
            println!("cargo:rustc-link-lib={name}");
        }
    }

    if !skipped.is_empty() {
        println!(
            "cargo:warning=ignoring libraries listed in JSBSim.pc that are not installed \
             (bundled into libJSBSim): {}",
            skipped.join(" ")
        );
    }
}

/// The C/C++ compiler's default library search directories, as reported by
/// `-print-search-dirs` (GCC and Clang).  Empty if unavailable, e.g. on MSVC.
fn compiler_library_dirs() -> Vec<std::path::PathBuf> {
    let Ok(compiler) = cc::Build::new().cpp(true).try_get_compiler() else {
        return Vec::new();
    };
    let Ok(output) = compiler.to_command().arg("-print-search-dirs").output() else {
        return Vec::new();
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("libraries: ="))
        .map(|dirs| std::env::split_paths(dirs).filter(|d| d.is_dir()).collect())
        .unwrap_or_default()
}
