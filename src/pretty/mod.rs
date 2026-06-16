//! Pretty-printing harness.
//!
//! A small `Doc` IR with explicit trivia slots that the renderer resolves
//! against a [`crate::attach::CommentMap`]. User code implements [`Format`]
//! per AST node and never has to think about comments directly.
//!
//! The combinator surface mirrors Wadler / Leijen `prettyprinter`:
//! primitives (`text`, `line`, `softline`, `hardline`), layout (`indent`,
//! `align`, `hang`, `group`, `flat_alt`, `flatten`), enclosers (`parens`,
//! `brackets`, `braces`, `angles`, `dquotes`, `squotes`, `enclose`,
//! `enclose_sep`), list combinators (`hcat`, `hsep`, `vcat`, `vsep`, `sep`,
//! `cat`, `punctuate`, `list`, `tupled`, `block` / [`Block`]), and shortcut
//! methods on `Doc` for the `<+>` / `</>` / `<$>` / `<//>` join operators.
//!
//! Render with [`render`] (full control over trivia and options) or the
//! [`pretty`] / [`pretty_flat`] shortcuts for the trivia-free case.

mod doc;
mod format;
mod render;

pub use doc::{
    align, angles, block, braces, brackets, cat, char, colon, comma, concat, dot, dquote, dquotes,
    enclose, enclose_sep, equals, flat_alt, flatten, group, hang, hardline, hcat, hsep, indent,
    langle, lbrace, lbracket, line, list, lparen, nil, parens, punctuate, rangle, rbrace, rbracket,
    rparen, semi, sep, softline, space, squote, squotes, text, trivia, tupled, vcat, vsep, Block,
    Doc, Side, TriviaSlot,
};
pub use format::{with_trivia, Format};
pub use render::{pretty, pretty_at, pretty_flat, render, RenderOpts};
