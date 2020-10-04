use crate::target::arch_str;
use std::io;
use std::path::Path;
use std::process::{Child, Command};
use target_lexicon::{Architecture, OperatingSystem, Triple};

pub fn link(
    target: &Triple,
    binary_path: &Path,
    host_input_path: &Path,
    dest_filename: &Path,
) -> io::Result<Child> {
    // TODO we should no longer need to do this once we have platforms on
    // a package repository, as we can then get precompiled hosts from there.
    rebuild_host(host_input_path);

    match target {
        Triple {
            architecture: Architecture::X86_64,
            operating_system: OperatingSystem::Linux,
            ..
        } => link_linux(target, binary_path, host_input_path, dest_filename),
        Triple {
            architecture: Architecture::X86_64,
            operating_system: OperatingSystem::Darwin,
            ..
        } => link_macos(target, binary_path, host_input_path, dest_filename),
        _ => panic!("TODO gracefully handle unsupported target: {:?}", target),
    }
}

fn rebuild_host(host_input_path: &Path) {
    let c_host_src = host_input_path.with_file_name("host.c");
    let c_host_dest = host_input_path.with_file_name("c_host.o");
    let rust_host_src = host_input_path.with_file_name("host.rs");
    let rust_host_dest = host_input_path.with_file_name("rust_host.o");
    let host_dest = host_input_path.with_file_name("host.o");

    // Compile host.c
    Command::new("clang")
        .env_clear()
        .args(&[
            "-c",
            c_host_src.to_str().unwrap(),
            "-o",
            c_host_dest.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    if rust_host_src.exists() {
        // Compile and link host.rs, if it exists
        Command::new("rustc")
            .args(&[
                rust_host_src.to_str().unwrap(),
                "-o",
                rust_host_dest.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        Command::new("ld")
            .env_clear()
            .args(&[
                "-r",
                c_host_dest.to_str().unwrap(),
                rust_host_dest.to_str().unwrap(),
                "-o",
                host_dest.to_str().unwrap(),
            ])
            .output()
            .unwrap();

        // Clean up rust_host.o
        Command::new("rm")
            .env_clear()
            .args(&[
                "-f",
                rust_host_dest.to_str().unwrap(),
                c_host_dest.to_str().unwrap(),
            ])
            .output()
            .unwrap();
    } else {
        // Clean up rust_host.o
        Command::new("mv")
            .env_clear()
            .args(&[c_host_dest, host_dest])
            .output()
            .unwrap();
    }
}

fn link_linux(
    target: &Triple,
    binary_path: &Path,
    host_input_path: &Path,
    dest_filename: &Path,
) -> io::Result<Child> {
    // NOTE: order of arguments to `ld` matters here!
    // The `-l` flags should go after the `.o` arguments
    Command::new("ld")
        // Don't allow LD_ env vars to affect this
        .env_clear()
        .args(&[
            "-arch",
            arch_str(target),
            "/usr/lib/x86_64-linux-gnu/crti.o",
            "/usr/lib/x86_64-linux-gnu/crtn.o",
            "/usr/lib/x86_64-linux-gnu/Scrt1.o",
            "-dynamic-linker",
            "/lib64/ld-linux-x86-64.so.2",
            // Inputs
            host_input_path.to_str().unwrap(), // host.o
            dest_filename.to_str().unwrap(),   // app.o
            // Libraries - see https://github.com/rtfeldman/roc/pull/554#discussion_r496365925
            // for discussion and further references
            "-lc",
            "-lm",
            "-lpthread",
            "-ldl",
            "-lrt",
            "-lutil",
            "-lc_nonshared",
            "-lc++",
            "-lunwind",
            // "-lgcc", // TODO will eventually need compiler_rt from gcc or something - see https://github.com/rtfeldman/roc/pull/554#discussion_r496370840
            // Output
            "-o",
            binary_path.to_str().unwrap(), // app
        ])
        .spawn()
}

fn link_macos(
    target: &Triple,
    binary_path: &Path,
    host_input_path: &Path,
    dest_filename: &Path,
) -> io::Result<Child> {
    // NOTE: order of arguments to `ld` matters here!
    // The `-l` flags should go after the `.o` arguments
    Command::new("ld")
        // Don't allow LD_ env vars to affect this
        .env_clear()
        .args(&[
            "-arch",
            target.architecture.to_string().as_str(),
            // Inputs
            host_input_path.to_str().unwrap(), // host.o
            dest_filename.to_str().unwrap(),   // roc_app.o
            // Libraries - see https://github.com/rtfeldman/roc/pull/554#discussion_r496392274
            // for discussion and further references
            "-lSystem",
            "-lresolv",
            "-lpthread",
            // "-lrt", // TODO shouldn't we need this?
            // "-lc_nonshared", // TODO shouldn't we need this?
            // "-lgcc", // TODO will eventually need compiler_rt from gcc or something - see https://github.com/rtfeldman/roc/pull/554#discussion_r496370840
            // "-lunwind", // TODO will eventually need this, see https://github.com/rtfeldman/roc/pull/554#discussion_r496370840
            "-lc++", // TODO shouldn't we need this?
            // Output
            "-o",
            binary_path.to_str().unwrap(), // app
        ])
        .spawn()
}