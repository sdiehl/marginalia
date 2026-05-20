//! Comment attachment.
//!
//! Given the `TriviaEvent`s from a [`crate::TriviaLexer`] and the spans of
//! "interesting" AST nodes, decide which comments are *leading*, *trailing*,
//! or *dangling* for each node.

mod attacher;
mod map;

pub use attacher::{attach, AttachOptions};
pub use map::{CommentMap, Comments, HasSpan};
