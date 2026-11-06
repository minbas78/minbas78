//! # Abstract Syntax Tree Utils.
//!
//! This module declares and implements the external numeric function reference struct, constants and errors.
//!
use crate::external_functions;
use crate::utils;
use anyhow::Error;
use anyhow::Result;
use cranelift::frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_codegen::ir::immediates::Ieee64;
use cranelift_codegen::{
    ir::{AbiParam, FuncRef, Function, Type, UserFuncName, types},
    isa::{self, CallConv},
    settings::{self, Configurable},
};
use cranelift_module::{FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use pest::Span;
use std::collections::HashMap;
use std::{ffi::CString, process::ExitCode};
use thiserror::Error as ThisError;

/// Constant for the `new line` value.  
/// - `"\r\n"` in windows
/// - `"\n"` for non windows environments.
///
#[cfg(windows)]
pub const NEW_LINE: &str = "\r\n";
#[cfg(not(windows))]
pub const NEW_LINE: &str = "\n";

/// Constant name for the `main` function.
///
pub const MAIN: &str = "main";

/// Constant name for the linker `target` architecture.
///
pub const TARGET: &str = "x86_64-pc-windows-msvc";

/// Constant name for the linker `tool` filename.
///
pub const TOOL: &str = "link.exe";

/// Constant name for the `global data` variable.
///
pub const GLOBAL_DATA: &str = "GLOBAL_DATA";

/// Constant name for the `global data index` variable.
///
pub const GLOBAL_DATA_INDEX: &str = "GLOBAL_DATA_INDEX";

/// Constant name for the `global data length` variable.
///
pub const GLOBAL_DATA_LEN: &str = "GLOBAL_DATA_LEN";

/// Constant name for the `seed` variable.
///
pub const SEED: &str = "SEED";

/// Constant name for the `"$"` character.
///
pub const DOLLAR_SIGN: &str = "$";

/// Constant name for the `'|'` character.
///
pub const PIPE: &str = "|";

/// Constant name for the `""` empty string.
///
pub const EMPTY_STRING: &str = "";

/// Constant name for the `','` character.
///
pub const COMMA: &str = ",";

/// Constant name for the `';'` character.
///
pub const SEMI_COLON: &str = ";";

/// Constant name for the `'.'` character.
///
pub const FULL_STOP: &str = ".";

/// Constant name for the `'"'` character.
///
pub const QUOTATION_MARK: char = '"';

/// Constant name for the `'('` character.
///
pub const LEFT_PARENTHESIS: char = '(';

/// Constant name for the `')'` character.
///
pub const RIGHT_PARENTHESIS: char = ')';

/// Constant name for the `'/'` character.
///
pub const SOLIDUS: &str = "/";

/// Constant name for the `"="` character of the `equals_sign` grammar rule.
///
pub const EQUALS_SIGN: &str = "=";

/// Constant name for the `"<>"` characters of the `not_equals` grammar rule.
///
pub const NOT_EQUALS: &str = "<>";

/// Constant name for the `">"` character of the `greater_than_sign` grammar rule.
///
pub const GREATER_THAN_SIGN: &str = ">";

/// Constant name for the `">="` characters of the `not_less` grammar rule.
///
pub const NOT_LESS: &str = ">=";

/// Constant name for the `"<"` character of the `less_than_sign` grammar rule.
///
pub const LESS_THAN_SIGN: &str = "<";

/// Constant name for the `"<="` characters of the `not_greater` grammar rule.
///
pub const NOT_GREATER: &str = "<=";

/// Constant name for the `"0"` character.
///
pub const ZERO: &str = "0";

/// Constant name for the `"-"` character `minus_sign` grammar rule.
///
pub const MINUS_SIGN: &str = "-";

/// Constant name for the `" "` character of the `space` grammar rule.
///
pub const SPACE: &str = " ";

/// Constant name for the `"E"` character of the exponential representation.
///
pub const E: &str = "E";

/// Constant name for the input prefix.
///
pub const INPUT_PREFIX: &str = "? ";

/// Constant name for the base constant of the `numeric_rep` grammar rule.
///
pub const BASE_10: f64 = 10.0_f64;

/// Constant name for the initial rows length of the `DIM` statement.
///
pub const ROWS_MAX: u8 = 10_u8;

/// Constant name for the initial columns length of the `DIM` statement.
///
pub const COLUMNS_MAX: u8 = 10_u8;

/// Constant name for the initial characters length of the `LET` statement for the `string_let_statement` grammar rule.
///
pub const STRING_MAX: usize = 18_usize;

/// Constant name for the prefix value of the local `FOR` variables.
///
pub const LOCAL_FOR_PREFIX_VARIABLE: &str = "";

/// Enum for the external numeric functions references.  
///
pub enum NumericFunctionRefExternal {
    /// - `ABS(X)`: the absolute value of `X`.
    ///
    ABS,

    /// - `ATN(X)`: The arctangent of `X` in radians. i.e. the angle whose tangent is `X`. The range of the function is: `-(pi/2) < ATN(X) < (pi/2)`.
    ///
    ATN,

    /// - `COS(X)`: The cosine of `X`, where `X` is in radians.
    ///
    COS,

    /// - `EXP(X)`: The exponential of `X`, i.e. the value of the base of natural logarithms (`e = 2,71828 ...`) raised to the power `X`; if `EXP(X)` is less than machine infinitesimal, then its value will be replaced by `zero`.
    ///
    EXP,

    /// - `INT(X)`: The largest integer not greater than `X`; e.g. `INT(1.3) = 1` and `INT(-1.3) = -2`.
    ///
    INT,

    /// - `LOG(X)`: The natural logarithm of `X`; `X` must be greater than `zero`.
    ///
    LOG,

    /// - `RND`   : The next pseudo-random number in an implementation-supplied sequence of pseudo-random numbers uniformly distributed in the range `0 <= RND < 1`.
    ///
    RND,

    /// - `SGN(X)`: The sign of `X`: `-1` if `X < 0`, `0` if `X = 0` and `+1` if `X > 0`.
    ///
    SGN,

    /// - `SIN(X)`: The sine of `X`, where `X` is in radians.
    ///
    SIN,

    /// - `SQR(X)`: The nonnegative square root of `X`; `X` must be nonnegative.
    ///
    SQR,

    /// - `TAN(X)`: The tangent of `X`, where `X` is in radians.
    ///
    TAN,
}

/// Implementation for `as_ref` and `as_value` methods.  
///
impl NumericFunctionRefExternal {
    /// # pub fn as_str
    ///
    /// Helper function for the external functions.
    ///
    /// _This function returns the &str representation
    /// of a NumericFunctionRefExternal struct._
    ///
    /// ## Examples
    ///
    /// ```
    /// # use minbas78::utils::NumericFunctionRefExternal;
    /// # use cranelift_codegen::ir::immediates::Ieee64;
    /// let numeric_functions_ref_external = vec![
    ///     NumericFunctionRefExternal::ABS,
    ///     NumericFunctionRefExternal::ATN,
    ///     NumericFunctionRefExternal::COS,
    ///     NumericFunctionRefExternal::EXP,
    ///     NumericFunctionRefExternal::INT,
    ///     NumericFunctionRefExternal::LOG,
    ///     NumericFunctionRefExternal::RND,
    ///     NumericFunctionRefExternal::SGN,
    ///     NumericFunctionRefExternal::SIN,
    ///     NumericFunctionRefExternal::SQR,
    ///     NumericFunctionRefExternal::TAN,
    /// ];
    /// numeric_functions_ref_external.iter().enumerate().for_each(|(i, numeric_function_ref_external)| {
    ///     let str_value = numeric_function_ref_external.as_str();
    ///     let str_assert_value = match numeric_function_ref_external {
    ///         NumericFunctionRefExternal::ABS => "ABS",
    ///         NumericFunctionRefExternal::ATN => "ATN",
    ///         NumericFunctionRefExternal::COS => "COS",
    ///         NumericFunctionRefExternal::EXP => "EXP",
    ///         NumericFunctionRefExternal::INT => "INT",
    ///         NumericFunctionRefExternal::LOG => "LOG",
    ///         NumericFunctionRefExternal::RND => "RND",
    ///         NumericFunctionRefExternal::SGN => "SGN",
    ///         NumericFunctionRefExternal::SIN => "SIN",
    ///         NumericFunctionRefExternal::SQR => "SQR",
    ///         NumericFunctionRefExternal::TAN => "TAN",
    ///     };
    ///     assert_eq!(str_value, str_assert_value);
    /// });
    /// ```
    ///
    pub fn as_str(&self) -> &str {
        match self {
            Self::ABS => "ABS",
            Self::ATN => "ATN",
            Self::COS => "COS",
            Self::EXP => "EXP",
            Self::INT => "INT",
            Self::LOG => "LOG",
            Self::RND => "RND",
            Self::SGN => "SGN",
            Self::SIN => "SIN",
            Self::SQR => "SQR",
            Self::TAN => "TAN",
        }
    }

    /// # pub fn as_value
    ///
    /// Helper function for the external functions.
    ///
    /// _This function returns the Ieee64 representation
    /// of a NumericFunctionRefExternal struct._
    ///
    /// ## Examples
    ///
    /// ```
    /// # use minbas78::utils::NumericFunctionRefExternal;
    /// # use cranelift_codegen::ir::immediates::Ieee64;
    /// let numeric_functions_ref_external = vec![
    ///     NumericFunctionRefExternal::ABS,
    ///     NumericFunctionRefExternal::ATN,
    ///     NumericFunctionRefExternal::COS,
    ///     NumericFunctionRefExternal::EXP,
    ///     NumericFunctionRefExternal::INT,
    ///     NumericFunctionRefExternal::LOG,
    ///     NumericFunctionRefExternal::RND,
    ///     NumericFunctionRefExternal::SGN,
    ///     NumericFunctionRefExternal::SIN,
    ///     NumericFunctionRefExternal::SQR,
    ///     NumericFunctionRefExternal::TAN,
    /// ];
    /// numeric_functions_ref_external.iter().enumerate().for_each(|(i, numeric_function_ref_external)| {
    ///     let ieee64_value = numeric_function_ref_external.as_value();
    ///     assert_eq!(ieee64_value, Ieee64::with_bits((i as f64).to_bits()));
    /// });
    /// ```
    ///
    pub fn as_value(&self) -> Ieee64 {
        match self {
            Self::ABS => Ieee64::with_bits(0_f64.to_bits()),
            Self::ATN => Ieee64::with_bits(1_f64.to_bits()),
            Self::COS => Ieee64::with_bits(2_f64.to_bits()),
            Self::EXP => Ieee64::with_bits(3_f64.to_bits()),
            Self::INT => Ieee64::with_bits(4_f64.to_bits()),
            Self::LOG => Ieee64::with_bits(5_f64.to_bits()),
            Self::RND => Ieee64::with_bits(6_f64.to_bits()),
            Self::SGN => Ieee64::with_bits(7_f64.to_bits()),
            Self::SIN => Ieee64::with_bits(8_f64.to_bits()),
            Self::SQR => Ieee64::with_bits(9_f64.to_bits()),
            Self::TAN => Ieee64::with_bits(10_f64.to_bits()),
        }
    }
}

/// # pub fn `create_module`.
///
/// Creates the Cranelift object module.
///
/// _This function returns a tuple with the
/// object module and the pointer type._
///
/// ## Examples
///
/// ```
/// # use minbas78::utils::create_module;
/// # use cranelift_codegen::ir::types;
/// let (module, pointer_type) = create_module().unwrap();
/// assert_eq!(pointer_type, types::I64);
/// ```
///
/// # Errors
///
/// This function returns an **anyhow::Error** if
/// it cannot create the object module.
///
pub fn create_module() -> Result<(ObjectModule, Type)> {
    let mut flag_builder = settings::builder();
    flag_builder.enable("is_pic")?;
    flag_builder.set("opt_level", "speed_and_size")?;
    let flags = settings::Flags::new(flag_builder);
    let isa_builder = isa::lookup_by_name(utils::TARGET)?;
    let isa = isa_builder.finish(flags)?;
    let obj_builder = ObjectBuilder::new(
        isa.clone(),
        utils::MAIN,
        cranelift_module::default_libcall_names(),
    )?;
    let module = ObjectModule::new(obj_builder);
    let pointer_type = module.target_config().pointer_type();
    Ok((module, pointer_type))
}

/// # fn `create_main_function`.
///
/// Creates the main function.
///
/// _This function takes the Cranelift module,
/// and returns a tuple with the function id,
/// the function and the function builder context._
///
/// ## Examples
///
/// ```
/// # use minbas78::utils::{create_main_function, create_module};
/// # use cranelift_codegen::entity::EntityRef;
/// let (mut module, _) = create_module().unwrap();
/// let (func_id, func, _) = create_main_function(&mut module).unwrap();
/// assert_eq!(func_id.index(), usize::default());
/// assert_eq!(func.name.to_string(), "u0:0");
/// ```
///
/// # Errors
///
/// This function returns an **anyhow::Error** if
/// it cannot declare the main function.
///
/// # Safety
///
pub fn create_main_function(
    module: &mut ObjectModule,
) -> Result<(FuncId, Function, FunctionBuilderContext)> {
    let mut signature = module.make_signature();
    signature.call_conv = CallConv::SystemV;
    signature.returns.push(AbiParam::new(types::I32));
    let func_id = module.declare_function(utils::MAIN, Linkage::Export, &signature)?;
    let user_func_name = UserFuncName::user(0, func_id.as_u32());
    let function = Function::with_name_signature(user_func_name, signature);
    let function_builder_context = FunctionBuilderContext::new();
    Ok((func_id, function, function_builder_context))
}

/// # pub fn `create_external_functions`.
///
/// Creates the hashmap of external functions.
///
/// _This function takes the Cranelift module,
/// the function builder, the pointer type
/// and returns a hashmap where the key is the
/// name of the external function and the value
/// is the function reference._
///
/// ## Examples
///
/// ```
/// # use minbas78::utils::{create_external_functions, create_main_function, create_module};
/// # use cranelift::frontend::FunctionBuilder;
/// # use cranelift_codegen::entity::EntityRef;
/// const EXTERNAL_FUNCTIONS_LEN: usize = 16;
/// let (mut module, pointer_type) = create_module().unwrap();
/// let (func_id, mut func, mut func_ctx) = create_main_function(&mut module).unwrap();
/// let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
/// let external_functions = create_external_functions(&mut module, &mut builder, &pointer_type).unwrap();
/// assert_eq!(external_functions.len(), EXTERNAL_FUNCTIONS_LEN);
/// ```
///
/// # Errors
///
/// This function returns an **anyhow::Error** if
/// it cannot create any of the external functions.
///
pub fn create_external_functions(
    module: &mut ObjectModule,
    builder: &mut FunctionBuilder,
    pointer_type: &Type,
) -> Result<HashMap<String, FuncRef>> {
    let mut external_functions = HashMap::<String, FuncRef>::new();

    let mut signature = module.make_signature();
    signature.params.push(AbiParam::new(*pointer_type));
    signature.params.push(AbiParam::new(types::I64));
    let mut func_id = module.declare_function("input_external", Linkage::Import, &signature)?;
    let mut func_ref = module.declare_func_in_func(func_id, &mut builder.func);
    external_functions.insert("input_external".to_string(), func_ref);

    signature = module.make_signature();
    signature.params.push(AbiParam::new(*pointer_type));
    signature.params.push(AbiParam::new(types::I64));
    signature.params.push(AbiParam::new(types::I8));
    signature.returns.push(AbiParam::new(types::I64));
    func_id = module.declare_function("print_external", Linkage::Import, &signature)?;
    func_ref = module.declare_func_in_func(func_id, &mut builder.func);
    external_functions.insert("print_external".to_string(), func_ref);

    signature = module.make_signature();
    signature.params.push(AbiParam::new(*pointer_type));
    signature.params.push(AbiParam::new(*pointer_type));
    signature.returns.push(AbiParam::new(types::I8));
    func_id = module.declare_function("strcmp_external", Linkage::Import, &signature)?;
    func_ref = module.declare_func_in_func(func_id, &mut builder.func);
    external_functions.insert("strcmp_external".to_string(), func_ref);

    signature = module.make_signature();
    signature.params.push(AbiParam::new(types::F64));
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature
        .returns
        .push(AbiParam::new(module.target_config().pointer_type()));
    func_id =
        module.declare_function("f64_to_pointer_value_external", Linkage::Import, &signature)?;
    func_ref = module.declare_func_in_func(func_id, &mut builder.func);
    external_functions.insert("f64_to_pointer_value_external".to_string(), func_ref);

    signature = module.make_signature();
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature
        .params
        .push(AbiParam::new(module.target_config().pointer_type()));
    signature
        .returns
        .push(AbiParam::new(module.target_config().pointer_type()));
    func_id =
        module.declare_function("pointer_value_to_f64_external", Linkage::Import, &signature)?;
    func_ref = module.declare_func_in_func(func_id, &mut builder.func);
    external_functions.insert("pointer_value_to_f64_external".to_string(), func_ref);

    signature = module.make_signature();
    signature.params.push(AbiParam::new(*pointer_type));
    signature.params.push(AbiParam::new(*pointer_type));
    signature.returns.push(AbiParam::new(types::F64));
    func_id = module.declare_function("get_term_external", Linkage::Import, &signature)?;
    func_ref = module.declare_func_in_func(func_id, &mut builder.func);
    external_functions.insert("get_term_external".to_string(), func_ref);

    signature = module.make_signature();
    signature.params.push(AbiParam::new(types::F64));
    signature.params.push(AbiParam::new(types::F64));
    signature.returns.push(AbiParam::new(types::F64));
    func_id = module.declare_function("powf_external", Linkage::Import, &signature)?;
    func_ref = module.declare_func_in_func(func_id, &mut builder.func);
    external_functions.insert("powf_external".to_string(), func_ref);

    signature = module.make_signature();
    signature.params.push(AbiParam::new(types::F64));
    signature.params.push(AbiParam::new(types::F64));
    signature.returns.push(AbiParam::new(types::F64));
    func_id = module.declare_function(
        "get_numeric_function_ref_external",
        Linkage::Import,
        &signature,
    )?;
    func_ref = module.declare_func_in_func(func_id, &mut builder.func);
    external_functions.insert("get_numeric_function_ref_external".to_string(), func_ref);

    let c_external_functions = vec![
        "atn_external",
        "cos_external",
        "exp_external",
        "log_external",
        "rnd_external",
        "sgn_external",
        "sin_external",
        "tan_external",
    ];
    c_external_functions.iter().copied().try_for_each(|ef| {
        signature = module.make_signature();
        signature.params.push(AbiParam::new(types::F64));
        signature.returns.push(AbiParam::new(types::F64));
        func_id = module.declare_function(ef, Linkage::Import, &signature)?;
        func_ref = module.declare_func_in_func(func_id, &mut builder.func);
        external_functions.insert(ef.to_string(), func_ref);
        anyhow::Ok(())
    })?;

    Ok(external_functions)
}

/// # pub fn `span_into_string`.
///
/// Helper function for the abstract syntax tree.
///
/// _This function returns the `String` representation
/// of a `Span`._
///
/// ## Examples
///
/// ```
/// # use minbas78::utils::span_into_string;
/// # use pest::Span;
/// let span_str = "span";
/// let span = Span::new(span_str, usize::default(), span_str.len()).unwrap();
/// let span_str_return = span_into_string(span);
/// assert_eq!(span_str_return, String::from(span_str));
/// ```
///
pub fn span_into_string(span: Span) -> String {
    span.as_str().to_string()
}

/// # pub enum `MinBas78Error`.
///
/// Enum for the Minimal BASIC 78 error types:
/// - `SystemError`
/// - `CompilerError`
/// - `RuntimeError`
///
#[derive(ThisError, Debug)]
pub enum MinBas78Error {
    #[error("{}SYSTEM ERROR{}{}", 
        utils::NEW_LINE,
        filenames.iter().fold(String::new(), |filename_accumulator, filename| format!("{}{}{}", filename_accumulator, utils::PIPE, filename)),
        message.as_ref().map(|msg| format!("{}{}", utils::PIPE, msg.to_uppercase())).unwrap_or_default()
    )]
    SystemError {
        filenames: Vec<String>,
        message: Option<String>,
    },
    #[error("{}COMPILER ERROR{}{}", utils::NEW_LINE, utils::PIPE, message.to_uppercase())]
    CompilerError { message: String },
    #[error("{}RUNTIME ERROR{}{}", utils::NEW_LINE, utils::PIPE, message.to_uppercase())]
    RuntimeError { message: String },
}

