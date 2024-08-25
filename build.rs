use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    let input_lib_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("lib");
    let output_path = get_output_path();
    let output_lib_path = Path::new(&output_path).join("lib");
    std::fs::create_dir_all(output_lib_path.clone()).unwrap();

    copy_lib_rsvm_zip(&output_lib_path);

    if cfg!(unix) {
        std::fs::copy(
            input_lib_path.join("unix_rt.jar"),
            output_lib_path.join("rt.jar"),
        )
        .unwrap();
    } else if cfg!(windows) {
        std::fs::copy(
            input_lib_path.join("windows_rt.jar"),
            output_lib_path.join("rt.jar"),
        )
        .unwrap();
    } else {
        unreachable!();
    }
    std::fs::copy(
        input_lib_path.join("charsets.jar"),
        output_lib_path.join("charsets.jar"),
    )
    .unwrap();
}

fn get_output_path() -> PathBuf {
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string)
        .join("target")
        .join(build_type);
    return PathBuf::from(path);
}

fn copy_lib_rsvm_zip(output_lib_path: &PathBuf) {
    let input_lib_rsvm_zip = std::env::var("CARGO_CDYLIB_FILE_RSVM_ZIP").unwrap();
    let input_lib_rsvm_zip_path = PathBuf::from(Path::new(&input_lib_rsvm_zip));
    let output_filename = get_lib_rsvm_zip_output_filename();
    std::fs::copy(
        input_lib_rsvm_zip_path,
        output_lib_path.join(output_filename),
    )
    .unwrap();
}

fn get_lib_rsvm_zip_output_filename() -> String {
    let mut lib_rsvm_zip = String::from("zip");
    if cfg!(target_os = "linux") {
        lib_rsvm_zip.insert_str(0, "lib");
        lib_rsvm_zip.push_str(".so");
    } else if cfg!(target_os = "macos") {
        lib_rsvm_zip.insert_str(0, "lib");
        lib_rsvm_zip.push_str(".dylib");
    } else if cfg!(windows) {
        lib_rsvm_zip.push_str(".dll");
    }
    return lib_rsvm_zip;
}
