//! Trivia-preserving lexer adapter for `logos` + `lalrpop` pipelines.
//!
//! See the [workspace README](https://github.com/sdiehl/marginalia) and the
//! `calc` example for end-to-end integration.

mod classify;
mod lexer;
mod span;
mod table;
mod trivia;

pub use classify::{Classify, TriviaPiece};
pub use lexer::TriviaLexer;
pub use span::{span, Span};
pub use table::{TriviaEvent, TriviaTable};
pub use trivia::{Trivia, TriviaKind};