impl MinBas78Error {
    /// # pub fn `exit_code`.
    ///
    /// Helper function for the Basic compiler error exit code.
    ///
    /// _This function returns the equivalent exit code
    /// for the Basic compiler error._
    ///
    /// ## Examples
    ///
    /// ```
    /// # use std::process::ExitCode;
    /// # use minbas78::utils::MinBas78Error;
    /// let basic_compiler_error = MinBas78Error::RuntimeError { message: String::from("error") };
    /// assert_eq!(basic_compiler_error.exit_code(), ExitCode::FAILURE);
    /// ```
    ///
    pub fn exit_code(&self) -> ExitCode {
        match self {
            MinBas78Error::SystemError {
                filenames: _,
                message: _,
            } => ExitCode::FAILURE,
            MinBas78Error::CompilerError { message: _ } => ExitCode::FAILURE,
            MinBas78Error::RuntimeError { message: _ } => ExitCode::FAILURE,
        }
    }

    /// # pub fn `eprint_error`.
    ///
    /// Helper function for the Basic compiler error print.
    ///
    /// _This function prints the Basic compiler error._
    ///
    pub fn eprint_error(error: &Error) -> Result<()> {
        let mut error_string = error.to_string();
        error_string.push_str(utils::NEW_LINE);
        let mut error_string_splits = error_string.split(utils::PIPE);
        let mut print_column = i64::default();
        error_string_splits.try_for_each(|error_string_split| {
            let mut error_string_split_cstring = CString::new(error_string_split)?;
            let mut error_string_split_cstring_pointer = error_string_split_cstring.as_ptr();
            print_column = external_functions::print_external(
                error_string_split_cstring_pointer,
                print_column as u64,
                false,
            );
            error_string_split_cstring = CString::new(utils::COMMA)?;
            error_string_split_cstring_pointer = error_string_split_cstring.as_ptr();
            print_column = external_functions::print_external(
                error_string_split_cstring_pointer,
                print_column as u64,
                false,
            );
            anyhow::Ok(())
        })?;
        Ok(())
    }
}
