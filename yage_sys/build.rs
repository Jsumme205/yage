use std::io;
use std::path::Path;
use std::process::Command;

const XDG_SHELL_XML: &str = "xdg-shell-unstable-v6.xml";

fn files_exist<P>(paths: &[P]) -> io::Result<bool>
where
    P: AsRef<Path>,
{
    let mut result = false;
    for path in paths {
        result = std::fs::exists(path)?;
    }
    Ok(result)
}

fn generate_files() -> io::Result<()> {
    Command::new("wayland-scanner")
        .args(["client-header", XDG_SHELL_XML, "xdg-shell.h"])
        .status()?;
    Command::new("wayland-scanner")
        .args(["code", XDG_SHELL_XML, "xdg-shell.c"])
        .status()?;
    Ok(())
}

fn cleanup() -> io::Result<()> {
    std::fs::remove_file(XDG_SHELL_XML)
}

fn main() -> io::Result<()> {
    let exists = files_exist(&["xdg-shell.h", "xdg-shell.c"])?;

    if !exists {
        Command::new("curl")
        .args(["-O", "https://cgit.freedesktop.org/wayland/wayland-protocols/plain/unstable/xdg-shell/xdg-shell-unstable-v6.xml"])
        .status()?;

        generate_files()?;
    }

    let wl_bindings = bindgen::builder()
        .use_core()
        .header("/usr/include/wayland-client.h")
        .header("/usr/include/wayland-server.h")
        .header("/usr/include/wayland-client-protocol.h")
        .header("/usr/include/wayland-egl.h")
        .raw_line("#![allow(unused)]")
        .raw_line("#![allow(nonstandard_style)]")
        .blocklist_item("^FP_.*")
        .generate()
        .unwrap();

    /*
    let xdg_bindings = bindgen::builder()
        .use_core()
        .header("xdg-shell.h")
        .raw_line("#![allow(unused)]")
        .raw_line("#![allow(nonstandard_style)]")
        .blocklist_item("^FP_.*")
        .generate()
        .unwrap();
        */

    wl_bindings.write_to_file("src/wayland/bindings.rs")?;
    //xdg_bindings.write_to_file("src/xdg_bindings.rs")?;

    /*
    cc::Build::new()
        .no_default_flags(false)
        .file("xdg-shell.c")
        .compile("xdg-shell");
        */

    println!("cargo:rustc-link-lib=wayland-client");
    println!("cargo:rustc-link-lib=wayland-server");
    //println!("cargo:rustc-link-lib=wayland-client-protocol");
    println!("cargo:rustc-link-lib=wayland-egl");
    // println!("cargo:rustc-link-lib=ld");

    if !exists {
        cleanup()?;
    }

    Ok(())
}
