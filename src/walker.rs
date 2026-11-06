//! # Abstract Syntax Tree Walker.
//!
//! This module declares the public abstract syntax tree walker.
//!
use crate::{
    abstract_syntax_tree::{
        ArrayDeclaration, Block, Bounds, ControlVariable, DefStatement, DimensionStatement,
        EndLine, EqualityRelation, Expression, Exrad, Factor, ForBlock, ForBody, ForLine,
        ForStatement, GosubStatement, GotoStatement, IfThenStatement, Increment, InitialValue,
        InputStatement, Integer, LetStatement, Limit, Line, LineNumber, Multiplier, NextLine,
        NextStatement, NumericArrayElement, NumericArrayName, NumericDefinedFunction,
        NumericExpression, NumericExpressionOther, NumericFunctionName, NumericFunctionRef,
        NumericLetStatement, NumericRep, NumericSuppliedFunction, NumericVariable, OnGotoStatement,
        OptionBase, OptionStatement, ParameterList, Primary, PrintItem, PrintList, PrintSeparator,
        PrintStatement, Program, ReadStatement, Relation, RelationalExpression, Sign, Significand,
        SimpleNumericVariable, Statement, StringExpression, StringLetStatement, StringVariable,
        TabCall, Term, Variable,
    },
    visitor::Visitor,
};
use anyhow::Result;
use cranelift_codegen::ir::Value;

/// # pub struct `Walker`.
///
/// Struct for the walks methods.
///
/// _This structs holds the walks methods for the abstract syntaxes tree._
///
pub struct Walker;

impl Walker {
    /*****************************************************************************
     *  5. PROGRAMS                                                              *
     *****************************************************************************/
    /// # pub fn `walk_program`.
    ///
    /// Walker for the `program` grammar rule.
    ///
    /// _This function walks the `program` grammar rule
    /// and all it dependendant rules._
    ///
    pub fn walk_program(&mut self, visitor: &mut Visitor, node: &Program) -> Result<()> {
        node.block
            .iter()
            .try_for_each(|block| self.walk_block(visitor, &false, &block))?;
        self.walk_end_line(visitor, &node.end_line)?;
        visitor.visit_eoi();
        visitor.visit_program();
        Ok(())
    }

    fn walk_block(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Block,
    ) -> Result<()> {
        match node {
            Block::Line(line) => self.walk_line(visitor, &is_local_for_variable, &line)?,
            Block::ForBlock(for_block) => {
                self.walk_for_block(visitor, &is_local_for_variable, &for_block)?
            }
        };
        visitor.visit_block();
        Ok(())
    }

    fn walk_line(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Line,
    ) -> Result<()> {
        let line_number = self.walk_line_number(visitor, &node.line_number);
        visitor.visit_pre_line(&line_number)?;
        self.walk_statement(visitor, &is_local_for_variable, &node.statement)?;
        visitor.visit_line(&line_number)?;
        Ok(())
    }

    fn walk_line_number(&mut self, visitor: &mut Visitor, node: &LineNumber) -> String {
        let line_number = node.line_number.trim().to_string();
        visitor.visit_line_number();
        line_number
    }

    fn walk_end_line(&mut self, visitor: &mut Visitor, node: &EndLine) -> Result<()> {
        let line_number = self.walk_line_number(visitor, &node.line_number);
        visitor.visit_end_line(&line_number)?;
        Ok(())
    }

