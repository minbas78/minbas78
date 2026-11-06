//! # Abstract Syntax Tree.
//!
//! This module declares the public structs of the abstract syntax tree.
//!
use crate::{grammars::basic_pest::Rule as RuleBasic, utils};
use pest_ast::FromPest;

/*****************************************************************************
 *  5. PROGRAMS                                                              *
 *****************************************************************************/
/// # pub struct `Program`.
///
/// Struct for the `program` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::program))]
pub struct Program {
    pub block: Vec<Block>,
    pub end_line: EndLine,
    pub _eoi: EOI,
}

/// # pub enum `Block`.
///
/// Struct for the `block` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::block))]
pub enum Block {
    Line(Line),
    ForBlock(ForBlock),
}

/// # pub struct `Line`.
///
/// Struct for the `line` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::line))]
pub struct Line {
    pub line_number: LineNumber,
    pub statement: Statement,
}

/// # pub struct `LineNumber`.
///
/// Struct for the `line_number` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::line_number))]
pub struct LineNumber {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub line_number: String,
}

/// # pub struct `EndLine`.
///
/// Struct for the `end_line` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::end_line))]
pub struct EndLine {
    pub line_number: LineNumber,
    pub _end_statement: EndStatement,
}

/// # pub struct `EndStatement`.
///
/// Struct for the `end_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::end_statement))]
pub struct EndStatement;

/// # pub enum `Statement`.
///
/// Struct for the `statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::statement))]
pub enum Statement {
    RemarkStatement(RemarkStatement),
    LetStatement(LetStatement),
    PrintStatement(PrintStatement),
    RandomizeStatement(RandomizeStatement),
    DefStatement(DefStatement),
    GotoStatement(GotoStatement),
    IfThenStatement(IfThenStatement),
    OnGotoStatement(OnGotoStatement),
    GoSubStatement(GosubStatement),
    ReturnStatement(ReturnStatement),
    StopStatement(StopStatement),
    InputStatement(InputStatement),
    ReadStatement(ReadStatement),
    RestoreStatement(RestoreStatement),
    DataStatement(DataStatement),
    DimensionStatement(DimensionStatement),
    OptionStatement(OptionStatement),
}

/// # pub struct `EOI`.
///
/// Struct for the `eoi` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::EOI))]
pub struct EOI;

/*****************************************************************************
 *  6. CONSTANTS                                                             *
 *****************************************************************************/

/// # pub struct `Sign`.
///
/// Struct for the `sign` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::sign))]
pub struct Sign {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub sign: String,
}

/// # pub struct `NumericRep`.
///
/// Struct for the `numeric_rep` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_rep))]
pub struct NumericRep {
    pub significand: Significand,
    pub exrad: Option<Exrad>,
}

/// # pub struct `Significand`.
///
/// Struct for the `significand` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::significand))]
pub struct Significand {
    pub integer: Option<Integer>,
    pub fraction: Option<Fraction>,
}

/// # pub struct `Integer`.
///
/// Struct for the `integer` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::integer))]
pub struct Integer {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub integer: String,
}

/// # pub struct `Fraction`.
///
/// Struct for the `fraction` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::fraction))]
pub struct Fraction {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub fraction: String,
}

/// # pub struct `Exrad`.
///
/// Struct for the `exrad` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::exrad))]
pub struct Exrad {
    pub sign: Option<Sign>,
    pub integer: Integer,
}

/// # pub struct `StringConstant`.
///
/// Struct for the `string_constant` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::string_constant))]
pub struct StringConstant {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub string_constant: String,
}

/*****************************************************************************
 *  7. VARIABLES                                                             *
 *****************************************************************************/

/// # pub enum `Variable`.
///
/// Struct for the `variable` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::variable))]
pub enum Variable {
    StringVariable(StringVariable),
    NumericVariable(NumericVariable),
}

/// # pub enum `NumericVariable`.
///
/// Struct for the `numeric_variable` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_variable))]
pub enum NumericVariable {
    NumericArrayElement(NumericArrayElement),
    SimpleNumericVariable(SimpleNumericVariable),
}

/// # pub struct `SimpleNumericVariable`.
///
/// Struct for the `simple_numeric_variable` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::simple_numeric_variable))]
pub struct SimpleNumericVariable {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub simple_numeric_variable: String,
}

