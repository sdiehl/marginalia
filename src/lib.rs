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
//! Trivia is generic over a kind enum `K` so downstream crates can carry a
//! richer classification through every layer. The default `K = BuiltinKind`
//! covers the common line/block distinction without any extra wiring.
//!
//! The layers in miniature, with the spans written out by hand where a real
//! integration would get them from the lexer and the AST:
//!
//! ```
//! use marginalia::{
//!     attach::{attach, AttachOptions},
//!     pretty::{render, text, with_trivia, RenderOpts},
//!     span, Trivia, TriviaEvent, TriviaTable,
//! };
//!
//! let source = "a; // note\nb;";
//! let (a, b) = (span(0, 2), span(11, 13));
//!
//! // A `TriviaLexer` fills this while the parser runs.
//! let table: TriviaTable = [TriviaEvent {
//!     span: span(3, 10),
//!     trivia: Trivia::line("// note"),
//! }]
//! .into_iter()
//! .collect();
//!
//! let map = attach(source, &table, [a, b], AttachOptions::default());
//! let doc = with_trivia(a, text("a;")).hardline(with_trivia(b, text("b;")));
//!
//! assert_eq!(render(&doc, &map, RenderOpts::default()), "a; // note\nb;");
//! ```
//!
//! See the `calc` example for an end-to-end integration.

#![warn(missing_docs)]

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
pub use trivia::{BuiltinKind, Trivia, TriviaClass};

/// Doctests the README so its integration sketch cannot drift from the API.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct Readme;
