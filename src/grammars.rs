//! # Grammar Rules.
//!
//! This module declares the Basic and BasicTreeMap grammar rules.
//!
pub mod basic_pest {
    use pest_derive::Parser;

    /// # pub struct `MinBas78`.
    ///
    /// Struct for the Minimal BASIC 78 grammar rules.
    ///
    #[derive(Parser)]
    #[grammar = "../pest/minbas78.pest"]
    pub struct MinBas78;
}

pub mod basic_treemap_pest {
    use pest_derive::Parser;

    /// # pub struct `MinBas78UnorderedLines`.
    ///
    /// Struct for the unordered lines of the Minimal BASIC 78 map grammar rules.
    ///
    #[derive(Parser)]
    #[grammar = "../pest/minbas78_unordered_lines.pest"]
    pub struct MinBas78UnorderedLines;
}