/// # pub struct `NumericArrayElement`.
///
/// Struct for the `numeric_array_element` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_array_element))]
pub struct NumericArrayElement {
    pub numeric_array_name: NumericArrayName,
    pub numeric_expression: Vec<NumericExpression>,
}

/// # pub struct `NumericArrayName`.
///
/// Struct for the `numeric_array_name` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_array_name))]
pub struct NumericArrayName {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub numeric_array_name: String,
}

/// # pub struct `StringVariable`.
///
/// Struct for the `string_variable` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::string_variable))]
pub struct StringVariable {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub string_variable: String,
}

/*****************************************************************************
 *  8. EXPRESSIONS                                                           *
 *****************************************************************************/

/// # pub enum `Expression`.
///
/// Struct for the `expression` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::expression))]
pub enum Expression {
    StringExpression(StringExpression),
    NumericExpression(NumericExpression),
}

/// # pub struct `NumericExpression`.
///
/// Struct for the `numeric_expression` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_expression))]
pub struct NumericExpression {
    pub sign: Option<Sign>,
    pub term: Term,
    pub sign_term: Vec<(Sign, Term)>,
}

/// # pub struct `Term`.
///
/// Struct for the `term` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::term))]
pub struct Term {
    pub factor: Factor,
    pub multiplier_factor: Vec<(Multiplier, Factor)>,
}

/// # pub struct `Factor`.
///
/// Struct for the `factor` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::factor))]
pub struct Factor {
    pub primary: Box<Primary>,
    pub circumflex_accent_primary: Option<Box<Primary>>,
}

/// # pub struct `Multiplier`.
///
/// Struct for the `multiplier` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::multiplier))]
pub struct Multiplier {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub multiplier: String,
}

/// # pub enum `Primary`.
///
/// Struct for the `primary` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::primary))]
pub enum Primary {
    NumericFunctionRef(NumericFunctionRef),
    NumericVariable(NumericVariable),
    NumericRep(NumericRep),
    NumericExpressionOther(NumericExpressionOther),
}

/// # pub struct `NumericExpressionOther`.
///
/// Struct for the `numeric_expression_other` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_expression_other))]
pub struct NumericExpressionOther {
    pub numeric_expression: NumericExpression,
}

/// # pub struct `NumericFunctionRef`.
///
/// Struct for the `numeric_function_ref` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_function_ref))]
pub struct NumericFunctionRef {
    pub numeric_function_name: NumericFunctionName,
    pub numeric_expression: Option<NumericExpression>,
}

/// # pub enum `NumericFunctionName`.
///
/// Struct for the `numeric_function_name` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_function_name))]
pub enum NumericFunctionName {
    NumericDefinedFunction(NumericDefinedFunction),
    NumericSuppliedFunction(NumericSuppliedFunction),
}

/// # pub enum `StringExpression`.
///
/// Struct for the `string_expression` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::string_expression))]
pub enum StringExpression {
    StringVariable(StringVariable),
    StringConstant(StringConstant),
}

/*****************************************************************************
 *  9. IMPLEMENTATION SUPPLIED FUNCTIONS                                     *
 *****************************************************************************/

/// # pub struct `NumericSuppliedFunction`.
///
/// Struct for the `numeric_supplied_function` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_supplied_function))]
pub struct NumericSuppliedFunction {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub numeric_supplied_function: String,
}

/*****************************************************************************
 *  10. USER DEFINED FUNCTIONS                                               *
 *****************************************************************************/

/// # pub struct `DefStatement`.
///
/// Struct for the `def_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::def_statement))]
pub struct DefStatement {
    pub numeric_defined_function: NumericDefinedFunction,
    pub parameter_list: Option<ParameterList>,
    pub numeric_expression: NumericExpression,
}

/// # pub struct `NumericDefinedFunction`.
///
/// Struct for the `numeric_defined_function` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_defined_function))]
pub struct NumericDefinedFunction {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub numeric_defined_function: String,
}

/// # pub struct `ParameterList`.
///
/// Struct for the `parameter_list` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::parameter_list))]
pub struct ParameterList {
    pub simple_numeric_variable: SimpleNumericVariable,
}

/*****************************************************************************
 *  11. LET STATEMENT                                                        *
 *****************************************************************************/

