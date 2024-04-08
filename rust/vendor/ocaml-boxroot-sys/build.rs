/* SPDX-License-Identifier: MIT */

#[cfg(feature = "bundle-boxroot")]
fn build_boxroot() {
    println!("cargo:rerun-if-changed=vendor/boxroot/");
    println!("cargo:rerun-if-env-changed=OCAMLOPT");
    println!("cargo:rerun-if-env-changed=OCAML_WHERE_PATH");
    println!("cargo:rerun-if-env-changed=OCAML_VERSION");

    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let ocaml_version = std::env::var("OCAML_VERSION");
    let ocaml_where_path = std::env::var("OCAML_WHERE_PATH");
    let ocamlopt = std::env::var("OCAMLOPT").unwrap_or_else(|_| "ocamlopt".to_string());

    let version: String;
    let ocaml_path: String;

    match (ocaml_version, ocaml_where_path) {
        (Ok(ver), Ok(path)) => {
            version = ver;
            ocaml_path = path;
        }
        _ => {
            version = std::str::from_utf8(
                std::process::Command::new(&ocamlopt)
                    .arg("-version")
                    .output()
                    .unwrap()
                    .stdout
                    .as_ref(),
            )
            .unwrap()
            .trim()
            .to_owned();
            ocaml_path = std::str::from_utf8(
                std::process::Command::new(&ocamlopt)
                    .arg("-where")
                    .output()
                    .unwrap()
                    .stdout
                    .as_ref(),
            )
            .unwrap()
            .trim()
            .to_owned();
        }
    }

    let mut config = cc::Build::new();

    config.define("BUILD_RS", None);

    let split: Vec<&str> = version.split('.').collect();
    let major = split[0].parse::<usize>().unwrap();
    if major >= 5 {
        config.define("OCAML_VERSION_5", None);
    }

    config.include(&ocaml_path);
    config.include("vendor/boxroot/");
    config.file("vendor/boxroot/boxroot.c");
    config.file("vendor/boxroot/ocaml_hooks.c");
    config.file("vendor/boxroot/platform.c");

    config.compile("libocaml-boxroot.a");

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=ocaml-boxroot");

    #[cfg(feature = "link-ocaml-runtime-and-dummy-program")]
    link_runtime(out_dir, &ocamlopt, &ocaml_path).unwrap();
}

#[cfg(feature = "link-ocaml-runtime-and-dummy-program")]
fn link_runtime(
    out_dir: std::path::PathBuf,
    ocamlopt: &str,
    ocaml_path: &str,
) -> std::io::Result<()> {
    use std::io::Write;

    let mut f = std::fs::File::create(out_dir.join("empty.ml")).unwrap();
    write!(f, "")?;

    assert!(std::process::Command::new(&ocamlopt)
        .args(&["-output-obj", "-o"])
        .arg(out_dir.join("dummy.o"))
        .arg(out_dir.join("empty.ml"))
        .status()?
        .success());

    let ar = std::env::var("AR").unwrap_or_else(|_| "ar".to_string());
    assert!(std::process::Command::new(&ar)
        .arg("rcs")
        .arg(out_dir.join("libdummy.a"))
        .arg(out_dir.join("dummy.o"))
        .status()?
        .success());

    let cc_libs: Vec<String> = std::str::from_utf8(
        std::process::Command::new(&ocamlopt)
            .args(&["-config-var", "native_c_libraries"])
            .output()
            .unwrap()
            .stdout
            .as_ref(),
    )
    .unwrap()
    .to_owned()
    .split_whitespace()
    .map(|s| { assert!(&s[0..2] == "-l"); String::from(&s[2..]) })
    .collect();

    for lib in cc_libs {
        println!("cargo:rustc-link-lib={}", lib);
    }

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=dummy");

    println!("cargo:rustc-link-search={}", ocaml_path);
    println!("cargo:rustc-link-lib=dylib=asmrun");

    Ok(())
}

fn main() {
    #[cfg(feature = "bundle-boxroot")]
    build_boxroot();
}
