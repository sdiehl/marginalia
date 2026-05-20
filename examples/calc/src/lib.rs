//! Tiny calculator language with trivia-preserving formatting.

use lalrpop_util::lalrpop_mod;
use thiserror::Error;
use marginalia::{TriviaLexer, TriviaTable};
use marginalia_attach::{attach, AttachOptions};
use marginalia_pretty::{render, Format, RenderOpts};

pub mod ast;
pub mod fmt;
pub mod lexer;

lalrpop_mod!(
    #[allow(
        clippy::all,
        clippy::pedantic,
        clippy::unwrap_used,
        clippy::panic,
        unused_imports,
        dead_code,
        unreachable_pub,
        missing_debug_implementations
    )]
    parser
);

pub use parser::ProgramParser;

use crate::{
    ast::{collect_spans, Program},
    lexer::LexicalError,
};

#[derive(Debug, Error)]
pub enum CalcError {
    #[error("lex error: {0}")]
    Lex(#[from] LexicalError),
    #[error("parse error: {0}")]
    Parse(String),
}

pub fn parse(source: &str) -> Result<(Program, TriviaTable), CalcError> {
    let logos_lex = lexer::raw_lexer(source);
    let mut lex = TriviaLexer::new(logos_lex, source);
    let program = ProgramParser::new()
        .parse(&mut lex)
        .map_err(|e| CalcError::Parse(e.to_string()))?;
    Ok((program, lex.into_table()))
}

pub fn format_source(source: &str) -> Result<String, CalcError> {
    let (program, table) = parse(source)?;
    let spans = collect_spans(&program);
    let map = attach(source, &table, spans, AttachOptions::default());
    let doc = program.doc();
    let mut out = render(&doc, &map, RenderOpts::default());
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