/// # pub enum `LetStatement`.
///
/// Struct for the `let_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::let_statement))]
pub enum LetStatement {
    NumericLetStatement(NumericLetStatement),
    StringLetStatement(StringLetStatement),
}

/// # pub struct `NumericLetStatement`.
///
/// Struct for the `numeric_let_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::numeric_let_statement))]
pub struct NumericLetStatement {
    pub numeric_variable: NumericVariable,
    pub numeric_expression: NumericExpression,
}

/// # pub struct `StringLetStatement`.
///
/// Struct for the `string_let_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::string_let_statement))]
pub struct StringLetStatement {
    pub string_variable: StringVariable,
    pub string_expression: StringExpression,
}

/*****************************************************************************
 *  12. CONTROL STATEMENT                                                    *
 *****************************************************************************/

/// # pub struct `GotoStatement`.
///
/// Struct for the `goto_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::goto_statement))]
pub struct GotoStatement {
    pub line_number: LineNumber,
}

/// # pub struct `IfThenStatement`.
///
/// Struct for the `if_then_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::if_then_statement))]
pub struct IfThenStatement {
    pub relational_expression: RelationalExpression,
    pub line_number: LineNumber,
}

/// # pub enum `RelationalExpression`.
///
/// Struct for the `relational_expression` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::relational_expression))]
pub enum RelationalExpression {
    NumericExpression((NumericExpression, Relation, NumericExpression)),
    StrinExpression((StringExpression, EqualityRelation, StringExpression)),
}

