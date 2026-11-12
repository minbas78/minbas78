//! # Abstract Syntax Tree Visitor.
//!
//! This module declares the implementation of the visitor trait for the abstract syntax tree walker.
//!
use crate::abstract_syntax_tree::{
    DefStatement, Expression, Exrad, Factor, Fraction, Integer, Multiplier, NumericArrayName,
    NumericDefinedFunction, NumericExpression, NumericFunctionRef, NumericRep, OptionBase,
    PrintSeparator, Program, Sign, Significand, SimpleNumericVariable, StringConstant,
    StringExpression, StringVariable,
};
use crate::grammars::basic_pest::{MinBas78, Rule as RuleBasic};
use crate::grammars::basic_treemap_pest::MinBas78UnorderedLines;
use crate::utils::MinBas78Error;
use crate::utils::{self, NumericFunctionRefExternal};
use crate::{arguments, grammars, user_defined_function_walker};
use anyhow::{Result, anyhow};
use config::{Config, File as ConfigFile};
use cranelift::frontend::{FunctionBuilder, Switch, Variable};
use cranelift_codegen::cfg_printer::CFGPrinter;
use cranelift_codegen::ir::condcodes::{FloatCC, IntCC};
use cranelift_codegen::ir::{
    Block, BlockArg, FuncRef, InstBuilder, MemFlagsData, StackSlotData, StackSlotKind, TrapCode,
    Type, Value, types,
};
use cranelift_module::{DataDescription, Module};
use cranelift_object::ObjectModule;
use from_pest::FromPest;
use grammars::basic_treemap_pest::Rule as RuleMinBas78UnorderedLines;
use pest::Parser;
use std::any::Any;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;
use std::{fs, mem, time};
use std::{fs::File, path::PathBuf};

/// pub struct `GlobalVariableDimension`.
///
/// Struct for the global variable dimension.  
///
#[derive(Debug, Copy, Clone)]
pub struct GlobalVariableDimension {
    pub rows: u8,
    pub columns: Option<u8>,
}

impl GlobalVariableDimension {
    /// # pub fn get_variable_dimension
    ///
    /// Implementation of the `get_variable_dimension` function of the
    /// `GlobalVariableDimension` struct.
    ///
    /// _This function takes the string variable with dimensions
    /// and returns a tuple with the variable name, the `GlobalVariableDimension` struct and `true` if is a matrix)._
    ///
    /// ## Examples
    ///
    /// ```
    /// # use minbas78::visitor::GlobalVariableDimension;
    /// let (name, global_variable_dimension, is_matrix) = GlobalVariableDimension::get_variable_dimension(&String::from("A(1,2)"));
    /// assert_eq!(name, String::from("A"));
    /// assert_eq!(global_variable_dimension, GlobalVariableDimension {
    ///     rows: 1,
    ///     columns: Some(2),
    /// });
    /// assert_eq!(is_matrix, true);
    /// ```
    ///
    pub fn get_variable_dimension(variable: &String) -> (String, GlobalVariableDimension, bool) {
        if let Some(open_paren) = variable.find(utils::LEFT_PARENTHESIS) {
            let name = variable[..open_paren].trim().to_string();
            let close_paren = variable.find(')').unwrap_or(variable.len());
            let inside = &variable[open_paren + 1..close_paren];
            let parts: Vec<&str> = inside.split(',').map(|s| s.trim()).collect();
            let d1 = parts.get(0).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
            let d2 = parts.get(1).and_then(|s| s.parse::<u8>().ok());
            (
                name,
                GlobalVariableDimension {
                    rows: d1,
                    columns: d2,
                },
                true,
            )
        } else {
            (
                variable.clone(),
                GlobalVariableDimension {
                    rows: 0,
                    columns: None,
                },
                false,
            )
        }
    }
}

impl PartialEq for GlobalVariableDimension {
    /// # fn eq
    ///
    /// Implementation function of the
    /// PartialEq trait for the GlobalVariableDimension struct.
    ///
    /// _This function takes two global variable dimensions
    /// and returns a bool indicating if both of them are equals._
    ///
    /// ## Examples
    ///
    /// ```
    /// # use minbas78::visitor::GlobalVariableDimension;
    /// let a = GlobalVariableDimension {
    ///     rows: u8::default(),
    ///     columns: Some(u8::default()),
    /// };
    /// let b = GlobalVariableDimension {
    ///     rows: u8::default(),
    ///     columns: Some(u8::default()),
    /// };
    /// assert_eq!(a, b);
    /// ```
    ///
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.columns == other.columns
    }
}

impl PartialOrd for GlobalVariableDimension {
    /// # fn partial_cmp
    ///
    /// Implementation function of the
    /// PartialOrd trait for the GlobalVariableDimension struct.
    ///
    /// _This function takes two global variable dimensions
    /// and returns a bool indicating if both of them are
    /// partial ordered:_ `lt`, `le`, `gt` or `ge`.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use minbas78::visitor::GlobalVariableDimension;
    /// # use cranelift_codegen::ir::Type;
    /// let a = GlobalVariableDimension {
    ///     rows: u8::default(),
    ///     columns: Some(u8::default()),
    /// };
    /// let b = GlobalVariableDimension {
    ///     rows: u8::default(),
    ///     columns: None,
    /// };
    /// assert!((|a: GlobalVariableDimension, b: GlobalVariableDimension| {a.ge(&b)})(a, b));
    /// ```
    ///
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.rows.partial_cmp(&other.rows) {
            Some(Ordering::Equal) | Some(Ordering::Less) => {}
            ord => return ord,
        }
        match (self.columns, other.columns) {
            (None, None) => Some(Ordering::Equal),
            (None, Some(_)) => Some(Ordering::Less),
            (Some(_), None) => Some(Ordering::Greater),
            (Some(a), Some(b)) => a.partial_cmp(&b),
        }
    }
}

/// struct `GlobalVariable`.
///
/// Struct for the global variable.  
///
#[derive(Debug, Copy, Clone)]
struct GlobalVariable {
    variable: Variable,
    dimension: GlobalVariableDimension,
}

/// # pub struct `Visitor`.
///
/// Struct for the Basic abstract syntax tree visitor.
///
pub struct Visitor<'a> {
    pub builder: &'a mut FunctionBuilder<'a>,
    pub blocks: HashMap<String, Block>,
    pointer_type: Type,
    module: &'a mut ObjectModule,
    external_functions: HashMap<String, FuncRef>,
    user_defined_functions: HashMap<String, (Option<String>, NumericExpression)>,
    global_variables: HashMap<String, GlobalVariable>,
    global_variables_dimensions: HashMap<String, GlobalVariableDimension>,
    global_data: Vec<String>,
    global_data_index: usize,
    option_base: u8,
    print_column: Option<Variable>,
    gosub_stack: Vec<String>,
    pre_gosub: bool,
    pre_for_to_step: bool,
    for_to_step_stack: Vec<String>,
    for_to_step_block_stack: Vec<(String, Block, Block)>,
    previous_line: String,
}

impl<'a> Visitor<'a> {
    pub fn new(
        pointer_type: &Type,
        builder: &'a mut FunctionBuilder<'a>,
        module: &'a mut ObjectModule,
        external_functions: &HashMap<String, FuncRef>,
    ) -> Self {
        let global_variables = HashMap::new();
        let global_variables_dimensions = HashMap::new();
        let global_data = Vec::new();
        let user_defined_functions = HashMap::new();
        let blocks = HashMap::new();
        let gosub_stack = Vec::new();
        let pre_gosub = false;
        let pre_for_to_step = false;
        let for_to_step_stack = Vec::new();
        let for_to_step_block_stack = Vec::new();
        let previous_line = String::default();
        Self {
            pointer_type: *pointer_type,
            builder,
            module,
            external_functions: external_functions.clone(),
            user_defined_functions,
            global_variables,
            global_variables_dimensions,
            global_data,
            global_data_index: usize::default(),
            option_base: u8::default(),
            print_column: None,
            blocks,
            gosub_stack,
            pre_gosub,
            pre_for_to_step,
            for_to_step_stack,
            for_to_step_block_stack,
            previous_line,
        }
    }

    pub fn get_minbas78_settings() -> Result<HashMap<String, String>> {
        let minbas78_settings_filename = Path::new(arguments::MINBAS78)
            .with_extension(arguments::TOML_EXTENSION)
            .into_string()
            .map_err(|path_buffer| MinBas78Error::RuntimeError {
                message: format!("{:?} not found.", path_buffer),
            })?;
        let minbas78_settings = Config::builder()
            .add_source(ConfigFile::with_name(&minbas78_settings_filename))
            .build()?
            .try_deserialize::<HashMap<String, String>>()?;
        Ok(minbas78_settings)
    }

    pub fn get_program(input_bas_file: &Path) -> Result<Program> {
        let input_bas_string =
            fs::read_to_string(&input_bas_file).map_err(|error| MinBas78Error::SystemError {
                filenames: vec![input_bas_file.to_string_lossy().to_string()],
                message: Some(error.to_string()),
            })?;
        let input_bas_string_sorted = Visitor::get_input_bas_string_sorted(&input_bas_string)?;
        let mut pairs =
            MinBas78::parse(RuleBasic::program, &input_bas_string_sorted).map_err(|error| {
                MinBas78Error::SystemError {
                    filenames: vec![],
                    message: Some(error.to_string()),
                }
            })?;
        let program =
            Program::from_pest(&mut pairs).map_err(|error| MinBas78Error::CompilerError {
                message: error.to_string(),
            })?;
        Ok(program)
    }

    pub fn get_input_bas_string_sorted(input_bas_string: &String) -> Result<String> {
        let mut input_bas_unordered_lines = BTreeMap::<u32, String>::new();
        let pairs_unordered_lines =
            MinBas78UnorderedLines::parse(RuleMinBas78UnorderedLines::program, &input_bas_string)
                .map_err(|error: pest::error::Error<RuleMinBas78UnorderedLines>| {
                MinBas78Error::SystemError {
                    filenames: vec![],
                    message: Some(error.to_string()),
                }
            })?;
        for pair in pairs_unordered_lines {
            for pair_unordered_lines_rules in pair.into_inner() {
                if pair_unordered_lines_rules.as_rule() == RuleMinBas78UnorderedLines::line {
                    let statement = pair_unordered_lines_rules.as_str().to_string();
                    let mut line_number = u32::default();
                    for pair_unordered_lines_rule in pair_unordered_lines_rules.into_inner() {
                        if pair_unordered_lines_rule.as_rule()
                            == RuleMinBas78UnorderedLines::line_number
                        {
                            line_number = pair_unordered_lines_rule.as_str().parse::<u32>()?
                        }
                    }
                    input_bas_unordered_lines.insert(line_number, statement);
                }
            }
        }
        Ok(input_bas_unordered_lines.into_values().collect::<String>())
    }

