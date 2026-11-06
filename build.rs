use std::{
    env,
    path::PathBuf,
    process::{Command, ExitCode},
};

/// Constant for the `new line` value.  
/// - `"\r\n"` in windows
/// - `"\n"` for non windows environments.
///
#[cfg(windows)]
const NEW_LINE: &str = "\r\n";
#[cfg(not(windows))]
const NEW_LINE: &str = "\n";

/// Constant for the resources folder.  
///
const RESOURCE_FOLDER: &str = "res";

/// Constant for the `toml` file extension.  
///
const TOML_EXTENSION: &str = "toml";

/// Constant for the SHA-1 `minbas78` default commit value.  
///
const COMMIT_DEFAULT: &str = "ab6f2c2e2c46130677d8912e759a1532bd6f8e25";

/// Constant for the default commit date value.  
///
const DATE_DEFAULT: &str = "2026-10-20";

/// Constant for the resources filename.  
///
const RESOURCE_FILENAME: &str = "resources.rc";

fn main() -> ExitCode {
    match env::var("CARGO_PKG_NAME") {
        Ok(minbas78) => {
            println!("cargo:rustc-env=MINBAS78={}", minbas78);
        }
        Err(error) => {
            let error_string = error.to_string().to_uppercase();
            println!("{}BUILD ERROR\t{:?}", NEW_LINE, error_string);
            return ExitCode::FAILURE;
        }
    }
    println!("cargo:rustc-env=TOML_EXTENSION={}", TOML_EXTENSION);
    let commit_default = COMMIT_DEFAULT;
    let date_default = DATE_DEFAULT;
    let commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or(commit_default.to_string());
    let date = Command::new("git")
        .args(["log", "-1", "--format=%cs"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or(date_default.to_string());
    println!("cargo:rustc-env=GIT_COMMIT={}", commit);
    println!("cargo:rustc-env=GIT_DATE={}", date);
    println!("cargo:rerun-if-changed=resources.rc");
    let resource_file = PathBuf::from(RESOURCE_FOLDER).join(RESOURCE_FILENAME);
    match embed_resource::compile(resource_file, embed_resource::NONE).manifest_optional() {
        Err(error) => {
            let basic_compile_error = error.to_string().to_uppercase();
            println!("{}BUILD ERROR\t{:?}", NEW_LINE, basic_compile_error);
            ExitCode::FAILURE
        }
        _ => ExitCode::SUCCESS,
    }
}
