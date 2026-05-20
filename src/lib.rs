//! Trivia-preserving parsing and formatting for `logos` + `lalrpop` pipelines.
//!
//! - [`TriviaLexer`] wraps any `Iterator<Item = Result<(usize, Tok, usize),
//!   E>>` and records comments/blank lines on the side while the parser sees
//!   only semantic tokens.
//! - [`attach`] places those trivia events on AST node spans as leading,
//!   trailing, or dangling comments.
//! - [`pretty`] is a small `Doc` IR with explicit trivia slots that the
//!   renderer resolves against a `CommentMap`.
//!
//! See the `calc` example for an end-to-end integration.

mod classify;
mod lexer;
mod span;
mod table;
mod trivia;

pub mod attach;
pub mod pretty;

pub use classify::{Classify, TriviaPiece};
pub use lexer::TriviaLexer;
pub use span::{span, Span};
pub use table::{TriviaEvent, TriviaTable};
pub use trivia::{Trivia, TriviaKind};