    fn _log(&mut self, log: &String) -> Result<()> {
        let zero_i64 = self.builder.ins().iconst(types::I64, i64::default());
        let zero_i8 = self.builder.ins().iconst(types::I8, i64::default());
        let aa = self.push_string_stack_data(&log)?;
        self.print_external(&aa, &zero_i64, &zero_i8)?;
        Ok(())
    }

    fn array_dim(&mut self, var_name: &String, dimension: &GlobalVariableDimension) -> Result<()> {
        let rows = (dimension.rows + self.option_base + 1) as usize;
        let cols = match dimension.columns {
            Some(c) => (c + self.option_base + 1) as usize,
            None => 1,
        };
        let num_items = rows * cols;
        let bytes_per_f64 = 8;
        let array_size_in_bytes = num_items * bytes_per_f64;
        let mut data_ctx = DataDescription::new();
        data_ctx.define_zeroinit(array_size_in_bytes as usize);
        let array_data_id = self.module.declare_anonymous_data(true, false)?;
        self.module.define_data(array_data_id, &data_ctx)?;
        let local_data = self
            .module
            .declare_data_in_func(array_data_id, &mut self.builder.func);
        let base_ptr = self.builder.ins().symbol_value(types::I64, local_data);
        let vv = self.builder.declare_var(self.pointer_type);
        self.builder.def_var(vv, base_ptr);
        let cols_1 = match dimension.columns {
            Some(c) => Some(c + self.option_base + 1),
            None => None,
        };
        self.global_variables.insert(
            var_name.clone(),
            GlobalVariable {
                variable: vv,
                dimension: GlobalVariableDimension {
                    rows: rows as u8,
                    columns: cols_1,
                },
            },
        );
        self.global_variables_dimensions.insert(
            var_name.clone(),
            GlobalVariableDimension {
                rows: rows as u8,
                columns: cols_1,
            },
        );
        Ok(())
    }

