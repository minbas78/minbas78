mod abstract_syntax_tree;
mod arguments;
mod external_functions;
mod grammars;
mod lines_walker;
mod user_defined_function_walker;
mod utils;
mod visitor;
mod walker;

use crate::{
    arguments::Arguments, lines_walker::BasicAbstractSyntaxTreeLineWalker, utils::MinBas78Error,
    visitor::Visitor, walker::Walker,
};
use anyhow::Result;
use clap::{Parser as ClapParser, error::ErrorKind};
use cranelift::frontend::FunctionBuilder;
use cranelift_codegen::{
    Context as CraneliftCodegenContext,
    ir::{Block, InstBuilder, types},
};
use cranelift_module::Module;
use std::{fs, path::Path, process::ExitCode};

fn main() -> ExitCode {
    if let Err(error) = run() {
        if MinBas78Error::eprint_error(&error).is_err() {
            return ExitCode::FAILURE;
        }
        if let Some(minbas78_error) = error.downcast_ref::<MinBas78Error>() {
            return minbas78_error.exit_code();
        } else {
            return ExitCode::FAILURE;
        }
    } else {
        return ExitCode::SUCCESS;
    }
}

fn run() -> Result<()> {
    let arguments = match Arguments::try_parse() {
        Ok(parsed_arguments) => Ok(parsed_arguments),
        Err(error) => match error.kind() {
            ErrorKind::MissingRequiredArgument => Err(MinBas78Error::RuntimeError {
                message: format!(
                    "{}{}{}Usage: {} [OPTIONS] INPUT",
                    "no input file provided",
                    utils::NEW_LINE,
                    utils::NEW_LINE,
                    env!("CARGO_PKG_NAME")
                ),
            }),
            ErrorKind::UnknownArgument => Err(MinBas78Error::RuntimeError {
                message: format!(
                    "{}{}{}Usage: {} [OPTIONS] INPUT",
                    "unknown argument",
                    utils::NEW_LINE,
                    utils::NEW_LINE,
                    env!("CARGO_PKG_NAME")
                ),
            }),
            _ => error.exit(),
        },
    }?;
    let minbas78_settings = Visitor::get_minbas78_settings()?;
    let input_bas_folder =
        minbas78_settings
            .get("input_bas_folder")
            .ok_or(MinBas78Error::RuntimeError {
                message: String::from("input_bas_folder"),
            })?;
    let output_rusty_object_notation_folder = minbas78_settings
        .get("output_rusty_object_notation_folder")
        .ok_or(MinBas78Error::RuntimeError {
            message: String::from("output_rusty_object_notation_folder"),
        })?;
    let output_rusty_object_notation_extension = minbas78_settings
        .get("output_rusty_object_notation_extension")
        .ok_or(MinBas78Error::RuntimeError {
            message: String::from("output_rusty_object_notation_extension"),
        })?;
    let program_cranelift_intermediate_format_folder = minbas78_settings
        .get("program_cranelift_intermediate_format_folder")
        .ok_or(MinBas78Error::RuntimeError {
            message: String::from("program_cranelift_intermediate_format_folder"),
        })?;
    let program_cranelift_intermediate_format_extension = minbas78_settings
        .get("program_cranelift_intermediate_format_extension")
        .ok_or(MinBas78Error::RuntimeError {
            message: String::from("program_cranelift_intermediate_format_extension"),
        })?;
    let program_intermediate_representation_folder = minbas78_settings
        .get("program_intermediate_representation_folder")
        .ok_or(MinBas78Error::RuntimeError {
            message: String::from("program_intermediate_representation_folder"),
        })?;
    let program_intermediate_representation_extension = minbas78_settings
        .get("program_intermediate_representation_extension")
        .ok_or(MinBas78Error::RuntimeError {
            message: String::from("program_intermediate_representation_extension"),
        })?;
    let program_object_folder =
        minbas78_settings
            .get("program_object_folder")
            .ok_or(MinBas78Error::RuntimeError {
                message: String::from("program_object_folder"),
            })?;
    let program_object_extension =
        minbas78_settings
            .get("program_object_extension")
            .ok_or(MinBas78Error::RuntimeError {
                message: String::from("program_object_extension"),
            })?;
    let program_executable_folder =
        minbas78_settings
            .get("program_executable_folder")
            .ok_or(MinBas78Error::RuntimeError {
                message: String::from("program_executable_folder"),
            })?;
    let program_executable_extension = minbas78_settings
        .get("program_executable_extension")
        .ok_or(MinBas78Error::RuntimeError {
            message: String::from("program_executable_extension"),
        })?;
    let input_bas_filename = arguments
        .input
        .to_str()
        .ok_or(MinBas78Error::RuntimeError {
            message: String::from("input convert error"),
        })?
        .to_string();
    let filename = input_bas_filename
        .split(utils::FULL_STOP)
        .next()
        .ok_or(MinBas78Error::SystemError {
            filenames: vec![input_bas_filename.to_string()],
            message: Some(String::from("incorrect format")),
        })?
        .to_string();
    let input_bas_file = &Path::new(&input_bas_folder).join(&input_bas_filename);
    let output_rusty_object_notation_filename =
        Path::new(&filename).with_extension(&output_rusty_object_notation_extension);
    let output_rusty_object_notation_file = &Path::new(&output_rusty_object_notation_folder)
        .join(&output_rusty_object_notation_filename);
    let program_cranelift_intermediate_format_filename =
        Path::new(&filename).with_extension(&program_cranelift_intermediate_format_extension);
    let program_cranelift_intermediate_format_file =
        &Path::new(&program_cranelift_intermediate_format_folder)
            .join(&program_cranelift_intermediate_format_filename);
    let program_intermediate_representation_filename =
        Path::new(&filename).with_extension(&program_intermediate_representation_extension);
    let program_intermediate_representation_file =
        &Path::new(&program_intermediate_representation_folder)
            .join(&program_intermediate_representation_filename);
    let program_object_library_filename =
        Path::new(&filename).with_extension(&program_object_extension);
    let program_object_file =
        &Path::new(&program_object_folder).join(&program_object_library_filename);
    let program_library_folder =
        minbas78_settings
            .get("program_library_folder")
            .ok_or(MinBas78Error::RuntimeError {
                message: String::from("program_library_folder"),
            })?;
    let program_library_extension =
        minbas78_settings
            .get("program_library_extension")
            .ok_or(MinBas78Error::RuntimeError {
                message: String::from("program_library_extension"),
            })?;
    let program_library_filename =
        Path::new(&arguments::MINBAS78).with_extension(program_library_extension);
    let program_library_file = &Path::new(program_library_folder).join(program_library_filename);
    let program_executable_filename = arguments
        .output
        .unwrap_or_else(|| Path::new(&filename).with_extension(&program_executable_extension));
    let program_executable_file =
        &Path::new(&program_executable_folder).join(&program_executable_filename);
    let program = Visitor::get_program(input_bas_file)?;
    let (mut module, pointer_type) =
        utils::create_module().map_err(|error| MinBas78Error::CompilerError {
            message: error.to_string(),
        })?;
    let (function_id, mut function, mut function_builder_context) =
        utils::create_main_function(&mut module).map_err(|error| MinBas78Error::CompilerError {
            message: error.to_string(),
        })?;
    let mut builder = FunctionBuilder::new(&mut function, &mut function_builder_context);
    let external_functions =
        utils::create_external_functions(&mut module, &mut builder, &pointer_type).map_err(
            |error| MinBas78Error::CompilerError {
                message: error.to_string(),
            },
        )?;
    let mut visitor = Visitor::new(
        &pointer_type,
        &mut builder,
        &mut module,
        &external_functions,
    );
    let block = visitor.builder.create_block();
    visitor.builder.switch_to_block(block);
    visitor.initialize_seed();
    visitor.initialize_global_data_index();
    let mut lines_walker = BasicAbstractSyntaxTreeLineWalker;
    lines_walker.walk_program(&mut visitor, &program)?;
    let mut walker = Walker;
    walker.walk_program(&mut visitor, &program)?;
    let blocks = visitor.blocks.values().cloned().collect::<Vec<Block>>();
    for block in blocks {
        let has_terminator = if let Some(instruction) = visitor.builder.func.layout.last_inst(block)
        {
            let instruction_data = &visitor.builder.func.dfg.insts[instruction];
            let has_terminator = instruction_data.opcode().is_terminator();
            has_terminator
        } else {
            false
        };
        if !has_terminator {
            visitor.builder.switch_to_block(block);
            let return_value = visitor.builder.ins().iconst(types::I64, i64::default());
            visitor.builder.ins().return_(&[return_value]);
        }
    }
    visitor.builder.seal_all_blocks();
    if arguments.clif {
        visitor
            .generate_intermediate_representation_file(&program_cranelift_intermediate_format_file)
            .map_err(|error| MinBas78Error::SystemError {
                filenames: vec![],
                message: Some(error.to_string()),
            })?;
    }
    if arguments.dot {
        visitor
            .generate_dot_file(&program_intermediate_representation_file)
            .map_err(|error| MinBas78Error::SystemError {
                filenames: vec![],
                message: Some(error.to_string()),
            })?;
    }
    if arguments.ron {
        fs::write(
            &output_rusty_object_notation_file,
            format!("{:#?}", program),
        )
        .map_err(|error| MinBas78Error::SystemError {
            filenames: vec![
                output_rusty_object_notation_file
                    .to_string_lossy()
                    .to_string(),
            ],
            message: Some(error.to_string()),
        })?;
    }
    let mut context = CraneliftCodegenContext::for_function(function);
    module
        .define_function(function_id, &mut context)
        .map_err(|error| MinBas78Error::CompilerError {
            message: error.to_string(),
        })?;
    let object_product = module.finish();
    let object_product_bytes =
        object_product
            .emit()
            .map_err(|error| MinBas78Error::CompilerError {
                message: error.to_string(),
            })?;
    Visitor::generate_object_library_file(&program_object_file, &object_product_bytes).map_err(
        |error| MinBas78Error::SystemError {
            filenames: vec![program_object_file.to_string_lossy().to_string()],
            message: Some(error.to_string()),
        },
    )?;
    Visitor::generate_executable_file(
        program_object_file,
        program_library_file,
        program_executable_file,
    )
    .map_err(|error| MinBas78Error::SystemError {
        filenames: vec![
            program_object_file.to_string_lossy().to_string(),
            program_executable_file.to_string_lossy().to_string(),
        ],
        message: Some(error.to_string()),
    })?;
    Ok(())
}
