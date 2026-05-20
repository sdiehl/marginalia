//! Pretty-printing harness for `marginalia` pipelines.
//!
//! A small Wadler-style `Doc` IR with explicit trivia slots that the renderer
//! resolves against a `CommentMap`. User code implements `Format` per AST
//! node and never has to think about comments directly.

mod doc;
mod format;
mod render;

pub use doc::{
    concat, group, hardline, indent, line, nil, softline, text, trivia, Doc, Side, TriviaSlot,
};
pub use format::{with_trivia, Format};
pub use render::{render, RenderOpts};