    fn array_get(
        &mut self,
        var_name: &String,
        dimension: &GlobalVariableDimension,
    ) -> Result<Value> {
        let row = dimension.rows + self.option_base + 1;
        let col = match dimension.columns {
            Some(c) => c + self.option_base + 1,
            None => 1,
        };
        let bytes_per_f64 = 8;
        let vv = self
            .global_variables
            .get(var_name)
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("la variable {:?} no existe", var_name),
            })?;
        let vvv = vv.variable;
        let vvvv = vv.dimension;
        let base_ptr = self.builder.use_var(vvv);
        let element_size = self
            .builder
            .ins()
            .iconst(self.pointer_type, bytes_per_f64 as i64);
        //k_val * element_size
        let index_val = self.builder.ins().iconst(
            types::I64,
            // position = columns(x-1) + y
            ((vvvv.columns.unwrap_or(1) * (row - 1)) + col) as i64,
        );
        let byte_offset = self.builder.ins().imul(index_val, element_size);
        let element_addr = self.builder.ins().iadd(base_ptr, byte_offset);
        let value = self
            .builder
            .ins()
            .load(types::F64, MemFlagsData::new(), element_addr, 0);
        Ok(value)
    }

    fn array_set(
        &mut self,
        var_name: &String,
        dimension: &GlobalVariableDimension,
        set_val: &Value,
    ) -> Result<()> {
        let row = dimension.rows + self.option_base + 1;
        let col = match dimension.columns {
            Some(c) => c + self.option_base + 1,
            None => 1,
        };
        let bytes_per_f64 = 8;
        let vv = self
            .global_variables
            .get(var_name)
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("la variable {:?} no existe", var_name),
            })?;
        let vvv = vv.variable;
        let vvvv = vv.dimension;
        let base_ptr = self.builder.use_var(vvv);
        let element_size = self
            .builder
            .ins()
            .iconst(self.pointer_type, bytes_per_f64 as i64);
        //k_val * element_size
        let index_val = self.builder.ins().iconst(
            types::I64,
            // position = columns(x-1) + y
            ((vvvv.columns.unwrap_or(1) * (row - 1)) + col) as i64,
        );
        let byte_offset = self.builder.ins().imul(index_val, element_size);
        let element_addr = self.builder.ins().iadd(base_ptr, byte_offset);
        self.builder
            .ins()
            .store(MemFlagsData::new(), *set_val, element_addr, 0);
        Ok(())
    }

    pub fn initialize_seed(&mut self) {
        let seed = f64::default();
        let seed_val = self.builder.ins().f64const(seed);
        let seed_var = self.builder.declare_var(types::F64);
        self.builder.def_var(seed_var, seed_val);
        self.global_variables.insert(
            utils::SEED.to_string(),
            GlobalVariable {
                variable: seed_var,
                dimension: GlobalVariableDimension {
                    rows: u8::default(),
                    columns: None,
                },
            },
        );
    }

    pub fn initialize_global_data_index(&mut self) {
        let global_data_index = self.global_data_index as i64;
        let global_data_index_val = self.builder.ins().iconst(types::I64, global_data_index);
        let global_data_index_var = self.builder.declare_var(types::I64);
        self.builder
            .def_var(global_data_index_var, global_data_index_val);
        self.global_variables.insert(
            utils::GLOBAL_DATA_INDEX.to_string(),
            GlobalVariable {
                variable: global_data_index_var,
                dimension: GlobalVariableDimension {
                    rows: u8::default(),
                    columns: None,
                },
            },
        );
    }

    fn push_string_stack_data(&mut self, string_data: &String) -> Result<Value> {
        let str_data = string_data.as_str();
        let mut data_description = DataDescription::new();
        let mut bytes = str_data.as_bytes().to_vec();
        bytes.push(0);
        data_description.define(bytes.into_boxed_slice());
        let data_id = self.module.declare_anonymous_data(true, false)?;
        self.module.define_data(data_id, &data_description)?;
        let data_global_value = self
            .module
            .declare_data_in_func(data_id, &mut self.builder.func);
        let data_global_pointer = self
            .builder
            .ins()
            .symbol_value(self.pointer_type, data_global_value);
        Ok(data_global_pointer)
    }

    fn print_external(
        &mut self,
        stack_value: &Value,
        print_column: &Value,
        is_tab_call_value: &Value,
    ) -> Result<Value> {
        let local_print_external_fn =
            self.external_functions
                .get("print_external")
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("external funcion not exists {:?}", "print_external"),
                })?;
        let call_print_external_inst = self.builder.ins().call(
            *local_print_external_fn,
            &[*stack_value, *print_column, *is_tab_call_value],
        );
        let ret_call_print_statement_inst =
            self.builder.inst_results(call_print_external_inst)[usize::default()];
        Ok(ret_call_print_statement_inst)
    }

    fn f64_to_pointer_value_external(&mut self, ne: &Value) -> Result<Value> {
        let tamaño_datos = 32;
        let stack_slot_data = StackSlotData::new(StackSlotKind::ExplicitSlot, tamaño_datos, 4);
        let mi_stack_slot = self.builder.create_sized_stack_slot(stack_slot_data);
        let puntero_buffer = self
            .builder
            .ins()
            .stack_addr(self.pointer_type, mi_stack_slot, 0);
        let local_callee = self
            .external_functions
            .get("f64_to_pointer_value_external")
            .ok_or(MinBas78Error::RuntimeError {
                message: format!(
                    "external funcion not exists {:?}",
                    "f64_to_pointer_value_external"
                ),
            })?;
        let llamada_inst = self
            .builder
            .ins()
            .call(*local_callee, &[*ne, puntero_buffer]);
        let _bytes_escritos = self.builder.inst_results(llamada_inst)[usize::default()];
        Ok(puntero_buffer)
    }

    fn pointer_value_to_f64_external(&mut self, ne: &Value) -> Result<Value> {
        let tamaño_datos = 32;
        let stack_slot_data = StackSlotData::new(StackSlotKind::ExplicitSlot, tamaño_datos, 4);
        let mi_stack_slot = self.builder.create_sized_stack_slot(stack_slot_data);
        let puntero_buffer = self
            .builder
            .ins()
            .stack_addr(self.pointer_type, mi_stack_slot, 0);
        let local_callee = self
            .external_functions
            .get("pointer_value_to_f64_external")
            .ok_or(MinBas78Error::RuntimeError {
                message: format!(
                    "external funcion not exists {:?}",
                    "pointer_value_to_f64_external"
                ),
            })?;
        let llamada_inst = self
            .builder
            .ins()
            .call(*local_callee, &[*ne, puntero_buffer]);
        let _bytes_escritos = self.builder.inst_results(llamada_inst)[usize::default()];
        let f64_modificado =
            self.builder
                .ins()
                .stack_load(self.pointer_type, types::F64, mi_stack_slot, 0);
        Ok(f64_modificado)
    }

    pub fn generate_intermediate_representation_file(
        &mut self,
        program_cranelift_intermediate_format_file: &PathBuf,
    ) -> Result<()> {
        let clif_ir = self.builder.func.display().to_string();
        let mut clif_file = File::create(&program_cranelift_intermediate_format_file)?;
        clif_file.write_all(clif_ir.as_bytes())?;
        Ok(())
    }

    pub fn generate_dot_file(&mut self, program_dot_file: &PathBuf) -> Result<()> {
        let cfg_compiler = CFGPrinter::new(self.builder.func);
        let mut file = File::create(&program_dot_file)?;
        write!(file, "{}", cfg_compiler)?;
        Ok(())
    }

    pub fn generate_object_library_file(
        program_obj_file: &PathBuf,
        obj_bytes: &Vec<u8>,
    ) -> Result<()> {
        fs::write(&program_obj_file, &obj_bytes)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn generate_executable_file(
        program_object_file: &PathBuf,
        program_library_file: &PathBuf,
        program_executable_file: &PathBuf,
    ) -> Result<()> {
        let tool = cc::windows_registry::find_tool(utils::TARGET, utils::TOOL).ok_or({
            MinBas78Error::CompilerError {
                message: format!("not found {}", utils::TOOL),
            }
        })?;
        let mut command = Command::new(tool.path());
        command.envs(tool.env().iter().cloned());
        command
            .arg("/nologo")
            .arg(format!("{}", &program_object_file.display()))
            .arg(program_library_file)
            .arg("msvcrt.lib")
            .arg("ws2_32.lib")
            .arg("ntdll.lib")
            .arg("userenv.lib")
            .arg("advapi32.lib")
            .arg("ole32.lib")
            .arg("oleaut32.lib")
            .arg(format!("/OUT:{}", &program_executable_file.display()))
            .arg("/subsystem:console")
            .arg("/release")
            .arg("/incremental:no")
            .arg("/opt:ref")
            .arg("/opt:icf")
            .arg("/ltcg");
        let output = command.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error = format!("{}\n{}", stdout, stderr);
            return Err(anyhow!(MinBas78Error::SystemError {
                filenames: vec![
                    program_object_file.to_string_lossy().to_string(),
                    program_executable_file.to_string_lossy().to_string()
                ],
                message: Some(error)
            }));
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn generate_executable_file(
        program_object_file: &PathBuf,
        program_library_file: &PathBuf,
        program_executable_file: &PathBuf,
    ) -> Result<()> {
        let mut command = Command::new(utils::TOOL);
        command
                .arg(format!("{}", &program_object_file.display()))
                .arg(program_library_file)
                //.arg("target/debug/libminbas78.a")
                .arg(format!("-o{}", &program_executable_file.display()))
                //.arg(format!("-oTEST"))
                ;
        let output = command.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let error = format!("{}\n{}", stdout, stderr);
            return Err(anyhow!(MinBas78Error::SystemError {
                filenames: vec![
                    program_object_file.to_string_lossy().to_string(),
                    program_executable_file.to_string_lossy().to_string()
                ],
                message: Some(error)
            }));
        }
        Ok(())
    }

    /*****************************************************************************
     *  5. PROGRAMS                                                              *
     *****************************************************************************/
    pub fn visit_program(&mut self) {}

    pub fn visit_program_de_walker_line(&mut self) -> Result<()> {
        // 1. Calculate how many bytes you need (each f64 is 8 bytes)
        let global_data_len = self.global_data.len();
        let global_data_total_bytes = (global_data_len * 8) as u32;
        // 2. Allocate an explicit stack slot matching your dynamic size
        let sized_stack_slot = self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            global_data_total_bytes,
            // 2. Define the shift exponent for alignment: 2^3 = 8-byte alignment (optimal for f64)
            3,
        ));
        self.global_data
            .iter()
            .enumerate()
            .try_for_each(|(i, global_data_string)| {
                let global_data_f64 = global_data_string.parse::<f64>()?;
                let global_data_f64_value = self.builder.ins().f64const(global_data_f64);
                // Calculate byte offset: index * 8 bytes
                let byte_offset = (i * 8) as i32;
                self.builder.ins().stack_store(
                    self.pointer_type,
                    global_data_f64_value,
                    sized_stack_slot,
                    byte_offset,
                );
                anyhow::Ok(())
            })?;
        let base_array_pointer =
            self.builder
                .ins()
                .stack_addr(self.pointer_type, sized_stack_slot, 0);
        let variable = self.builder.declare_var(self.pointer_type);
        self.builder.def_var(variable, base_array_pointer);
        self.global_variables.insert(
            utils::GLOBAL_DATA.to_string(),
            GlobalVariable {
                variable,
                dimension: GlobalVariableDimension {
                    rows: u8::default(),
                    columns: None,
                },
            },
        );
        let global_data_len_value = self
            .builder
            .ins()
            .iconst(types::I64, global_data_len as i64);
        let global_data_len_variable = self.builder.declare_var(types::I64);
        self.builder
            .def_var(global_data_len_variable, global_data_len_value);
        self.global_variables.insert(
            utils::GLOBAL_DATA_LEN.to_string(),
            GlobalVariable {
                variable: global_data_len_variable,
                dimension: GlobalVariableDimension {
                    rows: u8::default(),
                    columns: None,
                },
            },
        );
        Ok(())
    }

    pub fn visit_block(&mut self) {}

    pub fn visit_make_block_of_line(&mut self, stack: &String) {
        let block = self.builder.create_block();
        self.blocks.insert(stack.clone(), block);
    }

    pub fn visit_pre_line(&mut self, stack: &String) -> Result<()> {
        let block = self
            .blocks
            .get(&stack.clone())
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("la línea {:?} no existe", stack),
            })?
            .clone();
        self.builder.ins().jump(block, &[]);
        self.builder.switch_to_block(block);
        if self.pre_gosub == true {
            self.gosub_stack.push(stack.clone());
            self.pre_gosub = false;
        }
        Ok(())
    }

    pub fn visit_line(&mut self, stack: &String) -> Result<()> {
        self.previous_line = stack.clone();
        if self.pre_for_to_step == true {
            self.for_to_step_stack.push(stack.clone());
            self.pre_for_to_step = false;
        }
        Ok(())
    }

    pub fn visit_line_number(&mut self) {}

    pub fn visit_end_line(&mut self, stack: &String) -> Result<()> {
        let block = self
            .blocks
            .get(&stack.clone())
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("la línea {:?} no existe", stack),
            })?
            .clone();
        self.builder.ins().jump(block, &[]);
        self.builder.switch_to_block(block);
        let return_value = self.builder.ins().iconst(types::I32, i64::default());
        self.builder.ins().return_(&[return_value]);
        Ok(())
    }

    pub fn visit_statement(&mut self) {}

    pub fn visit_eoi(&mut self) {}

    /*****************************************************************************
     *  6. CONSTANTS                                                             *
     *****************************************************************************/
    pub fn visit_sign(&mut self, node: &Sign) -> Result<Value> {
        let sign = node.sign.trim().to_string();
        let sign_value = match sign.as_str() {
            "+" => Ok(self.builder.ins().f64const(1_f64)),
            "-" => Ok(self.builder.ins().f64const(-1_f64)),
            _ => Err(MinBas78Error::CompilerError {
                message: format!("invalid sign {}", sign),
            }),
        }?;
        Ok(sign_value)
    }

    pub fn visit_numeric_rep(
        &mut self,
        stack: &mut Vec<(String, Value)>,
        node: &NumericRep,
    ) -> Result<(String, Value)> {
        let mut numeric_rep = self.builder.ins().f64const(1_f64);
        if node.exrad.is_some() {
            let base = self.builder.ins().f64const(utils::BASE_10);
            let (_, exrad) = stack.pop().ok_or(MinBas78Error::RuntimeError {
                message: format!("pop error"),
            })?;
            let powf_external_function_reference = self
                .external_functions
                .get("powf_external")
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("external funcion not exists {:?}", "powf_external"),
                })?;
            let powf_external_instruction = self
                .builder
                .ins()
                .call(*powf_external_function_reference, &[base, exrad]);
            let powf_external_result =
                self.builder.inst_results(powf_external_instruction)[usize::default()];
            numeric_rep = powf_external_result;
        }
        let (significand_string, significand_value) =
            stack.pop().ok_or(MinBas78Error::RuntimeError {
                message: format!("pop error"),
            })?;
        numeric_rep = self.builder.ins().fmul(significand_value, numeric_rep);
        Ok((significand_string, numeric_rep))
    }

    pub fn visit_significand(
        &mut self,
        stack: &mut Vec<(String, Value)>,
        node: &Significand,
    ) -> Result<(String, Value)> {
        let mut integer_string = String::default();
        let mut integer_value = self.builder.ins().f64const(f64::default());
        if node.integer.is_some() {
            (integer_string, integer_value) = stack.pop().ok_or(MinBas78Error::RuntimeError {
                message: format!("pop error"),
            })?;
        }
        let mut fraction = self.builder.ins().f64const(f64::default());
        if node.fraction.is_some() {
            (_, fraction) = stack.pop().ok_or(MinBas78Error::RuntimeError {
                message: format!("pop error"),
            })?;
        }
        let significand = self.builder.ins().fadd(integer_value, fraction);
        Ok((integer_string, significand))
    }

    pub fn visit_integer(&mut self, node: &Integer) -> Result<(String, Value)> {
        let integer = node.integer.trim().to_string();
        let integer_f64 = integer.parse::<f64>()?;
        let integer_value = self.builder.ins().f64const(integer_f64);
        Ok((integer, integer_value))
    }

    pub fn visit_fraction(&mut self, node: &Fraction) -> Result<Value> {
        let fraction = node.fraction.trim().parse::<f64>()?;
        let fraction_value = self.builder.ins().f64const(fraction);
        Ok(fraction_value)
    }

    pub fn visit_exrad(&mut self, stack: &mut Vec<Value>, node: &Exrad) -> Result<Value> {
        let mut exrad = stack.pop().ok_or(MinBas78Error::RuntimeError {
            message: format!("pop error"),
        })?;
        if node.sign.is_some() {
            let sign = stack.pop().ok_or(MinBas78Error::RuntimeError {
                message: format!("pop error"),
            })?;
            exrad = self.builder.ins().fmul(sign, exrad);
        }
        Ok(exrad)
    }

    pub fn visit_string_constant(&mut self, node: &StringConstant) -> String {
        let string_constant = node
            .string_constant
            .trim()
            .trim_matches(utils::QUOTATION_MARK)
            .to_string();
        string_constant
    }

    /*****************************************************************************
     *  7. VARIABLES                                                             *
     *****************************************************************************/
    pub fn visit_variable(&mut self, stack: &(String, Value)) -> (String, Value) {
        stack.clone()
    }

    pub fn visit_numeric_variable(&mut self, stack: &dyn Any) -> Result<(String, Value)> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        let (variable_string, variable_dimension, is_matrix) =
            GlobalVariableDimension::get_variable_dimension(&stack_downcast);
        let numeric_variable =
            if let Some(numeric_variable_some) = self.global_variables.get(&variable_string) {
                numeric_variable_some
            } else {
                self.global_variables
                    .iter()
                    .try_for_each(|(global_variable_string, _)| {
                        let (global_variable, _) = stack_downcast
                            .split_once("(")
                            .unwrap_or((stack_downcast, &String::default()));
                        if global_variable.eq(global_variable_string) {
                            return Err(anyhow!(MinBas78Error::RuntimeError {
                                message: format!(
                                    "no se puede crear {:?} porque ya existe {:?}",
                                    stack_downcast, global_variable_string
                                )
                            }));
                        }
                        /*
                                        gv
                                        A	A1  A(0)    A(1)
                        stack	A		OK  OK  ERR	    ERR
                                A1		OK  OK	ERR     ERR
                                A(0)	ERR ERR OK		OK
                                A(1)	ERR	ERR	OK      OK
                        */
                        Ok(())
                    })?;
                let (new_variable_string, new_variable_dimension, _) =
                    GlobalVariableDimension::get_variable_dimension(&stack_downcast);
                let new_global_variables_dimensions =
                    self.global_variables_dimensions.get(&new_variable_string);
                let mut rows_max = utils::ROWS_MAX;
                let mut columns_max = utils::COLUMNS_MAX;
                if new_global_variables_dimensions.is_some() {
                    let new_global_variables_dimensions_ok = new_global_variables_dimensions
                        .ok_or(MinBas78Error::RuntimeError {
                            message: format!("global variable dimension error"),
                        })?;
                    rows_max = new_global_variables_dimensions_ok.rows;
                    if new_global_variables_dimensions_ok.columns.is_some() {
                        let new_global_variables_dimensions_ok_columns =
                            new_global_variables_dimensions_ok.columns.ok_or(
                                MinBas78Error::RuntimeError {
                                    message: format!("global variable dimension error"),
                                },
                            )?;
                        columns_max = new_global_variables_dimensions_ok_columns;
                    }
                }
                if (new_variable_dimension.rows < self.option_base)
                    || (new_variable_dimension.rows > rows_max)
                {
                    return Err(anyhow!(MinBas78Error::RuntimeError {
                        message: format!(
                            "la primera dimension de {:?} debe estar entre {} y {} inclusive",
                            stack_downcast, self.option_base, rows_max
                        )
                    }));
                }
                if new_variable_dimension.columns.is_some() {
                    let new_variable_dimension_column =
                        new_variable_dimension
                            .columns
                            .ok_or(MinBas78Error::RuntimeError {
                                message: format!("column error"),
                            })?;
                    if (new_variable_dimension_column < self.option_base)
                        || (new_variable_dimension_column > columns_max)
                    {
                        return Err(anyhow!(MinBas78Error::RuntimeError {
                            message: format!(
                                "la segunda dimension de {:?} debe estar entre {} y {} inclusive",
                                stack_downcast, self.option_base, columns_max
                            )
                        }));
                    }
                }
                let numeric_variable = self.builder.declare_var(types::F64);
                let zero_value = self.builder.ins().f64const(f64::default());
                self.builder.def_var(numeric_variable, zero_value);
                self.global_variables.insert(
                    new_variable_string.clone(),
                    GlobalVariable {
                        variable: numeric_variable,
                        dimension: GlobalVariableDimension {
                            rows: new_variable_dimension.rows,
                            columns: new_variable_dimension.columns,
                        },
                    },
                );
                if new_global_variables_dimensions.is_none() {
                    self.global_variables_dimensions.insert(
                        new_variable_string.clone(),
                        GlobalVariableDimension {
                            rows: new_variable_dimension.rows,
                            columns: new_variable_dimension.columns,
                        },
                    );
                }
                &GlobalVariable {
                    variable: numeric_variable,
                    dimension: GlobalVariableDimension {
                        rows: new_variable_dimension.rows,
                        columns: new_variable_dimension.columns,
                    },
                }
            };
        let numeric_variable_value = if is_matrix {
            self.array_get(&variable_string, &variable_dimension)?
        } else {
            self.builder.use_var(numeric_variable.variable)
        };
        Ok((stack_downcast.clone(), numeric_variable_value))
    }

    pub fn visit_simple_numeric_variable(
        &mut self,
        stack: &bool,
        node: &SimpleNumericVariable,
    ) -> String {
        let simple_numeric_variable = format!(
            "{}{}",
            if *stack {
                utils::LOCAL_FOR_PREFIX_VARIABLE
            } else {
                ""
            },
            node.simple_numeric_variable.trim().to_string()
        );
        simple_numeric_variable
    }

    pub fn visit_numeric_array_element(&mut self, stack: &dyn Any) -> Result<String> {
        let stack_downcast = stack
            .downcast_ref::<(String, Vec<(String, Value)>)>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        let (numeric_array_element_string, numeric_array_element_dimensions) = stack_downcast;
        let mut numeric_array_element = format!(
            "{}{}",
            numeric_array_element_string.clone(),
            utils::LEFT_PARENTHESIS
        );
        for (i, (dimension_string, _)) in numeric_array_element_dimensions.iter().enumerate() {
            numeric_array_element = format!(
                "{}{}{}",
                numeric_array_element,
                dimension_string,
                if i < numeric_array_element_dimensions.len() - 1 {
                    utils::COMMA
                } else {
                    utils::EMPTY_STRING
                }
            );
        }
        numeric_array_element = format!("{}{}", numeric_array_element, utils::RIGHT_PARENTHESIS);
        if let Some(variable) = self.global_variables.get(&stack_downcast.0) {
            let (_, variable_dimension, _) =
                GlobalVariableDimension::get_variable_dimension(&numeric_array_element);
            if variable_dimension.gt(&variable.dimension) {
                return Err(anyhow!(MinBas78Error::RuntimeError {
                    message: format!(
                        "--> {:?} ES MAYOR A {:?}",
                        variable_dimension, variable.dimension
                    )
                }));
            }
        } else {
            self.array_dim(
                &stack_downcast.0.clone(),
                &GlobalVariableDimension {
                    rows: utils::ROWS_MAX,
                    columns: None,
                },
            )?;
        }
        Ok(numeric_array_element)
    }

    pub fn visit_numeric_array_name(&mut self, stack: &bool, node: &NumericArrayName) -> String {
        let numeric_array_name = format!(
            "{}{}",
            if *stack {
                utils::LOCAL_FOR_PREFIX_VARIABLE
            } else {
                utils::EMPTY_STRING
            },
            node.numeric_array_name.trim().to_string()
        );
        numeric_array_name
    }

    pub fn visit_string_variable(&mut self, node: &StringVariable) -> Result<(String, Value)> {
        let string_variable_name = node.string_variable.trim().to_string();
        let string_variable =
            if let Some(string_variable_some) = self.global_variables.get(&string_variable_name) {
                string_variable_some
            } else {
                let empty_string = String::default();
                let stack_address_value = self.push_string_stack_data(&empty_string)?;
                let string_pointer_variable = self.builder.declare_var(self.pointer_type);
                self.builder
                    .def_var(string_pointer_variable, stack_address_value);
                self.global_variables.insert(
                    string_variable_name.clone(),
                    GlobalVariable {
                        variable: string_pointer_variable,
                        dimension: GlobalVariableDimension {
                            rows: u8::default(),
                            columns: None,
                        },
                    },
                );
                &GlobalVariable {
                    variable: string_pointer_variable,
                    dimension: GlobalVariableDimension {
                        rows: u8::default(),
                        columns: None,
                    },
                }
            };
        let string_variable_value = self.builder.use_var(string_variable.variable);
        Ok((string_variable_name, string_variable_value))
    }

    /*****************************************************************************
     *  8. EXPRESSIONS                                                           *
     *****************************************************************************/
    pub fn visit_expression(&mut self, stack: &Value, node: &Expression) -> Result<Value> {
        let expression = match node {
            Expression::StringExpression(_) => {
                let string_expression_value = stack;
                string_expression_value
            }
            Expression::NumericExpression(_) => {
                let numeric_expression_value = stack;
                let numeric_expression_pointer_value =
                    self.f64_to_pointer_value_external(&numeric_expression_value)?;
                &numeric_expression_pointer_value.clone()
            }
        };
        Ok(*expression)
    }

    pub fn visit_numeric_expression(
        &mut self,
        stack: &mut Vec<Value>,
        node: &NumericExpression,
    ) -> Result<Value> {
        let mut numeric_expression = self.builder.ins().f64const(f64::default());
        let mut sign = self.builder.ins().f64const(1_f64);
        stack.reverse();
        if node.sign.is_some() {
            sign = stack.pop().ok_or(MinBas78Error::RuntimeError {
                message: format!("pop error"),
            })?;
        }
        let term = stack.pop().ok_or(MinBas78Error::RuntimeError {
            message: format!("pop error"),
        })?;
        numeric_expression = self.builder.ins().fadd(numeric_expression, term);
        numeric_expression = self.builder.ins().fmul(numeric_expression, sign);
        stack.reverse();
        while !stack.is_empty() {
            let sign_term_sign = stack.pop().ok_or(MinBas78Error::RuntimeError {
                message: format!("pop error"),
            })?;
            let sign_term_term = stack.pop().ok_or(MinBas78Error::RuntimeError {
                message: format!("pop error"),
            })?;
            let sign_term_value = self.builder.ins().fmul(sign_term_term, sign_term_sign);
            numeric_expression = self.builder.ins().fadd(numeric_expression, sign_term_value);
        }
        let numeric_expression_minimum_value = self.builder.ins().f64const(-1E+38);
        let numeric_expression_maximum_value = self.builder.ins().f64const(1E+38);
        let rows_then_block = self.builder.create_block();
        let rows_else_block = self.builder.create_block();
        let rows_end_block = self.builder.create_block();
        self.builder.append_block_param(rows_end_block, types::F64);
        let numeric_expression_minimum_comparison = self.builder.ins().fcmp(
            FloatCC::LessThan,
            numeric_expression,
            numeric_expression_minimum_value,
        );
        self.builder.ins().brif(
            numeric_expression_minimum_comparison,
            rows_then_block,
            &[],
            rows_else_block,
            &[],
        );
        self.builder.switch_to_block(rows_then_block);
        self.builder.seal_block(rows_then_block);
        self.builder.ins().jump(
            rows_end_block,
            &[BlockArg::Value(numeric_expression_minimum_value)],
        );
        self.builder.switch_to_block(rows_else_block);
        self.builder.seal_block(rows_else_block);
        self.builder
            .ins()
            .jump(rows_end_block, &[BlockArg::Value(numeric_expression)]);
        self.builder.switch_to_block(rows_end_block);
        self.builder.seal_block(rows_end_block);
        numeric_expression = self.builder.block_params(rows_end_block)[usize::default()];
        let columns_then_block = self.builder.create_block();
        let columns_else_block = self.builder.create_block();
        let columns_end_block = self.builder.create_block();
        self.builder
            .append_block_param(columns_end_block, types::F64);
        let numeric_expression_minimum_comparison = self.builder.ins().fcmp(
            FloatCC::GreaterThan,
            numeric_expression,
            numeric_expression_maximum_value,
        );
        self.builder.ins().brif(
            numeric_expression_minimum_comparison,
            columns_then_block,
            &[],
            columns_else_block,
            &[],
        );
        self.builder.switch_to_block(columns_then_block);
        self.builder.seal_block(columns_then_block);
        self.builder.ins().jump(
            columns_end_block,
            &[BlockArg::Value(numeric_expression_maximum_value)],
        );
        self.builder.switch_to_block(columns_else_block);
        self.builder.seal_block(columns_else_block);
        self.builder
            .ins()
            .jump(columns_end_block, &[BlockArg::Value(numeric_expression)]);
        self.builder.switch_to_block(columns_end_block);
        self.builder.seal_block(columns_end_block);
        numeric_expression = self.builder.block_params(columns_end_block)[usize::default()];
        Ok(numeric_expression)
    }

    pub fn visit_term(&mut self, stack: &mut dyn Any) -> Result<(String, Value)> {
        let stack_downcast =
            stack
                .downcast_ref::<Vec<(String, Value)>>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        let stack_len = stack_downcast.len();
        let stack_len_bytes = (stack_len * 8) as u32;
        let sized_stack_slot = self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            stack_len_bytes,
            3, //2^3=8 bits
        ));
        let mut term_string = String::default();
        for (i, (clif_string, clif_value)) in stack_downcast.iter().enumerate() {
            if !clif_string.is_empty() {
                term_string = clif_string.clone();
            }
            let offset_bytes = (i * 8) as i32;
            self.builder.ins().stack_store(
                self.pointer_type,
                *clif_value,
                sized_stack_slot,
                offset_bytes,
            );
        }
        let pointer_value = self
            .builder
            .ins()
            .stack_addr(self.pointer_type, sized_stack_slot, 0);
        let stack_len_value = self
            .builder
            .ins()
            .iconst(self.pointer_type, stack_len as i64);
        let get_term_external_function_reference = self
            .external_functions
            .get("get_term_external")
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("external funcion not exists {:?}", "get_term_external"),
            })?;
        let get_term_external_instruction = self.builder.ins().call(
            *get_term_external_function_reference,
            &[pointer_value, stack_len_value],
        );
        let term = self.builder.inst_results(get_term_external_instruction)[usize::default()];
        Ok((term_string, term))
    }

    pub fn visit_multiplier_factor(&mut self, stack: &dyn Any) -> Result<Vec<(String, Value)>> {
        let stack_downcast =
            stack
                .downcast_ref::<Vec<(String, Value)>>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        Ok(stack_downcast.to_vec())
    }

    pub fn visit_factor(
        &mut self,
        stack: &mut Vec<(String, Value)>,
        node: &Factor,
    ) -> Result<(String, Value)> {
        stack.reverse();
        let primary = stack.pop().ok_or(MinBas78Error::RuntimeError {
            message: format!("pop error"),
        })?;
        let (primary_string, primary_value) = primary.clone();
        let mut factor = primary;
        if node.circumflex_accent_primary.is_some() {
            let (_, circumflex_accent_primary_value) =
                stack.pop().ok_or(MinBas78Error::RuntimeError {
                    message: format!("pop error"),
                })?;
            let powf_external_function_reference = self
                .external_functions
                .get("powf_external")
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("external funcion not exists {:?}", "powf_external"),
                })?;
            let powf_external_function_instruction = self.builder.ins().call(
                *powf_external_function_reference,
                &[primary_value, circumflex_accent_primary_value],
            );
            factor = (
                primary_string,
                self.builder
                    .inst_results(powf_external_function_instruction)[usize::default()],
            );
        }
        Ok(factor)
    }

    pub fn visit_circumflex_accent_primary(&mut self, stack: &dyn Any) -> Result<Value> {
        let stack_downcast = stack
            .downcast_ref::<Value>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(*stack_downcast)
    }

    pub fn visit_multiplier(&mut self, node: &Multiplier) -> Value {
        let multiplier = node.multiplier.trim().to_string();
        let mut multiplier_value = self.builder.ins().f64const(f64::default());
        if multiplier.eq(&utils::SOLIDUS) {
            multiplier_value = self.builder.ins().f64const(1_f64);
        }
        multiplier_value
    }

    pub fn visit_primary(&mut self, stack: &mut dyn Any) -> Result<(String, Value)> {
        let stack_downcast =
            stack
                .downcast_mut::<Vec<(String, Value)>>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        let primary = stack_downcast.pop().ok_or(MinBas78Error::RuntimeError {
            message: format!("pop error"),
        })?;
        Ok(primary)
    }

    pub fn visit_numeric_expression_other(&mut self, stack: &dyn Any) -> Result<Value> {
        let stack_downcast = stack
            .downcast_ref::<Value>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(*stack_downcast)
    }

    pub fn visit_numeric_function_ref(
        &mut self,
        numeric_function_name: &String,
        numeric_expression: &Option<(String, Value)>,
        node: &NumericFunctionRef,
    ) -> Result<Value> {
        let numeric_function_reference_result: Option<Value>;
        let get_numeric_function_ref_funref = self
            .external_functions
            .get("get_numeric_function_ref_external")
            .ok_or(MinBas78Error::RuntimeError {
                message: format!(
                    "external funcion not exists {:?}",
                    "get_numeric_function_ref_external"
                ),
            })?
            .clone();
        let numeric_function_name_value = match numeric_function_name.as_str() {
            "ABS" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::ABS.as_value()),
            ),
            "ATN" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::ATN.as_value()),
            ),
            "COS" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::COS.as_value()),
            ),
            "EXP" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::EXP.as_value()),
            ),
            "INT" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::INT.as_value()),
            ),
            "LOG" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::LOG.as_value()),
            ),
            "RND" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::RND.as_value()),
            ),
            "SGN" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::SGN.as_value()),
            ),
            "SIN" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::SIN.as_value()),
            ),
            "SQR" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::SQR.as_value()),
            ),
            "TAN" => Some(
                self.builder
                    .ins()
                    .f64const(NumericFunctionRefExternal::TAN.as_value()),
            ),
            _ => None,
        };
        if numeric_function_name_value.is_none() {
            let mut param = None::<Value>;
            if numeric_expression.is_some() {
                let numeric_expression_tuple =
                    numeric_expression
                        .clone()
                        .ok_or(MinBas78Error::RuntimeError {
                            message: format!("numeric_expression error"),
                        })?;
                let (numeric_expression_string, numeric_expression_value) =
                    numeric_expression_tuple;
                numeric_expression_string.parse::<f64>()?;
                param = Some(numeric_expression_value);
            }
            let numeric_function_name_string = format!("{}", &numeric_function_name);
            let user_defined_function_numeric_expression = self
                .user_defined_functions
                .get(&numeric_function_name_string)
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!(
                        "la function externa {:?} no existe",
                        numeric_function_name_string
                    ),
                })?
                .clone();
            let mut user_defined_function_walker =
                user_defined_function_walker::BasicAbstractSyntaxTreeUserDefinedFunctionWalker;
            let mut local_variable = None::<GlobalVariable>;
            let mut local_value_before = None::<Value>;
            if user_defined_function_numeric_expression.0.is_some() {
                let (user_defined_function_numeric_expression_string, _) =
                    user_defined_function_numeric_expression.clone();
                let user_defined_function_numeric_expression_string_ok =
                    user_defined_function_numeric_expression_string.ok_or(
                        MinBas78Error::RuntimeError {
                            message: format!(
                                "user_defined_function_numeric_expression_string error"
                            ),
                        },
                    )?;
                local_variable = self
                    .global_variables
                    .get(&user_defined_function_numeric_expression_string_ok)
                    .copied();
                if local_variable.is_some() {
                    let global_variable = local_variable.ok_or(MinBas78Error::RuntimeError {
                        message: format!("global variable error"),
                    })?;
                    local_value_before = Some(self.builder.use_var(global_variable.variable));
                    let param_param = param.ok_or(MinBas78Error::RuntimeError {
                        message: format!("param value error"),
                    })?;
                    self.builder.def_var(global_variable.variable, param_param);
                }
            }
            let (_, user_defined_function_abstract_syntax_tree_value) =
                user_defined_function_walker.walk_numeric_expression(
                    self,
                    &false,
                    &user_defined_function_numeric_expression.1,
                )?;
            if user_defined_function_numeric_expression.0.is_some() && local_variable.is_some() {
                let global_variable = local_variable.ok_or(MinBas78Error::RuntimeError {
                    message: format!("global variable error"),
                })?;
                let global_variable_value =
                    local_value_before.ok_or(MinBas78Error::RuntimeError {
                        message: format!("x_local_value_before value error"),
                    })?;
                self.builder
                    .def_var(global_variable.variable, global_variable_value);
            }
            Ok(user_defined_function_abstract_syntax_tree_value)
        } else {
            let seed_variable =
                self.global_variables
                    .get(utils::SEED)
                    .ok_or(MinBas78Error::RuntimeError {
                        message: format!("la variable {:?} no existe", utils::SEED),
                    })?;
            let default_value = self.builder.ins().f64const(f64::default());
            let (_, mut numeric_expression_tuple_value) = (String::default(), default_value);
            if node.numeric_expression.is_some() {
                if numeric_function_name == NumericFunctionRefExternal::RND.as_str() {
                    let seed_value = self.builder.use_var(seed_variable.variable);
                    (_, numeric_expression_tuple_value) = (String::default(), seed_value);
                } else {
                    let nec = numeric_expression
                        .clone()
                        .ok_or(MinBas78Error::RuntimeError {
                            message: format!("numeric_expression value error"),
                        })?;
                    (_, numeric_expression_tuple_value) = nec;
                }
            } else {
                if numeric_function_name == NumericFunctionRefExternal::RND.as_str() {
                    let seed_value = self.builder.use_var(seed_variable.variable);
                    (_, numeric_expression_tuple_value) = (String::default(), seed_value);
                }
            }
            let numeric_function_name_value_ok =
                numeric_function_name_value.ok_or(MinBas78Error::RuntimeError {
                    message: format!("numeric_function_name_value error"),
                })?;
            let numeric_function_reference_instruction = self.builder.ins().call(
                get_numeric_function_ref_funref,
                &[
                    numeric_function_name_value_ok,
                    numeric_expression_tuple_value,
                ],
            );
            numeric_function_reference_result = Some(
                self.builder
                    .inst_results(numeric_function_reference_instruction)[usize::default()],
            );
            let numeric_function_reference_result_ok =
                numeric_function_reference_result.ok_or(MinBas78Error::RuntimeError {
                    message: format!("numeric_function_ref_result value error"),
                })?;
            if numeric_function_name == NumericFunctionRefExternal::RND.as_str() {
                self.builder
                    .def_var(seed_variable.variable, numeric_function_reference_result_ok);
            }
            Ok(numeric_function_reference_result_ok)
        }
    }

    pub fn visit_numeric_function_name(&mut self, stack: &dyn Any) -> Result<String> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(stack_downcast.clone())
    }

    pub fn visit_string_expression(
        &mut self,
        stack: &String,
        node: &StringExpression,
    ) -> Result<(String, Value)> {
        let string_expression = match node {
            StringExpression::StringConstant(_) => {
                let string_pointer_value = self.push_string_stack_data(&stack)?;
                (stack.clone(), string_pointer_value)
            }
            StringExpression::StringVariable(_) => {
                let variable =
                    self.global_variables
                        .get(stack)
                        .ok_or(MinBas78Error::RuntimeError {
                            message: format!("la variable {:?} no existe", stack),
                        })?;
                let value = self.builder.use_var(variable.variable);
                (String::default(), value)
            }
        };
        Ok(string_expression)
    }

    /*****************************************************************************
     *  9. IMPLEMENTATION SUPPLIED FUNCTIONS                                     *
     *****************************************************************************/
    pub fn visit_numeric_supplied_function(&mut self, stack: &dyn Any) -> Result<String> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(stack_downcast.clone())
    }

    /*****************************************************************************
     *  10. USER DEFINED FUNCTIONS                                               *
     *****************************************************************************/
    pub fn visit_def_statement(
        &mut self,
        numeric_defined_function: String,
        parameter_list: Option<String>,
        node: &DefStatement,
    ) -> Result<()> {
        let user_defined_function_name = format!("{}", &numeric_defined_function);
        let user_defined_function_name_numeric_expresion = format!("{:?}", node.numeric_expression);
        if user_defined_function_name_numeric_expresion.contains(&user_defined_function_name) {
            return Err(anyhow!(MinBas78Error::RuntimeError {
                message: format!(
                    "la función {:?} contiene una llamada recursiva --> {:?}",
                    &user_defined_function_name, user_defined_function_name_numeric_expresion
                )
            }));
        }
        if self
            .user_defined_functions
            .get(&user_defined_function_name)
            .is_some()
        {
            return Err(anyhow!(MinBas78Error::RuntimeError {
                message: format!(
                    "la función {:?} ya estaba definida",
                    &user_defined_function_name
                )
            }));
        }
        self.user_defined_functions.insert(
            user_defined_function_name.clone(),
            (parameter_list.clone(), node.numeric_expression.clone()),
        );
        Ok(())
    }

    pub fn visit_numeric_defined_function(&mut self, node: &NumericDefinedFunction) -> String {
        let numeric_defined_function = node.numeric_defined_function.trim().to_string();
        numeric_defined_function
    }

    pub fn visit_parameter_list(&mut self, stack: &dyn Any) -> Result<String> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(stack_downcast.clone())
    }

    /*****************************************************************************
     *  11. LET STATEMENT                                                        *
     *****************************************************************************/
    pub fn visit_let_statement(&mut self) {}

    pub fn visit_numeric_let_statement(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast =
            stack
                .downcast_ref::<(String, Value)>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        let (stack_string, stack_value) = stack_downcast;
        let (variable_dimension_string, variable_dimension, is_matrix) =
            GlobalVariableDimension::get_variable_dimension(stack_string);
        let variable = self
            .global_variables
            .get(&variable_dimension_string)
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("la variable {:?} no existe", variable_dimension_string),
            })?;
        if is_matrix {
            self.array_set(&variable_dimension_string, &variable_dimension, stack_value)?;
        } else {
            self.builder.def_var(variable.variable, *stack_value);
        }
        Ok(())
    }

    pub fn visit_string_let_statement(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast =
            stack
                .downcast_ref::<(String, String, Value)>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        let (string_variable, stack_string, stack_value) = stack_downcast;
        if stack_string.len().gt(&utils::STRING_MAX) {
            return Err(anyhow!(MinBas78Error::RuntimeError {
                message: format!(
                    "la variable {:?} excede los 18 caracteres: {:?} tiene {} caracteres.",
                    string_variable,
                    stack_string,
                    stack_string.len()
                )
            }))?;
        }
        let variable = self.builder.declare_var(types::I64);
        self.builder.def_var(variable, *stack_value);
        self.global_variables.insert(
            string_variable.clone(),
            GlobalVariable {
                variable: variable,
                dimension: GlobalVariableDimension {
                    rows: u8::default(),
                    columns: None,
                },
            },
        );
        Ok(())
    }

    /*****************************************************************************
     *  12. CONTROL STATEMENT                                                    *
     *****************************************************************************/
    pub fn visit_goto_statement(&mut self, stack: &dyn Any) -> Result<String> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        let target_block = self
            .blocks
            .get(stack_downcast)
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("el bloque {:?} no existe", stack_downcast),
            })?;
        self.builder.ins().jump(*target_block, &[]);
        let goto_block = self.builder.create_block();
        self.builder.switch_to_block(goto_block);
        Ok(stack_downcast.clone())
    }

    pub fn visit_if_then_statement(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast =
            stack
                .downcast_ref::<(Value, String)>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        let (relational_expression, line_number) = stack_downcast;
        let block = self
            .blocks
            .get(line_number)
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("la linea {:?} no existe", line_number),
            })?;
        let intermediate_block = self.builder.create_block();
        self.builder
            .ins()
            .brif(*relational_expression, *block, &[], intermediate_block, &[]);
        self.builder.switch_to_block(intermediate_block);
        Ok(())
    }

    pub fn visit_relational_expression(&mut self, stack: &dyn Any) -> Result<Value> {
        let stack_downcast = stack.downcast_ref::<(Value, String, Value, bool)>().ok_or(
            MinBas78Error::RuntimeError {
                message: format!("stack error"),
            },
        )?;

        let (
            relational_expression_a_value,
            relational_expression_string,
            relational_expression_b_value,
            is_numeric_expression,
        ) = stack_downcast;
        let relational_expression = if *is_numeric_expression == true {
            self.builder.ins().fcmp(
                match relational_expression_string.as_str() {
                    utils::EQUALS_SIGN => FloatCC::Equal,
                    utils::NOT_EQUALS => FloatCC::NotEqual,
                    utils::GREATER_THAN_SIGN => FloatCC::GreaterThan,
                    utils::NOT_LESS => FloatCC::GreaterThanOrEqual,
                    utils::LESS_THAN_SIGN => FloatCC::LessThan,
                    utils::NOT_GREATER => FloatCC::LessThanOrEqual,
                    _ => {
                        return Err(anyhow!(MinBas78Error::RuntimeError {
                            message: format!(
                                "la relación {:?} no existe",
                                relational_expression_string
                            )
                        }));
                    }
                },
                *relational_expression_a_value,
                *relational_expression_b_value,
            )
        } else {
            let strcmp_external_function_reference = self
                .external_functions
                .get("strcmp_external")
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("external funcion not exists {:?}", "strcmp_external"),
                })?;
            let strcmp_external_instruction = self.builder.ins().call(
                *strcmp_external_function_reference,
                &[
                    *relational_expression_a_value,
                    *relational_expression_b_value,
                ],
            );
            let return_strcmp_external =
                self.builder.inst_results(strcmp_external_instruction)[usize::default()];
            return_strcmp_external
        };
        Ok(relational_expression)
    }

    pub fn visit_relation(&mut self) {}

    pub fn visit_equality_relation(&mut self) {}

    pub fn visit_gosub_statement(&mut self, stack: &dyn Any) -> Result<String> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        let block = self
            .blocks
            .get(stack_downcast)
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("el bloque {:?} no existe", stack_downcast),
            })?
            .clone();
        self.builder.ins().jump(block, &[]);
        let next_linear_block = self.builder.create_block();
        self.builder.switch_to_block(next_linear_block);
        self.builder.seal_block(next_linear_block);
        self.pre_gosub = true;
        Ok(stack_downcast.clone())
    }

    pub fn visit_return_statement(&mut self) -> Result<()> {
        let return_line = self.gosub_stack.pop();
        if let Some(return_line_some) = return_line {
            let block = self
                .blocks
                .get(&return_line_some)
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("la variable {:?} no existe", return_line_some),
                })?
                .clone();
            self.builder.ins().jump(block, &[]);
            let intermediate_block = self.builder.create_block();
            self.builder.switch_to_block(intermediate_block);
            self.builder.seal_block(intermediate_block);
        } else {
            return Err(anyhow!(MinBas78Error::RuntimeError {
                message: String::from("RETURN sin GOSUB")
            }));
        }
        Ok(())
    }

    pub fn visit_on_goto_statement(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast =
            stack
                .downcast_ref::<(Value, Vec<String>)>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;

        let (numeric_expression, line_numbers) = stack_downcast;
        let integer_value = self
            .builder
            .ins()
            .fcvt_to_sint(types::I64, *numeric_expression);
        let one_value = self.builder.ins().iconst(types::I64, 1_i64);
        let index_value = self.builder.ins().isub(integer_value, one_value);
        let mut switch = Switch::new();
        line_numbers
            .iter()
            .enumerate()
            .try_for_each(|(i, line_number_string)| {
                let block =
                    self.blocks
                        .get(line_number_string)
                        .ok_or(MinBas78Error::RuntimeError {
                            message: format!("la variable {:?} no existe", line_number_string),
                        })?;
                switch.set_entry(i as u128, *block);
                anyhow::Ok(())
            })?;
        let error_block = self.builder.create_block();
        switch.emit(self.builder, index_value, error_block);
        self.builder.switch_to_block(error_block);
        self.builder.ins().trap(TrapCode::unwrap_user(1_u8));
        let next_line_block = self.builder.create_block();
        self.builder.switch_to_block(next_line_block);
        Ok(())
    }

    pub fn visit_stop_statement(&mut self) {
        let return_value = self.builder.ins().iconst(types::I64, i64::default());
        self.builder.ins().return_(&[return_value]);
        let intermediate_block = self.builder.create_block();
        self.builder.switch_to_block(intermediate_block);
    }

    /*****************************************************************************
     *  13. FOR AND NEXT STATEMENTS                                              *
     *****************************************************************************/
    pub fn visit_for_block(&mut self) -> Result<()> {
        if !self.for_to_step_stack.is_empty() {
            self.for_to_step_stack
                .pop()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("for_to_step_stack error"),
                })?;
        }
        Ok(())
    }

    pub fn visit_for_body(&mut self) {}

    pub fn visit_for_line(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast = stack
            .downcast_ref::<(String, String, Value, Value, Option<Value>)>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        let (line_number, control_variable, initial_value, limit, increment) = stack_downcast;
        self.previous_line = line_number.clone();
        let first_for_value = *limit;
        let second_for_value = if increment.is_some() {
            let increment_value = increment.ok_or(MinBas78Error::RuntimeError {
                message: format!("for_statement error"),
            })?;
            increment_value
        } else {
            self.builder.ins().f64const(1_f64)
        };
        let zero_value = self.builder.ins().f64const(f64::default());
        let one_value = self.builder.ins().f64const(1_f64);
        let negative_one_value = self.builder.ins().f64const(-1_f64);
        let is_positive =
            self.builder
                .ins()
                .fcmp(FloatCC::GreaterThan, second_for_value, zero_value);
        let is_negative = self
            .builder
            .ins()
            .fcmp(FloatCC::LessThan, second_for_value, zero_value);
        let sign_or_zero = self
            .builder
            .ins()
            .select(is_negative, negative_one_value, zero_value);
        let sign_second_for_value = self
            .builder
            .ins()
            .select(is_positive, one_value, sign_or_zero);
        let variable_value = initial_value;
        let variable = self.builder.declare_var(types::F64);
        self.builder.def_var(variable, *variable_value);
        let local_variable_name = format!(
            "{}{}",
            utils::LOCAL_FOR_PREFIX_VARIABLE,
            control_variable.clone()
        );
        self.global_variables.insert(
            local_variable_name.clone(),
            GlobalVariable {
                variable,
                dimension: GlobalVariableDimension {
                    rows: u8::default(),
                    columns: None,
                },
            },
        );
        let first_for_variable = self.builder.declare_var(types::F64);
        self.builder.def_var(first_for_variable, first_for_value);
        let sign_second_for_variable = self.builder.declare_var(types::F64);
        self.builder
            .def_var(sign_second_for_variable, sign_second_for_value);
        let second_for_variable = self.builder.declare_var(types::F64);
        self.builder.def_var(second_for_variable, second_for_value);
        self.global_variables.insert(
            format!("{}{}", local_variable_name, ".OWN2"),
            GlobalVariable {
                variable: second_for_variable,
                dimension: GlobalVariableDimension {
                    rows: u8::default(),
                    columns: None,
                },
            },
        );
        let loop_header = self.builder.create_block();
        let loop_body = self.builder.create_block();
        let loop_exit = self.builder.create_block();
        self.builder.ins().jump(loop_header, &[]);
        self.builder.switch_to_block(loop_header);
        self.for_to_step_block_stack
            .push((control_variable.clone(), loop_header, loop_exit));
        let current_value = self.builder.use_var(variable);
        let current_first_for_value = self.builder.use_var(first_for_variable);
        let current_sign_second_for_value = self.builder.use_var(sign_second_for_variable);
        let mut if_value = self
            .builder
            .ins()
            .fsub(current_value, current_first_for_value);
        if_value = self
            .builder
            .ins()
            .fmul(if_value, current_sign_second_for_value);
        let loop_overflow = self
            .builder
            .ins()
            .fcmp(FloatCC::GreaterThan, if_value, zero_value);
        self.builder
            .ins()
            .brif(loop_overflow, loop_exit, &[], loop_body, &[]);
        self.builder.switch_to_block(loop_body);
        Ok(())
    }

    pub fn visit_next_line(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        let local_variable_name = format!("{}{}", utils::LOCAL_FOR_PREFIX_VARIABLE, stack_downcast);
        let variable = self
            .global_variables
            .get(&local_variable_name)
            .expect("Variable FOR no declarada")
            .clone();
        let mut variable_value = self.builder.use_var(variable.variable);
        let second_for_variable = self
            .global_variables
            .get(&format!("{}{}", local_variable_name, ".OWN2"))
            .ok_or(MinBas78Error::RuntimeError {
                message: format!(
                    "la variable {:?} no existe",
                    format!("{}{}", local_variable_name, ".OWN2")
                ),
            })?;
        let second_variable_value = self.builder.use_var(second_for_variable.variable);
        variable_value = self
            .builder
            .ins()
            .fadd(variable_value, second_variable_value);
        self.builder.def_var(variable.variable, variable_value);
        let index = self
            .for_to_step_block_stack
            .iter()
            .rposition(|(var_name, _, _)| var_name == stack_downcast)
            .ok_or(anyhow!(MinBas78Error::RuntimeError {
                message: format!(
                    "NEXT sin un FOR correspondiente para la variable {:?}",
                    stack_downcast
                )
            }))?;
        let (_, loop_header, loop_exit) = self.for_to_step_block_stack.remove(index);
        self.builder.ins().jump(loop_header, &[]);
        self.builder.switch_to_block(loop_exit);
        let loop_exit_variable =
            self.global_variables
                .get(&local_variable_name)
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("la variable {:?} no existe", local_variable_name),
                })?;
        let mut loop_exit_value = self.builder.use_var(loop_exit_variable.variable);
        let second_variable_exit_value = self.builder.use_var(second_for_variable.variable);
        loop_exit_value = self
            .builder
            .ins()
            .fsub(loop_exit_value, second_variable_exit_value);
        self.builder
            .def_var(loop_exit_variable.variable, loop_exit_value);
        self.global_variables
            .insert(local_variable_name, *loop_exit_variable);
        Ok(())
    }

    pub fn visit_for_statement(
        &mut self,
        stack: &dyn Any,
    ) -> Result<(String, Value, Value, Option<Value>)> {
        /*
        control_variable: &String,
        initial_value: &Value,
        limit: &Value,
        increment: &Option<Value>,
         */
        let stack_downcast = stack
            .downcast_ref::<(String, Value, Value, Option<Value>)>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;

        let (control_variable, initial_value, limit, increment) = stack_downcast;
        Ok((control_variable.clone(), *initial_value, *limit, *increment))
    }

    pub fn visit_control_variable(&mut self, stack: &dyn Any) -> Result<String> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(stack_downcast.clone())
    }

    pub fn visit_initial_value(&mut self, stack: &dyn Any) -> Result<Value> {
        let stack_downcast = stack
            .downcast_ref::<Value>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(*stack_downcast)
    }

    pub fn visit_limit(&mut self, stack: &dyn Any) -> Result<Value> {
        let stack_downcast = stack
            .downcast_ref::<Value>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(*stack_downcast)
    }

    pub fn visit_increment(&mut self, stack: &dyn Any) -> Result<Value> {
        let stack_downcast = stack
            .downcast_ref::<Value>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(*stack_downcast)
    }

    pub fn visit_next_statement(&mut self, stack: &dyn Any) -> Result<String> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(stack_downcast.clone())
    }

    /*****************************************************************************
     *  14. PRINT STATEMENT                                                      *
     *****************************************************************************/
    pub fn visit_print_statement(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast = stack
            .downcast_ref::<Vec<(Option<(Value, bool)>, Option<String>)>>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        let zero_i64 = self.builder.ins().iconst(types::I64, i64::default());
        let zero_i8 = self.builder.ins().iconst(types::I8, i64::default());
        let one_i8 = self.builder.ins().iconst(types::I8, 1_i64);
        let new_line_value = self.push_string_stack_data(&utils::NEW_LINE.to_string())?;
        if self.print_column.is_none() {
            let print_column_var = self.builder.declare_var(types::I64);
            self.builder.def_var(print_column_var, zero_i64);
            self.print_column = Some(print_column_var);
        }
        let print_column_variable = self.print_column.ok_or(MinBas78Error::RuntimeError {
            message: format!("print_column error"),
        })?;
        if stack_downcast.is_empty() {
            let mut print_column_value = self.builder.use_var(print_column_variable);
            print_column_value =
                self.print_external(&new_line_value, &print_column_value, &zero_i8)?;
            self.builder
                .def_var(print_column_variable, print_column_value);
        } else {
            stack_downcast
                .iter()
                .try_for_each(|(puntero_buffer, separator)| {
                    if puntero_buffer.is_some() {
                        let mut print_column_variable_value =
                            self.builder.use_var(print_column_variable);
                        let (stack_pointer_buffer, is_tab_call) =
                            &puntero_buffer.ok_or(MinBas78Error::RuntimeError {
                                message: format!("puntero_buffer error"),
                            })?;
                        let is_tab_call_value = if *is_tab_call { one_i8 } else { zero_i8 };
                        print_column_variable_value = self.print_external(
                            &stack_pointer_buffer,
                            &print_column_variable_value,
                            &is_tab_call_value,
                        )?;
                        self.builder
                            .def_var(print_column_variable, print_column_variable_value);
                    }
                    if separator.is_some() {
                        let separator = separator.clone().ok_or(MinBas78Error::RuntimeError {
                            message: format!("separator error"),
                        })?;
                        let mut print_column_value = self.builder.use_var(print_column_variable);
                        let separator_value = self.push_string_stack_data(&separator)?;
                        print_column_value =
                            self.print_external(&separator_value, &print_column_value, &zero_i8)?;
                        self.builder
                            .def_var(print_column_variable, print_column_value);
                    } else {
                        let print_column_value = self.builder.use_var(print_column_variable);
                        self.print_external(&new_line_value, &print_column_value, &zero_i8)?;
                        self.builder.def_var(print_column_variable, zero_i64);
                    }
                    anyhow::Ok(())
                })?;
        }
        Ok(())
    }

    pub fn visit_print_list(
        &mut self,
        stack: &dyn Any,
    ) -> Result<Vec<(Option<(Value, bool)>, Option<String>)>> {
        let stack_downcast = stack
            .downcast_ref::<Vec<(Option<(Value, bool)>, Option<String>)>>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(stack_downcast.clone())
    }

    pub fn visit_print_item(&mut self, stack: &dyn Any) -> Result<Option<(Value, bool)>> {
        let stack_downcast =
            stack
                .downcast_ref::<Option<(Value, bool)>>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        Ok(stack_downcast.clone())
    }

    pub fn visit_print_separator(&mut self, node: &PrintSeparator) -> String {
        let print_separator = node.print_separator.trim().to_string();
        print_separator
    }

    pub fn visit_tab_call(&mut self, stack: &dyn Any) -> Result<Value> {
        let stack_downcast = stack
            .downcast_ref::<Value>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        let one_f64 = self.builder.ins().f64const(1_f64);
        let is_less = self
            .builder
            .ins()
            .fcmp(FloatCC::LessThan, *stack_downcast, one_f64);
        self.builder
            .ins()
            .trapnz(is_less, TrapCode::unwrap_user(1_u8));
        let stack_truncate_value = self.builder.ins().trunc(*stack_downcast);
        let tab_call = self.f64_to_pointer_value_external(&stack_truncate_value)?;
        Ok(tab_call)
    }

    /*****************************************************************************
     *  15. INPUT STATEMENT                                                      *
     *****************************************************************************/
    pub fn visit_input_statement(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast =
            stack
                .downcast_ref::<Vec<(String, Value)>>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        let stack_variable_names = stack_downcast
            .iter()
            .map(|(variable_name, _)| variable_name.clone())
            .collect::<Vec<String>>();
        let stack_length = stack_downcast.len();
        let variable_length_bytes = mem::size_of::<*mut std::os::raw::c_char>();
        let total_bytes = stack_length * variable_length_bytes;
        let alignment_bytes = align_of::<*mut std::os::raw::c_char>();
        let align_shift = alignment_bytes.trailing_zeros() as u8;
        let sized_stack_slot = self.builder.create_sized_stack_slot(StackSlotData::new(
            StackSlotKind::ExplicitSlot,
            total_bytes as u32,
            align_shift,
        ));
        let array_pointer = self
            .builder
            .ins()
            .stack_addr(self.pointer_type, sized_stack_slot, 0);
        let count_value = self.builder.ins().iconst(types::I64, stack_length as i64);
        let input_external_function_reference = self
            .external_functions
            .get(&String::from("input_external"))
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("external funcion not exists {:?}", "input_external"),
            })?;
        self.builder.ins().call(
            *input_external_function_reference,
            &[array_pointer, count_value],
        );
        for (i, variable_name) in stack_variable_names.iter().enumerate() {
            let offset = i * variable_length_bytes;
            let input_variable_value = self.builder.ins().stack_load(
                self.pointer_type,
                self.pointer_type,
                sized_stack_slot,
                offset as i32,
            );
            let mut pointer_type = self.pointer_type;
            let mut variable_value = input_variable_value;
            if !variable_name.ends_with(utils::DOLLAR_SIGN) {
                pointer_type = types::F64;
                variable_value = self.pointer_value_to_f64_external(&input_variable_value)?;
            }
            let (variable_name_string, variable_dimensions, _) =
                GlobalVariableDimension::get_variable_dimension(variable_name);
            let input_variable = match self.global_variables.get(variable_name) {
                Some(&existing_variable) => existing_variable,
                None => {
                    let new_variable = self.builder.declare_var(pointer_type);
                    self.global_variables.insert(
                        variable_name_string,
                        GlobalVariable {
                            variable: new_variable,
                            dimension: GlobalVariableDimension {
                                rows: variable_dimensions.rows,
                                columns: variable_dimensions.columns,
                            },
                        },
                    );
                    GlobalVariable {
                        variable: new_variable,
                        dimension: GlobalVariableDimension {
                            rows: variable_dimensions.rows,
                            columns: variable_dimensions.columns,
                        },
                    }
                }
            };
            self.builder
                .def_var(input_variable.variable, variable_value);
            self.global_variables
                .insert(variable_name.clone(), input_variable);
        }
        Ok(())
    }

    pub fn visit_datum(&mut self, stack: &dyn Any) -> Result<String> {
        let stack_downcast = stack
            .downcast_ref::<String>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(stack_downcast.clone())
    }

    /*****************************************************************************
     *  16. READ AND RESTORE STATEMENTS                                          *
     *****************************************************************************/
    pub fn visit_read_statement(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast =
            stack
                .downcast_ref::<Vec<(String, Value)>>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        let global_data_index_variable = self
            .global_variables
            .get(utils::GLOBAL_DATA_INDEX)
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("la variable {:?} no existe", utils::GLOBAL_DATA_INDEX),
            })?;
        let mut global_data_index_value = self.builder.use_var(global_data_index_variable.variable);
        let block1 = self.builder.create_block();
        let block2 = self.builder.create_block();
        let global_data_length = self.global_variables.get(utils::GLOBAL_DATA_LEN).ok_or(
            MinBas78Error::RuntimeError {
                message: format!("la variable {:?} no existe", utils::GLOBAL_DATA_LEN),
            },
        )?;
        let global_data_length_value = self.builder.use_var(global_data_length.variable);
        let condition_value = self.builder.ins().icmp(
            IntCC::SignedGreaterThanOrEqual,
            global_data_index_value,
            global_data_length_value,
        );
        self.builder
            .ins()
            .brif(condition_value, block1, &[], block2, &[]);
        self.builder.switch_to_block(block1);
        self.builder.ins().trap(TrapCode::unwrap_user(1_u8));
        self.builder.switch_to_block(block2);
        stack_downcast.iter().try_for_each(|(stack_string, _)| {
            let target_index_value = global_data_index_value;
            let global_data_variable = self.global_variables.get(utils::GLOBAL_DATA).ok_or(
                MinBas78Error::RuntimeError {
                    message: format!("la variable {:?} no existe", utils::GLOBAL_DATA),
                },
            )?;
            let global_data_pointer_value = self.builder.use_var(global_data_variable.variable);
            let bytes_size_of_f64 = self.builder.ins().iconst(self.pointer_type, 8_i64);
            let byte_offset_value = self
                .builder
                .ins()
                .imul(target_index_value, bytes_size_of_f64);
            let target_element_pointer_value = self
                .builder
                .ins()
                .iadd(global_data_pointer_value, byte_offset_value);
            let memory_flags_data = MemFlagsData::new();
            let global_data_value = self.builder.ins().load(
                types::F64,
                memory_flags_data,
                target_element_pointer_value,
                0,
            );
            let v1 =
                self.global_variables
                    .get(stack_string)
                    .ok_or(MinBas78Error::RuntimeError {
                        message: format!("la variable {:?} no existe", stack_string),
                    })?;
            self.builder.def_var(v1.variable, global_data_value);
            let one_i64 = self.builder.ins().iconst(types::I64, 1_i64);
            global_data_index_value = self.builder.ins().iadd(global_data_index_value, one_i64);
            self.builder
                .def_var(global_data_index_variable.variable, global_data_index_value);
            anyhow::Ok(())
        })?;
        Ok(())
    }

    pub fn visit_restore_statement(&mut self) -> () {
        self.global_data_index = usize::default();
    }

    /*****************************************************************************
     *  17. DATA STATEMENT                                                       *
     *****************************************************************************/
    pub fn visit_data_statement(&mut self) {}

    pub fn visit_data_list(&mut self, stack: &mut dyn Any) -> Result<()> {
        let stack_downcast =
            stack
                .downcast_mut::<Vec<String>>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        self.global_data.append(stack_downcast);
        Ok(())
    }

    /*****************************************************************************
     *  18. ARRAY DECLARATIONS                                                   *
     *****************************************************************************/
    pub fn visit_dimension_statement(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast = stack
            .downcast_ref::<Vec<(String, Vec<(String, Value)>)>>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        stack_downcast
            .iter()
            .try_for_each(|(variable_name, variable_dimensions)| {
                let mut variable_name_string = format!("{}(", variable_name.clone());
                let mut rows_dimension = u8::default();
                let mut columns_dimension = None::<u8>;
                variable_dimensions.iter().enumerate().try_for_each(
                    |(i, (variable_dimension_string, _))| {
                        variable_name_string = format!(
                            "{}{}{}",
                            variable_name_string,
                            variable_dimension_string.clone(),
                            if i < variable_dimensions.len() - 1 {
                                utils::COMMA
                            } else {
                                ""
                            }
                        );
                        if i == usize::default() {
                            rows_dimension = variable_dimension_string.parse::<u8>()?;
                        } else if i == 1 {
                            columns_dimension = Some(variable_dimension_string.parse::<u8>()?);
                        }
                        anyhow::Ok(())
                    },
                )?;
                variable_name_string = format!("{})", variable_name_string);

                if rows_dimension < self.option_base {
                    return Err(anyhow!(MinBas78Error::RuntimeError {
                        message: format!(
                            "la primera dimension de {:?} es menor a la option base {}",
                            variable_name_string, self.option_base
                        )
                    }));
                }
                if columns_dimension.is_some() {
                    if columns_dimension.ok_or(MinBas78Error::RuntimeError {
                        message: format!("sd error"),
                    })? < self.option_base
                    {
                        return Err(anyhow!(MinBas78Error::RuntimeError {
                            message: format!(
                                "la segunda dimension de {:?} es menor a la option base {}",
                                variable_name_string, self.option_base
                            )
                        }));
                    }
                }
                let variable_before = self.global_variables_dimensions.get(variable_name);
                if variable_before.is_some() {
                    return Err(anyhow!(MinBas78Error::RuntimeError {
                        message: format!(
                            "la variable {:?} solo se puede dimensionar una sola vez",
                            &variable_name
                        )
                    }));
                }
                self.array_dim(
                    &variable_name.clone(),
                    &GlobalVariableDimension {
                        rows: rows_dimension,
                        columns: columns_dimension,
                    },
                )?;
                variable_name_string = String::default();
                Ok(())
            })?;
        Ok(())
    }

    pub fn visit_array_declaration(
        &mut self,
        stack: &dyn Any,
    ) -> Result<(String, Vec<(String, Value)>)> {
        let stack_downcast = stack
            .downcast_ref::<(String, Vec<(String, Value)>)>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;
        Ok(stack_downcast.clone())
    }

    pub fn visit_bounds(&mut self, stack: &dyn Any) -> Result<Vec<(String, Value)>> {
        let stack_downcast =
            stack
                .downcast_ref::<Vec<(String, Value)>>()
                .ok_or(MinBas78Error::RuntimeError {
                    message: format!("stack error"),
                })?;
        Ok(stack_downcast.clone())
    }

    pub fn visit_option_statement(&mut self, stack: &dyn Any) -> Result<()> {
        let stack_downcast = stack
            .downcast_ref::<u8>()
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("stack error"),
            })?;

        self.option_base = *stack_downcast;
        Ok(())
    }

    pub fn visit_option_base(&mut self, node: &OptionBase) -> Result<u8> {
        let option_base_string = node.option_base.trim().to_string();
        let option_base = option_base_string.parse::<u8>()?;
        Ok(option_base)
    }

    /*****************************************************************************
     *  19. REMARK STATEMENT                                                     *
     *****************************************************************************/
    pub fn visit_remark_statement(&mut self) {}

    /*****************************************************************************
     *  20. RANDOMIZE STATEMENT                                                  *
     *****************************************************************************/
    pub fn visit_randomize_statement(&mut self) -> Result<()> {
        let current_timestamp = SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_secs() as f64;
        let seed = current_timestamp % f64::MAX;
        let seed_value = self.builder.ins().f64const(seed);
        let seed_variable = self
            .global_variables
            .get(&String::from(utils::SEED))
            .ok_or(MinBas78Error::RuntimeError {
                message: format!("la variable {:?} no existe", utils::SEED),
            })?;
        self.builder.def_var(seed_variable.variable, seed_value);
        Ok(())
    }
}