/// # pub enum `Relation`.
///
/// Struct for the `relation` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::relation))]
pub enum Relation {
    EqualityRelation(EqualityRelation),
    EqualityRelationOther(#[pest_ast(outer(with(utils::span_into_string)))] String),
}

/// # pub struct `EqualityRelation`.
///
/// Struct for the `equality_relation` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::equality_relation))]
pub struct EqualityRelation {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub equality_relation: String,
}

/// # pub struct `GosubStatement`.
///
/// Struct for the `gosub_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::gosub_statement))]
pub struct GosubStatement {
    pub line_number: LineNumber,
}

/// # pub struct `ReturnStatement`.
///
/// Struct for the `return_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::return_statement))]
pub struct ReturnStatement;

/// # pub struct `OnGotoStatement`.
///
/// Struct for the `on_goto_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::on_goto_statement))]
pub struct OnGotoStatement {
    pub numeric_expression: NumericExpression,
    pub line_number: Vec<LineNumber>,
}

/// # pub struct `StopStatement`.
///
/// Struct for the `stop_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::stop_statement))]
pub struct StopStatement;

/*****************************************************************************
 *  13. FOR AND NEXT STATEMENTS                                              *
 *****************************************************************************/

/// # pub struct `ForBlock`.
///
/// Struct for the `for_block` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::for_block))]
pub struct ForBlock {
    pub for_line: ForLine,
    pub for_body: ForBody,
}

/// # pub struct `ForBody`.
///
/// Struct for the `for_body` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::for_body))]
pub struct ForBody {
    pub block: Vec<Block>,
    pub next_line: NextLine,
}

/// # pub struct `ForLine`.
///
/// Struct for the `for_line` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::for_line))]
pub struct ForLine {
    pub line_number: LineNumber,
    pub for_statement: ForStatement,
}

/// # pub struct `NextLine`.
///
/// Struct for the `next_line` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::next_line))]
pub struct NextLine {
    pub line_number: LineNumber,
    pub next_statement: NextStatement,
}

/// # pub struct `ForStatement`.
///
/// Struct for the `for_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::for_statement))]
pub struct ForStatement {
    pub control_variable: ControlVariable,
    pub initial_value: InitialValue,
    pub limit: Limit,
    pub increment: Option<Increment>,
}

/// # pub struct `ControlVariable`.
///
/// Struct for the `control_variable` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::control_variable))]
pub struct ControlVariable {
    pub control_variable: SimpleNumericVariable,
}

/// # pub struct `InitialValue`.
///
/// Struct for the `initial_value` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::initial_value))]
pub struct InitialValue {
    pub initial_value: NumericExpression,
}

/// # pub struct `Limit`.
///
/// Struct for the `limit` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::limit))]
pub struct Limit {
    pub limit: NumericExpression,
}

/// # pub struct `Increment`.
///
/// Struct for the `increment` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::increment))]
pub struct Increment {
    pub increment: NumericExpression,
}

/// # pub struct `NextStatement`.
///
/// Struct for the `next_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::next_statement))]
pub struct NextStatement {
    pub control_variable: ControlVariable,
}

/*****************************************************************************
 *  14. PRINT STATEMENT                                                      *
 *****************************************************************************/

/// # pub struct `PrintStatement`.
///
/// Struct for the `print_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::print_statement))]
pub struct PrintStatement {
    pub print_list: PrintList,
}

/// # pub struct `PrintList`.
///
/// Struct for the `print_list` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::print_list))]
pub struct PrintList {
    pub print_item_separator: Vec<(Option<PrintItem>, PrintSeparator)>,
    pub print_item: Option<PrintItem>,
}

/// # pub struct `PrintSeparator`.
///
/// Struct for the `print_separator` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::print_separator))]
pub struct PrintSeparator {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub print_separator: String,
}

/// # pub enum `PrintItem`.
///
/// Struct for the `print_item` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::print_item))]
pub enum PrintItem {
    TabCall(TabCall),
    Expression(Expression),
}

/// # pub struct `TabCall`.
///
/// Struct for the `tab_call` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::tab_call))]
pub struct TabCall {
    pub numeric_expression: NumericExpression,
}

/*****************************************************************************
 *  15. INPUT STATEMENT                                                      *
 *****************************************************************************/

/// # pub struct `InputStatement`.
///
/// Struct for the `input_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::input_statement))]
pub struct InputStatement {
    pub variable: Vec<Variable>,
}

/// # pub struct `Datum`.
///
/// Struct for the `datum` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::datum))]
pub struct Datum {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub datum: String,
}

/*****************************************************************************
 *  16. READ AND RESTORE STATEMENTS                                          *
 *****************************************************************************/

/// # pub struct `ReadStatement`.
///
/// Struct for the `read_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::read_statement))]
pub struct ReadStatement {
    pub variable_list: Vec<Variable>,
}

/// # pub struct `RestoreStatement`.
///
/// Struct for the `restore_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::restore_statement))]
pub struct RestoreStatement;

/*****************************************************************************
 *  17. DATA STATEMENT                                                       *
 *****************************************************************************/

/// # pub struct `DataStatement`.
///
/// Struct for the `data_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::data_statement))]
pub struct DataStatement {
    pub data_list: DataList,
}

/// # pub struct `DataList`.
///
/// Struct for the `data_list` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::data_list))]
pub struct DataList {
    pub datums: Vec<Datum>,
}

/*****************************************************************************
 *  18. ARRAY DECLARATIONS                                                   *
 *****************************************************************************/

/// # pub struct `DimensionStatement`.
///
/// Struct for the `dimension_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::dimension_statement))]
pub struct DimensionStatement {
    pub array_declaration: Vec<ArrayDeclaration>,
}

/// # pub struct `ArrayDeclaration`.
///
/// Struct for the `array_declaration` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::array_declaration))]
pub struct ArrayDeclaration {
    pub numeric_array_name: NumericArrayName,
    pub bound: Bounds,
}

/// # pub struct `Bounds`.
///
/// Struct for the `bounds` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::bounds))]
pub struct Bounds {
    pub integer: Vec<Integer>,
}

/// # pub struct `OptionStatement`.
///
/// Struct for the `option_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::option_statement))]
pub struct OptionStatement {
    pub option_base: OptionBase,
}

/// # pub struct `OptionBase`.
///
/// Struct for the `option_base` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::option_base))]
pub struct OptionBase {
    #[pest_ast(outer(with(utils::span_into_string)))]
    pub option_base: String,
}

/*****************************************************************************
 *  19. REMARK STATEMENT                                                     *
 *****************************************************************************/

/// # pub struct `RemarkStatement`.
///
/// Struct for the `remark_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::remark_statement))]
pub struct RemarkStatement;

/*****************************************************************************
 *  20. RANDOMIZE STATEMENT                                                  *
 *****************************************************************************/

/// # pub struct `RandomizeStatement`.
///
/// Struct for the `randomize_statement` grammar rule.
///
#[derive(Debug, Clone, FromPest)]
#[pest_ast(rule(RuleBasic::randomize_statement))]
pub struct RandomizeStatement;
