//! # Arguments.
//!
//! This module declares the public struct of the arguments command line options.
//!
use clap::Parser;
use std::path::PathBuf;

/// Constant for the `minbas78` value.  
///
pub const MINBAS78: &str = env!("MINBAS78");

/// Constant for the `toml` file extension value.  
///
pub const TOML_EXTENSION: &str = "toml";

/// Constant for the `minbas78` banner value.  
///
const MINBAS78_BANNER: &str = "
▄▄▄▄  ▀ ▄▄▄  █▄▄   ▄▄▄   ▄▄▄ ▀▀▀█ ▄▀▀▄ 
█ █ █ █ █  █ █  █ █  █  ▀▀▄▄  ▄▀  ▄▀▀▄ 
▀ ▀ ▀ ▀ ▀  ▀ ▀▀▀   ▀▀▀▀ ▀▀▀   ▀    ▀▀  
";

/// Constant for the cargo pakage name value.  
///
const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// Constant for the `name` value.  
///
const NAME: &str = const_format::formatcp!("{MINBAS78_BANNER}{CARGO_PKG_NAME}");

/// Constant for the override value of `usage`.  
///
const OVERRIDE_USAGE: &str = concat!(env!("CARGO_PKG_NAME"), " [OPTIONS] INPUT");

/// Constant for the value of the `version`.  
///
const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_COMMIT"),
    " ",
    env!("GIT_DATE"),
    ")"
);

/// Constant for the `help` template.  
///
const HELP_TEMPLATE: &str = "\
{before-help}
{author}Usage: {usage}

Options:
    -h, --help                  Display this message
    -o, --output <FILENAME>     Write output to FILENAME
    --clif                      Generates the Cranelift Intermediate Format
    --dot                       Generates the Intermediate Representation
    --ron                       Generates the Rusty Object Notation
    -V, --version               Print version info and exit

{after-help}";

/// # pub struct `Arguments`.
///
/// Struct for the arguments command line options.
///
#[derive(Parser)]
#[command(name = NAME)]
#[command(version = VERSION)]
#[command(override_usage = OVERRIDE_USAGE)]
#[command(author = MINBAS78_BANNER)]
#[command(help_template = HELP_TEMPLATE)]
pub struct Arguments {
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    #[arg(long)]
    pub clif: bool,

    #[arg(long)]
    pub dot: bool,

    #[arg(long)]
    pub ron: bool,
}
