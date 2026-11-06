use anyhow::Result;
use cranelift::frontend::FunctionBuilder;
use from_pest::FromPest;
use minbas78::{
    abstract_syntax_tree::Program,
    grammars::basic_pest::{MinBas78, Rule as RuleBasic},
    lines_walker, utils,
    visitor::Visitor,
    walker,
};
use pest::Parser;
use std::collections::BTreeMap;

#[test]
fn test() -> Result<()> {
    let (mut module, pointer_type) = utils::create_module()?;
    let (_, mut func, mut func_ctx) = utils::create_main_function(&mut module)?;
    let mut builder = FunctionBuilder::new(&mut func, &mut func_ctx);
    let external_functions =
        utils::create_external_functions(&mut module, &mut builder, &pointer_type)?;
    let mut ast_compiler = Visitor::new(
        &pointer_type,
        &mut builder,
        &mut module,
        &external_functions,
    );
    let mut input_bas_unordered_lines = BTreeMap::<u32, String>::new();
    input_bas_unordered_lines.insert(
        10,
        format!(r#"10 REM IF YOU ARE NERD THEN MINBAS78{}"#, utils::NEW_LINE),
    );
    input_bas_unordered_lines.insert(20, format!(r#"20 LET Y$ = "NERD"{}"#, utils::NEW_LINE));
    input_bas_unordered_lines.insert(
        30,
        format!(r#"30 IF Y$ = "NERD" THEN 78{}"#, utils::NEW_LINE),
    );
    input_bas_unordered_lines.insert(
        78,
        format!(
            r#"78 PRINT {:?}{}"#,
            env!("MINBAS78").to_uppercase(),
            utils::NEW_LINE
        ),
    );
    input_bas_unordered_lines.insert(99, format!(r#"99 END{}"#, utils::NEW_LINE));
    let input_bas_string_sorted = input_bas_unordered_lines.into_values().collect::<String>();
    let mut pairs = MinBas78::parse(RuleBasic::program, &input_bas_string_sorted)?;
    let ast = Program::from_pest(&mut pairs)?;
    let block = ast_compiler.builder.create_block();
    ast_compiler.builder.switch_to_block(block);
    let mut lines_walker = lines_walker::BasicAbstractSyntaxTreeLineWalker;
    lines_walker.walk_program(&mut ast_compiler, &ast)?;
    let mut walker = walker::Walker;
    walker.walk_program(&mut ast_compiler, &ast)?;
    Ok(())
}