    fn walk_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Statement,
    ) -> Result<()> {
        match node {
            Statement::RemarkStatement(_) => {
                self.walk_remark_statement(visitor);
            }
            Statement::LetStatement(let_statement) => {
                self.walk_let_statement(visitor, is_local_for_variable, let_statement)?;
            }
            Statement::PrintStatement(print_statement) => {
                self.walk_print_statement(visitor, is_local_for_variable, print_statement)?;
            }
            Statement::RandomizeStatement(_) => {
                self.walk_randomize_statement(visitor)?;
            }
            Statement::DefStatement(def_statement) => {
                self.walk_def_statement(visitor, &is_local_for_variable, def_statement)?;
            }
            Statement::GotoStatement(goto_statement) => {
                self.walk_goto_statement(visitor, goto_statement)?;
            }
            Statement::IfThenStatement(if_then_statement) => {
                self.walk_if_then_statement(visitor, &is_local_for_variable, if_then_statement)?;
            }
            Statement::OnGotoStatement(on_goto_statement) => {
                self.walk_on_goto_statement(visitor, &is_local_for_variable, on_goto_statement)?;
            }
            Statement::GoSubStatement(gosub_statement) => {
                self.walk_gosub_statement(visitor, gosub_statement)?;
            }
            Statement::ReturnStatement(_) => {
                self.walk_return_statement(visitor)?;
            }
            Statement::StopStatement(_) => {
                self.walk_stop_statement(visitor);
            }
            Statement::InputStatement(input_statement) => {
                self.walk_input_statement(visitor, &is_local_for_variable, input_statement)?;
            }
            Statement::ReadStatement(read_statement) => {
                self.walk_read_statement(visitor, &is_local_for_variable, read_statement)?;
            }
            Statement::RestoreStatement(_) => {
                self.walk_restore_statement(visitor);
            }
            Statement::DataStatement(_) => {
                self.walk_data_statement();
            }
            Statement::DimensionStatement(dimension_statement) => {
                self.walk_dimension_statement(
                    visitor,
                    &is_local_for_variable,
                    dimension_statement,
                )?;
            }
            Statement::OptionStatement(option_statement) => {
                self.walk_option_statement(visitor, option_statement)?;
            }
        };
        visitor.visit_statement();
        Ok(())
    }

    /*****************************************************************************
     *  6. CONSTANTS                                                             *
     *****************************************************************************/
    fn walk_numeric_rep(
        &mut self,
        visitor: &mut Visitor,
        node: &NumericRep,
    ) -> Result<(String, Value)> {
        let mut stack = Vec::new();
        let significand = self.walk_significand(visitor, &node.significand)?;
        stack.push(significand);
        if let Some(exrad_node) = &node.exrad {
            let exrad = self.walk_exrad(visitor, exrad_node)?;
            stack.push((String::default(), exrad));
        }
        let numeric_rep = visitor.visit_numeric_rep(&mut stack, node)?;
        Ok(numeric_rep)
    }

    fn walk_significand(
        &mut self,
        visitor: &mut Visitor,
        node: &Significand,
    ) -> Result<(String, Value)> {
        let mut stack = Vec::new();
        if let Some(integer_some) = &node.integer {
            let integer = visitor.visit_integer(&integer_some)?;
            stack.push(integer);
        }
        if let Some(fraction_some) = &node.fraction {
            let fraction = visitor.visit_fraction(&fraction_some)?;
            stack.push((String::default(), fraction));
        }
        let significand = visitor.visit_significand(&mut stack, node)?;
        Ok(significand)
    }

    fn walk_integer(&mut self, visitor: &mut Visitor, node: &Integer) -> Result<(String, Value)> {
        let integer = visitor.visit_integer(node)?;
        Ok(integer)
    }

    fn walk_exrad(&mut self, visitor: &mut Visitor, node: &Exrad) -> Result<Value> {
        let mut stack = Vec::new();
        if let Some(sign_some) = &node.sign {
            let sign = visitor.visit_sign(sign_some)?;
            stack.push(sign);
        }
        let (_, integer_value) = visitor.visit_integer(&node.integer)?;
        stack.push(integer_value);
        let exrad = visitor.visit_exrad(&mut stack, node)?;
        Ok(exrad)
    }

    /*****************************************************************************
     *  7. VARIABLES                                                             *
     *****************************************************************************/

    fn walk_variable(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Variable,
    ) -> Result<(String, Value)> {
        let stack = match node {
            Variable::NumericVariable(numeric_variable) => {
                self.walk_numeric_variable(visitor, is_local_for_variable, &numeric_variable)?
            }
            Variable::StringVariable(string_variable) => {
                self.walk_string_variable(visitor, &string_variable)?
            }
        };
        let variable = visitor.visit_variable(&stack);
        Ok(variable)
    }

    fn walk_numeric_variable(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NumericVariable,
    ) -> Result<(String, Value)> {
        let stack = match node {
            NumericVariable::SimpleNumericVariable(simple_numeric_variable) => self
                .walk_simple_numeric_variable(
                    visitor,
                    &is_local_for_variable,
                    &simple_numeric_variable,
                ),
            NumericVariable::NumericArrayElement(numeric_array_element) => self
                .walk_numeric_array_element(
                    visitor,
                    &is_local_for_variable,
                    numeric_array_element,
                )?,
        };
        let numeric_variable = visitor.visit_numeric_variable(&stack)?;
        Ok(numeric_variable)
    }

    fn walk_simple_numeric_variable(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &SimpleNumericVariable,
    ) -> String {
        let simple_numeric_variable =
            visitor.visit_simple_numeric_variable(&is_local_for_variable, node);
        simple_numeric_variable
    }

    fn walk_numeric_array_element(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NumericArrayElement,
    ) -> Result<String> {
        let numeric_array_name =
            self.walk_numeric_array_name(visitor, is_local_for_variable, &node.numeric_array_name);
        let mut numeric_expressions = Vec::new();
        node.numeric_expression
            .iter()
            .try_for_each(|numeric_expression_item| {
                let numeric_expression = self.walk_numeric_expression(
                    visitor,
                    &is_local_for_variable,
                    &numeric_expression_item,
                )?;
                numeric_expressions.push(numeric_expression);
                anyhow::Ok(())
            })?;
        let numeric_array_element =
            visitor.visit_numeric_array_element(&(numeric_array_name, numeric_expressions))?;
        Ok(numeric_array_element)
    }

    fn walk_numeric_array_name(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NumericArrayName,
    ) -> String {
        visitor.visit_numeric_array_name(&is_local_for_variable, node)
    }

    fn walk_string_variable(
        &mut self,
        visitor: &mut Visitor,
        node: &StringVariable,
    ) -> Result<(String, Value)> {
        let string_variable = visitor.visit_string_variable(node)?;
        Ok(string_variable)
    }

    /*****************************************************************************
     *  8. EXPRESSIONS                                                           *
     *****************************************************************************/
    fn walk_expression(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Expression,
    ) -> Result<Value> {
        let stack = match node {
            Expression::NumericExpression(numeric_expresion) => {
                let (_, numeric_expression_value) = self.walk_numeric_expression(
                    visitor,
                    &is_local_for_variable,
                    numeric_expresion,
                )?;
                numeric_expression_value
            }
            Expression::StringExpression(string_expression) => {
                let (_, string_expression_value) =
                    self.walk_string_expressions(visitor, string_expression)?;
                string_expression_value
            }
        };
        let expression = visitor.visit_expression(&stack, node)?;
        Ok(expression)
    }

    fn walk_numeric_expression(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NumericExpression,
    ) -> Result<(String, Value)> {
        let mut stack = Vec::new();
        if let Some(sign_node) = &node.sign {
            let sign = visitor.visit_sign(sign_node)?;
            stack.push(sign);
        }
        let term = self.walk_term(visitor, &is_local_for_variable, &node.term)?;
        let (term_string, term_value) = term;
        stack.push(term_value);
        if !node.sign_term.is_empty() {
            let mut sign_term =
                self.walk_sign_term(visitor, &is_local_for_variable, &node.sign_term)?;
            stack.append(&mut sign_term);
        }
        let numeric_expression = visitor.visit_numeric_expression(&mut stack, node)?;
        Ok((term_string, numeric_expression))
    }

    fn walk_term(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Term,
    ) -> Result<(String, Value)> {
        let mut stack = Vec::new();
        let factor = self.walk_factor(visitor, &is_local_for_variable, &node.factor)?;
        stack.push(factor);
        if node.multiplier_factor.is_empty().eq(&false) {
            let mut multiplier_factor = self.walk_multiplier_factor(
                visitor,
                is_local_for_variable,
                &node.multiplier_factor.clone(),
            )?;
            stack.append(&mut multiplier_factor);
        }
        let term = visitor.visit_term(&mut stack)?;
        Ok(term)
    }

    fn walk_multiplier_factor(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Vec<(Multiplier, Factor)>,
    ) -> Result<Vec<(String, Value)>> {
        let mut stack = Vec::new();
        node.iter().try_for_each(|(multiplier, factor)| {
            let factor = self.walk_factor(visitor, &is_local_for_variable, &factor)?;
            stack.push(factor);
            let multiplier = visitor.visit_multiplier(&multiplier);
            stack.push((String::default(), multiplier));
            anyhow::Ok(())
        })?;
        let multiplier_factor = visitor.visit_multiplier_factor(&mut stack)?;
        Ok(multiplier_factor)
    }

    fn walk_factor(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Factor,
    ) -> Result<(String, Value)> {
        let mut stack = Vec::new();
        let primary = self.walk_primary(visitor, &is_local_for_variable, &node.primary)?;
        stack.push(primary);
        if let Some(accent_node) = &node.circumflex_accent_primary {
            let circumflex_accent_primary =
                self.walk_circumflex_accent_primary(visitor, &is_local_for_variable, accent_node)?;
            stack.push(circumflex_accent_primary);
        }
        let factor = visitor.visit_factor(&mut stack, node)?;
        Ok(factor)
    }

    fn walk_circumflex_accent_primary(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Box<Primary>,
    ) -> Result<(String, Value)> {
        let primary = node.as_ref().to_owned();
        let (stack_string, mut stack_value) =
            self.walk_primary(visitor, &is_local_for_variable, &primary)?;
        let circumflex_accent_primary =
            visitor.visit_circumflex_accent_primary(&mut stack_value)?;
        Ok((stack_string, circumflex_accent_primary))
    }

    fn walk_primary(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Primary,
    ) -> Result<(String, Value)> {
        let mut stack = Vec::new();
        match node {
            Primary::NumericRep(numeric_rep) => {
                let numeric_rep = self.walk_numeric_rep(visitor, &numeric_rep)?;
                stack.push(numeric_rep);
            }
            Primary::NumericExpressionOther(numeric_expression_other) => {
                let numeric_expression_other = self.walk_numeric_expression_other(
                    visitor,
                    &is_local_for_variable,
                    &numeric_expression_other,
                )?;
                stack.push((String::default(), numeric_expression_other));
            }
            Primary::NumericFunctionRef(numeric_function_ref) => {
                let numeric_function_ref = self.walk_numeric_function_ref(
                    visitor,
                    &is_local_for_variable,
                    &numeric_function_ref,
                )?;
                stack.push((String::default(), numeric_function_ref));
            }
            Primary::NumericVariable(numeric_variable) => {
                let (numeric_variable, numeric_variable_value) =
                    self.walk_numeric_variable(visitor, &is_local_for_variable, numeric_variable)?;
                stack.push((numeric_variable, numeric_variable_value));
            }
        }
        let primary = visitor.visit_primary(&mut stack)?;
        Ok(primary)
    }

    fn walk_numeric_expression_other(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NumericExpressionOther,
    ) -> Result<Value> {
        let (_, mut stack_value) = self.walk_numeric_expression(
            visitor,
            &is_local_for_variable,
            &node.numeric_expression,
        )?;
        let numeric_expression_other = visitor.visit_numeric_expression_other(&mut stack_value)?;
        Ok(numeric_expression_other)
    }

    fn walk_numeric_function_ref(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NumericFunctionRef,
    ) -> Result<Value> {
        let numeric_function_name =
            self.walk_numeric_function_name(visitor, &node.numeric_function_name)?;
        let mut numeric_expression = None::<(String, Value)>;
        if let Some(expr_node) = &node.numeric_expression {
            numeric_expression =
                Some(self.walk_numeric_expression(visitor, is_local_for_variable, expr_node)?);
        }
        let numeric_function_ref = visitor.visit_numeric_function_ref(
            &numeric_function_name,
            &numeric_expression,
            node,
        )?;
        Ok(numeric_function_ref)
    }

    fn walk_numeric_function_name(
        &mut self,
        visitor: &mut Visitor,
        node: &NumericFunctionName,
    ) -> Result<String> {
        let numeric_function_name_match = match node {
            NumericFunctionName::NumericDefinedFunction(numeric_defined_function) => {
                self.walk_numeric_defined_function(visitor, &numeric_defined_function)
            }
            NumericFunctionName::NumericSuppliedFunction(numeric_supplied_function) => {
                self.walk_numeric_supplied_function(visitor, &numeric_supplied_function)?
            }
        };
        let numeric_function_name =
            visitor.visit_numeric_function_name(&numeric_function_name_match)?;
        Ok(numeric_function_name)
    }

    fn walk_string_expressions(
        &mut self,
        visitor: &mut Visitor,
        node: &StringExpression,
    ) -> Result<(String, Value)> {
        let stack = match node {
            StringExpression::StringConstant(string_constant) => {
                visitor.visit_string_constant(string_constant)
            }
            StringExpression::StringVariable(string_variable) => {
                self.walk_string_variable(visitor, string_variable)?.0
            }
        };
        let string_expression = visitor.visit_string_expression(&stack, node)?;
        Ok(string_expression)
    }

    /*****************************************************************************
     *  9. IMPLEMENTATION SUPPLIED FUNCTIONS                                     *
     *****************************************************************************/
    fn walk_numeric_supplied_function(
        &mut self,
        visitor: &mut Visitor,
        node: &NumericSuppliedFunction,
    ) -> Result<String> {
        let stack = &node.numeric_supplied_function;
        let numeric_supplied_function = visitor.visit_numeric_supplied_function(&stack.clone())?;
        Ok(numeric_supplied_function)
    }

    /*****************************************************************************
     *  10. USER DEFINED FUNCTIONS                                               *
     *****************************************************************************/
    fn walk_def_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &DefStatement,
    ) -> Result<()> {
        let numeric_defined_function =
            self.walk_numeric_defined_function(visitor, &node.numeric_defined_function);
        let mut parameter_list = None::<String>;
        if let Some(params) = &node.parameter_list {
            parameter_list =
                Some(self.walk_parameter_list(visitor, &is_local_for_variable, params)?);
        }
        self.walk_numeric_expression(visitor, &is_local_for_variable, &node.numeric_expression)?;
        visitor.visit_def_statement(numeric_defined_function, parameter_list, node)?;
        Ok(())
    }

    fn walk_numeric_defined_function(
        &mut self,
        visitor: &mut Visitor,
        node: &NumericDefinedFunction,
    ) -> String {
        let numeric_defined_function = visitor.visit_numeric_defined_function(node);
        numeric_defined_function
    }

    fn walk_parameter_list(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &ParameterList,
    ) -> Result<String> {
        let stack = self.walk_simple_numeric_variable(
            visitor,
            &is_local_for_variable,
            &node.simple_numeric_variable,
        );
        let parameter_list = visitor.visit_parameter_list(&stack)?;
        Ok(parameter_list)
    }

    /*****************************************************************************
     *  11. LET STATEMENT                                                        *
     *****************************************************************************/
    fn walk_let_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &LetStatement,
    ) -> Result<()> {
        match node {
            LetStatement::NumericLetStatement(numeric_let_statement) => self
                .walk_numeric_let_statement(
                    visitor,
                    is_local_for_variable,
                    numeric_let_statement,
                )?,
            LetStatement::StringLetStatement(string_let_statement) => {
                self.walk_string_let_statement(visitor, string_let_statement)?
            }
        }
        visitor.visit_let_statement();
        Ok(())
    }

    fn walk_numeric_let_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NumericLetStatement,
    ) -> Result<()> {
        let (simple_numeric_variable, _) =
            self.walk_numeric_variable(visitor, is_local_for_variable, &node.numeric_variable)?;
        let (_, stack_value) =
            self.walk_numeric_expression(visitor, is_local_for_variable, &node.numeric_expression)?;
        visitor.visit_numeric_let_statement(&(simple_numeric_variable, stack_value))?;
        Ok(())
    }

    fn walk_string_let_statement(
        &mut self,
        visitor: &mut Visitor,
        node: &StringLetStatement,
    ) -> Result<()> {
        let (string_variable, _) = self.walk_string_variable(visitor, &node.string_variable)?;
        let (string_expression, string_expression_value) =
            self.walk_string_expressions(visitor, &node.string_expression)?;
        visitor.visit_string_let_statement(&(
            string_variable,
            string_expression,
            string_expression_value,
        ))?;
        Ok(())
    }

    /*****************************************************************************
     *  12. CONTROL STATEMENT                                                    *
     *****************************************************************************/
    fn walk_goto_statement(
        &mut self,
        visitor: &mut Visitor,
        node: &GotoStatement,
    ) -> Result<String> {
        let mut goto_line_number = self.walk_line_number(visitor, &node.line_number);
        goto_line_number = visitor.visit_goto_statement(&goto_line_number)?;
        Ok(goto_line_number)
    }
    fn walk_if_then_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &IfThenStatement,
    ) -> Result<()> {
        let relational_expression = self.walk_relational_expression(
            visitor,
            &is_local_for_variable,
            &node.relational_expression,
        )?;
        let line_number = self.walk_line_number(visitor, &node.line_number);
        visitor.visit_if_then_statement(&mut (relational_expression, line_number))?;
        Ok(())
    }

    fn walk_relational_expression(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &RelationalExpression,
    ) -> Result<Value> {
        let relational_expression = match node {
            RelationalExpression::NumericExpression((
                numeric_expression_item_a,
                numeric_expression_item_relation,
                numeric_expression_item_b,
            )) => {
                let (_, numeric_expression_a_value) = self.walk_numeric_expression(
                    visitor,
                    &is_local_for_variable,
                    &numeric_expression_item_a,
                )?;
                let numeric_expression_relation =
                    self.walk_relation(visitor, &numeric_expression_item_relation);
                let (_, numeric_expression_b_value) = self.walk_numeric_expression(
                    visitor,
                    &is_local_for_variable,
                    &numeric_expression_item_b,
                )?;
                (
                    numeric_expression_a_value,
                    numeric_expression_relation,
                    numeric_expression_b_value,
                    true,
                )
            }
            RelationalExpression::StrinExpression((
                string_expression_a,
                string_expression_relation,
                string_expression_b,
            )) => {
                let (_, string_expression_a_value) =
                    self.walk_string_expressions(visitor, &string_expression_a)?;
                let string_expression_relation =
                    self.walk_equality_relation(visitor, &string_expression_relation);
                let (_, string_expression_b_value) =
                    self.walk_string_expressions(visitor, &string_expression_b)?;
                (
                    string_expression_a_value,
                    string_expression_relation,
                    string_expression_b_value,
                    false,
                )
            }
        };
        let relational_expression_value =
            visitor.visit_relational_expression(&relational_expression)?;
        Ok(relational_expression_value)
    }
    fn walk_relation(&mut self, visitor: &mut Visitor, node: &Relation) -> String {
        let relation = match node {
            Relation::EqualityRelation(equality_relation) => {
                self.walk_equality_relation(visitor, &equality_relation)
            }
            Relation::EqualityRelationOther(equality_relation_other) => {
                equality_relation_other.trim().to_string()
            }
        };
        visitor.visit_relation();
        relation
    }
    fn walk_equality_relation(&mut self, visitor: &mut Visitor, node: &EqualityRelation) -> String {
        let equality_relation = node.equality_relation.trim().to_string();
        visitor.visit_equality_relation();
        equality_relation
    }

    fn walk_gosub_statement(
        &mut self,
        visitor: &mut Visitor,
        node: &GosubStatement,
    ) -> Result<String> {
        let mut gosub_line_number = self.walk_line_number(visitor, &node.line_number);
        gosub_line_number = visitor.visit_gosub_statement(&gosub_line_number)?;
        Ok(gosub_line_number)
    }

    fn walk_return_statement(&mut self, visitor: &mut Visitor) -> Result<()> {
        visitor.visit_return_statement()?;
        Ok(())
    }

    fn walk_on_goto_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &OnGotoStatement,
    ) -> Result<()> {
        let (_, numeric_expression_value) =
            self.walk_numeric_expression(visitor, is_local_for_variable, &node.numeric_expression)?;
        let mut line_numbers = Vec::new();
        node.line_number.iter().for_each(|numeric_expression_item| {
            let numeric_expression_string = self.walk_line_number(visitor, numeric_expression_item);
            line_numbers.push(numeric_expression_string);
        });
        visitor.visit_on_goto_statement(&(numeric_expression_value, line_numbers))?;
        Ok(())
    }

    fn walk_stop_statement(&mut self, visitor: &mut Visitor) {
        visitor.visit_stop_statement();
    }

    /*****************************************************************************
     *  13. FOR AND NEXT STATEMENTS                                              *
     *****************************************************************************/
    fn walk_for_block(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &ForBlock,
    ) -> Result<()> {
        self.walk_for_line(visitor, &is_local_for_variable, &node.for_line)?;
        self.walk_for_body(visitor, &is_local_for_variable, &node.for_body)?;
        visitor.visit_for_block()?;
        Ok(())
    }

    fn walk_for_body(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &ForBody,
    ) -> Result<()> {
        node.block
            .iter()
            .try_for_each(|block_item| self.walk_block(visitor, &true, block_item))?;
        self.walk_next_line(visitor, &is_local_for_variable, &node.next_line)?;
        visitor.visit_for_body();
        Ok(())
    }

    fn walk_for_line(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &ForLine,
    ) -> Result<()> {
        let line_number = self.walk_line_number(visitor, &node.line_number);
        visitor.visit_pre_line(&line_number)?;
        let (control_variable, initial_value, limit, increment) =
            self.walk_for_statement(visitor, &is_local_for_variable, &node.for_statement)?;
        visitor.visit_for_line(&(
            line_number,
            control_variable,
            initial_value,
            limit,
            increment,
        ))?;
        Ok(())
    }

    fn walk_next_line(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NextLine,
    ) -> Result<()> {
        let line_number = self.walk_line_number(visitor, &node.line_number);
        visitor.visit_pre_line(&line_number)?;
        let next_statement =
            self.walk_next_statement(visitor, &is_local_for_variable, &node.next_statement)?;
        visitor.visit_next_line(&next_statement)?;
        Ok(())
    }

    fn walk_for_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &ForStatement,
    ) -> Result<(String, Value, Value, Option<Value>)> {
        let control_variable =
            self.walk_control_variable(visitor, &is_local_for_variable, &node.control_variable)?;
        let initial_value =
            self.walk_initial_value(visitor, &is_local_for_variable, &node.initial_value)?;
        let limit = self.walk_limit(visitor, &is_local_for_variable, &node.limit)?;
        let increment1 = node.increment.as_ref();
        let increment = increment1
            .map(|inc_node| self.walk_increment(visitor, &is_local_for_variable, inc_node))
            .transpose()?;
        let for_statement = visitor.visit_for_statement(&(
            control_variable,
            initial_value.clone(),
            limit.clone(),
            increment.clone(),
        ))?;
        Ok(for_statement)
    }
    fn walk_control_variable(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &ControlVariable,
    ) -> Result<String> {
        let simple_numeric_variable = self.walk_simple_numeric_variable(
            visitor,
            is_local_for_variable,
            &node.control_variable,
        );
        let control_variable = visitor.visit_control_variable(&simple_numeric_variable)?;
        Ok(control_variable)
    }
    fn walk_initial_value(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &InitialValue,
    ) -> Result<Value> {
        let (_, numeric_expression_value) =
            self.walk_numeric_expression(visitor, &is_local_for_variable, &node.initial_value)?;
        let initial_value = visitor.visit_initial_value(&numeric_expression_value)?;
        Ok(initial_value)
    }
    fn walk_limit(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Limit,
    ) -> Result<Value> {
        let (_, numeric_expression_value) =
            self.walk_numeric_expression(visitor, &is_local_for_variable, &node.limit)?;
        let limit_value = visitor.visit_limit(&numeric_expression_value)?;
        Ok(limit_value)
    }
    fn walk_increment(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Increment,
    ) -> Result<Value> {
        let (_, numeric_expression_value) =
            self.walk_numeric_expression(visitor, &is_local_for_variable, &node.increment)?;
        let increment_value = visitor.visit_increment(&numeric_expression_value)?;
        Ok(increment_value)
    }
    fn walk_next_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NextStatement,
    ) -> Result<String> {
        let control_variable_string =
            self.walk_control_variable(visitor, &is_local_for_variable, &node.control_variable)?;
        let next_statement_string = visitor.visit_next_statement(&control_variable_string)?;
        Ok(next_statement_string)
    }

    /*****************************************************************************
     *  14. PRINT STATEMENT                                                      *
     *****************************************************************************/
    fn walk_print_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &PrintStatement,
    ) -> Result<()> {
        let stack = self.walk_print_list(visitor, is_local_for_variable, &node.print_list)?;
        visitor.visit_print_statement(&stack)?;
        Ok(())
    }

    fn walk_print_list(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &PrintList,
    ) -> Result<Vec<(Option<(Value, bool)>, Option<String>)>> {
        let mut stack = Vec::new();
        for (print_item_option, print_separator) in &node.print_item_separator {
            let print_item =
                self.walk_print_item(visitor, is_local_for_variable, print_item_option)?;
            let print_separator = self.walk_print_separator(visitor, &print_separator);
            stack.push((print_item, Some(print_separator)));
        }
        let print_item = self.walk_print_item(visitor, is_local_for_variable, &node.print_item)?;
        if print_item.is_some() {
            stack.push((print_item, None));
        }
        let print_list = visitor.visit_print_list(&stack)?;
        Ok(print_list)
    }

    fn walk_print_separator(&mut self, visitor: &mut Visitor, node: &PrintSeparator) -> String {
        let print_separator = visitor.visit_print_separator(&node);
        print_separator
    }

    fn walk_print_item(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Option<PrintItem>,
    ) -> Result<Option<(Value, bool)>> {
        let stack = match node.as_ref() {
            None => None,
            Some(PrintItem::Expression(expression)) => Some((
                self.walk_expression(visitor, &is_local_for_variable, expression)?,
                false,
            )),
            Some(PrintItem::TabCall(tab_call)) => Some((
                self.walk_tab_call(visitor, &is_local_for_variable, tab_call)?,
                true,
            )),
        };
        let print_item = visitor.visit_print_item(&stack)?;
        Ok(print_item)
    }

    fn walk_tab_call(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &TabCall,
    ) -> Result<Value> {
        let stack = self.walk_numeric_expression(
            visitor,
            &is_local_for_variable,
            &node.numeric_expression,
        )?;
        let tab_call = visitor.visit_tab_call(&stack.1)?;
        Ok(tab_call)
    }

    /*****************************************************************************
     *  15. INPUT STATEMENT                                                      *
     *****************************************************************************/
    fn walk_input_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &InputStatement,
    ) -> Result<()> {
        let mut stack = Vec::new();
        node.variable.iter().try_for_each(|variable_item| {
            let variable = self.walk_variable(visitor, is_local_for_variable, &variable_item)?;
            stack.push(variable);
            anyhow::Ok(())
        })?;
        visitor.visit_input_statement(&stack)?;
        Ok(())
    }

    /*****************************************************************************
     *  16. READ AND RESTORE STATEMENTS                                          *
     *****************************************************************************/
    fn walk_read_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &ReadStatement,
    ) -> Result<()> {
        let mut stack = Vec::new();
        node.variable_list
            .iter()
            .try_for_each(|variable_list_item| {
                let variable =
                    self.walk_variable(visitor, &is_local_for_variable, variable_list_item)?;
                stack.push(variable);
                anyhow::Ok(())
            })?;
        visitor.visit_read_statement(&stack)?;
        Ok(())
    }

    fn walk_restore_statement(&mut self, visitor: &mut Visitor) {
        visitor.visit_restore_statement();
    }

    /*****************************************************************************
     *  17. DATA STATEMENT                                                       *
     *****************************************************************************/
    fn walk_data_statement(&mut self) {}

    /*****************************************************************************
     *  18. ARRAY DECLARATIONS                                                   *
     *****************************************************************************/
    fn walk_dimension_statement(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &DimensionStatement,
    ) -> Result<()> {
        let mut stack = Vec::new();
        node.array_declaration
            .iter()
            .try_for_each(|array_declaration_item| {
                let array_declaration = self.walk_array_declaration(
                    visitor,
                    &is_local_for_variable,
                    array_declaration_item,
                )?;
                stack.push(array_declaration);
                anyhow::Ok(())
            })?;
        visitor.visit_dimension_statement(&stack)?;
        Ok(())
    }

    fn walk_array_declaration(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &ArrayDeclaration,
    ) -> Result<(String, Vec<(String, Value)>)> {
        let numeric_array_name =
            self.walk_numeric_array_name(visitor, &is_local_for_variable, &node.numeric_array_name);
        let bounds = self.walk_bounds(visitor, &node.bound)?;
        Ok(visitor.visit_array_declaration(&(numeric_array_name, bounds))?)
    }

    fn walk_bounds(
        &mut self,
        visitor: &mut Visitor,
        node: &Bounds,
    ) -> Result<Vec<(String, Value)>> {
        let mut stack = Vec::new();
        node.integer.iter().try_for_each(|integer_item| {
            let integer = self.walk_integer(visitor, integer_item)?;
            stack.push(integer);
            anyhow::Ok(())
        })?;
        Ok(visitor.visit_bounds(&stack)?)
    }

    fn walk_option_statement(
        &mut self,
        visitor: &mut Visitor,
        node: &OptionStatement,
    ) -> Result<()> {
        let stack = self.walk_option_base(visitor, &node.option_base)?;
        visitor.visit_option_statement(&stack)?;
        Ok(())
    }

    fn walk_option_base(&mut self, visitor: &mut Visitor, node: &OptionBase) -> Result<u8> {
        let option_base = visitor.visit_option_base(&node)?;
        Ok(option_base)
    }

    /*****************************************************************************
     *  19. REMARK STATEMENT                                                     *
     *****************************************************************************/
    fn walk_remark_statement(&mut self, visitor: &mut Visitor) {
        visitor.visit_remark_statement();
    }

    /*****************************************************************************
     *  20. RANDOMIZE STATEMENT                                                  *
     *****************************************************************************/
    fn walk_randomize_statement(&mut self, visitor: &mut Visitor) -> Result<()> {
        visitor.visit_randomize_statement()?;
        Ok(())
    }

    /*****************************************************************************
     *  TABLE 1                                                                  *
     *****************************************************************************/
    fn walk_sign_term(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Vec<(Sign, Term)>,
    ) -> Result<Vec<Value>> {
        let mut stack = Vec::new();
        node.iter().try_for_each(|(sign, term)| {
            let sign = visitor.visit_sign(&sign)?;
            stack.push(sign);
            let (_, term_value) = self.walk_term(visitor, &is_local_for_variable, term)?;
            stack.push(term_value);
            anyhow::Ok(())
        })?;
        Ok(stack)
    }
}
