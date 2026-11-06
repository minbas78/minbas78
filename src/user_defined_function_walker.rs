//! # Abstract Syntax Tree Walker for the User Defined Functions.
//!
//! This module declares the public abstract syntax tree lines walker for the user defined functions.
//!
use crate::{
    abstract_syntax_tree::{
        self, Exrad, Factor, NumericArrayElement, NumericArrayName, NumericDefinedFunction,
        NumericExpression, NumericExpressionOther, NumericFunctionName, NumericFunctionRef,
        NumericRep, NumericSuppliedFunction, NumericVariable, Primary, Significand,
        SimpleNumericVariable, Term,
    },
    visitor::Visitor,
};
use anyhow::Result;
use cranelift_codegen::ir::Value;

/// # pub struct `BasicAbstractSyntaxTreeUserDefinedFunctionWalker`.
///
/// Struct for the walks methods.
///
/// _This structs holds the walks methods for the abstract syntaxes tree
/// of the user defined functions._
///
pub struct BasicAbstractSyntaxTreeUserDefinedFunctionWalker;

impl BasicAbstractSyntaxTreeUserDefinedFunctionWalker {
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

    fn walk_exrad(&mut self, visitor: &mut Visitor, node: &Exrad) -> Result<Value> {
        let mut stack = Vec::new();
        if let Some(sign_node) = &node.sign {
            let sign = visitor.visit_sign(sign_node)?;
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
    fn walk_numeric_variable(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NumericVariable,
    ) -> Result<(String, Value)> {
        let stack = match node {
            abstract_syntax_tree::NumericVariable::SimpleNumericVariable(
                simple_numeric_variable,
            ) => self.walk_simple_numeric_variable(
                visitor,
                &is_local_for_variable,
                &simple_numeric_variable,
            ),
            abstract_syntax_tree::NumericVariable::NumericArrayElement(numeric_array_element) => {
                self.walk_numeric_array_element(
                    visitor,
                    &is_local_for_variable,
                    numeric_array_element,
                )?
            }
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

    /*****************************************************************************
     *  8. EXPRESSIONS                                                           *
     *****************************************************************************/
    /// # pub fn `walk_numeric_expression`.
    ///
    /// Walker for the `numeric_expression` grammar rule.
    ///
    /// _This function walks the `numeric_expression` grammar rule
    /// and all it dependendant rules._
    ///
    pub fn walk_numeric_expression(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &NumericExpression,
    ) -> Result<(String, Value)> {
        let mut stack = Vec::new();
        if let Some(sign) = node
            .sign
            .clone()
            .map(|sign_item| visitor.visit_sign(&sign_item))
            .transpose()?
        {
            stack.push(sign);
        }
        let (term_string, term_value) =
            self.walk_term(visitor, &is_local_for_variable, &node.term)?;
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
        node: &Vec<(
            abstract_syntax_tree::Multiplier,
            abstract_syntax_tree::Factor,
        )>,
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
        node: &Box<abstract_syntax_tree::Primary>,
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
            abstract_syntax_tree::Primary::NumericRep(numeric_rep) => {
                let numeric_rep = self.walk_numeric_rep(visitor, &numeric_rep)?;
                stack.push(numeric_rep);
            }
            abstract_syntax_tree::Primary::NumericExpressionOther(numeric_expression_other) => {
                let numeric_expression_other = self.walk_numeric_expression_other(
                    visitor,
                    &is_local_for_variable,
                    &numeric_expression_other,
                )?;
                stack.push((String::default(), numeric_expression_other));
            }
            abstract_syntax_tree::Primary::NumericFunctionRef(numeric_function_ref) => {
                let numeric_function_ref = self.walk_numeric_function_ref(
                    visitor,
                    &is_local_for_variable,
                    &numeric_function_ref,
                )?;
                stack.push((String::default(), numeric_function_ref));
            }
            abstract_syntax_tree::Primary::NumericVariable(numeric_variable) => {
                let (_, numeric_variable) =
                    self.walk_numeric_variable(visitor, &is_local_for_variable, numeric_variable)?;
                stack.push((String::default(), numeric_variable));
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
        let numeric_expression1 = node.numeric_expression.as_ref();
        let numeric_expression = numeric_expression1
            .map(|numeric_expression_item| {
                self.walk_numeric_expression(
                    visitor,
                    is_local_for_variable,
                    numeric_expression_item,
                )
            })
            .transpose()?;
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
        let stack = match node {
            abstract_syntax_tree::NumericFunctionName::NumericDefinedFunction(
                numeric_defined_function,
            ) => self.walk_numeric_defined_function(visitor, &numeric_defined_function),
            abstract_syntax_tree::NumericFunctionName::NumericSuppliedFunction(
                numeric_supplied_function,
            ) => self.walk_numeric_supplied_function(visitor, &numeric_supplied_function)?,
        };
        let numeric_function_name = visitor.visit_numeric_function_name(&stack)?;
        Ok(numeric_function_name)
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
    fn walk_numeric_defined_function(
        &mut self,
        visitor: &mut Visitor,
        node: &NumericDefinedFunction,
    ) -> String {
        let numeric_defined_function = visitor.visit_numeric_defined_function(node);
        numeric_defined_function
    }

    /*****************************************************************************
     *  TABLE 1                                                                  *
     *****************************************************************************/
    fn walk_sign_term(
        &mut self,
        visitor: &mut Visitor,
        is_local_for_variable: &bool,
        node: &Vec<(abstract_syntax_tree::Sign, abstract_syntax_tree::Term)>,
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
