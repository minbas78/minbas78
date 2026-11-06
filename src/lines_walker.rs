//! # Abstract Syntax Tree Walker Lines.
//!
//! This module declares the public abstract syntax tree lines walker.
//!
use crate::{
    abstract_syntax_tree::{
        self, Block, DataList, DataStatement, Datum, EndLine, ForBlock, ForBody, ForLine, Line,
        LineNumber, NextLine, Program, Statement,
    },
    visitor::Visitor,
};
use anyhow::{Ok, Result};

/// # pub struct `BasicAbstractSyntaxTreeLineWalker`.
///
/// Struct for the walks methods.
///
/// _This structs holds the walks methods for the abstract syntaxes tree
/// of the lines._
///
pub struct BasicAbstractSyntaxTreeLineWalker;

impl BasicAbstractSyntaxTreeLineWalker {
    /// # pub fn `walk_program`.
    ///
    /// Walker for the `program` grammar rule.
    ///
    /// _This function walks the `program` grammar rule
    /// and all it dependendant rules._
    ///
    pub fn walk_program(&mut self, visitor: &mut Visitor, node: &Program) -> Result<()> {
        node.block.iter().try_for_each(|block| {
            self.walk_block(visitor, &block)?;
            Ok(())
        })?;
        self.walk_end_line(visitor, &node.end_line);
        visitor.visit_program_de_walker_line()?;
        Ok(())
    }

    fn walk_block(&mut self, visitor: &mut Visitor, node: &Block) -> Result<()> {
        match node {
            abstract_syntax_tree::Block::Line(line) => self.walk_line(visitor, &line)?,
            abstract_syntax_tree::Block::ForBlock(for_block) => {
                self.walk_for_block(visitor, &for_block)?
            }
        };
        Ok(())
    }

    fn walk_line(&mut self, visitor: &mut Visitor, node: &Line) -> Result<()> {
        let line_number = self.walk_line_number(&node.line_number);
        self.walk_statement(visitor, &node.statement)?;
        visitor.visit_make_block_of_line(&line_number);
        Ok(())
    }

    fn walk_statement(&mut self, visitor: &mut Visitor, node: &Statement) -> Result<()> {
        match node {
            abstract_syntax_tree::Statement::DataStatement(s) => {
                self.walk_data_statement(visitor, s)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn walk_data_statement(&mut self, visitor: &mut Visitor, node: &DataStatement) -> Result<()> {
        self.walk_data_list(visitor, &node.data_list)?;
        visitor.visit_data_statement();
        Ok(())
    }

    fn walk_data_list(&mut self, visitor: &mut Visitor, node: &DataList) -> Result<()> {
        let mut stack = Vec::new();
        node.datums.iter().try_for_each(|d| {
            let datum = self.walk_datum(visitor, &d)?;
            stack.push(datum);
            anyhow::Ok(())
        })?;
        visitor.visit_data_list(&mut stack)?;
        Ok(())
    }

    fn walk_datum(&mut self, visitor: &mut Visitor, node: &Datum) -> Result<String> {
        let stack = node.datum.trim().to_string();
        let datum = visitor.visit_datum(&stack)?;
        Ok(datum)
    }

    fn walk_end_line(&mut self, visitor: &mut Visitor, node: &EndLine) {
        let line_number = self.walk_line_number(&node.line_number);
        visitor.visit_make_block_of_line(&line_number);
    }

    fn walk_line_number(&mut self, node: &LineNumber) -> String {
        let line_number = node.line_number.trim().to_string();
        line_number
    }

    fn walk_for_block(&mut self, visitor: &mut Visitor, node: &ForBlock) -> Result<()> {
        self.walk_for_line(visitor, &node.for_line);
        self.walk_for_body(visitor, &node.for_body)?;
        Ok(())
    }

    fn walk_for_line(&mut self, visitor: &mut Visitor, node: &ForLine) {
        let line_number = self.walk_line_number(&node.line_number);
        visitor.visit_make_block_of_line(&line_number);
    }

    fn walk_for_body(&mut self, visitor: &mut Visitor, node: &ForBody) -> Result<()> {
        node.block.iter().try_for_each(|b| {
            self.walk_block(visitor, b)?;
            Ok(())
        })?;
        self.walk_next_line(visitor, &node.next_line);
        Ok(())
    }

    fn walk_next_line(&mut self, visitor: &mut Visitor, node: &NextLine) {
        let line_number = self.walk_line_number(&node.line_number);
        visitor.visit_make_block_of_line(&line_number);
    }
}
