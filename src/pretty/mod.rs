//! Pretty-printing harness.
//!
//! A small `Doc` IR with explicit trivia slots that the renderer resolves
//! against a [`crate::attach::CommentMap`]. User code implements [`Format`]
//! per AST node and never has to think about comments directly.
//!
//! The combinator surface mirrors Wadler / Leijen `prettyprinter`:
//! primitives (`text`, `line`, `softline`, `hardline`), layout (`indent`,
//! `align`, `hang`, `group`, `flat_alt`), enclosers (`parens`, `brackets`,
//! `braces`, `angles`, `dquotes`, `squotes`, `enclose`, `enclose_sep`),
//! list combinators (`hcat`, `hsep`, `vcat`, `vsep`, `sep`, `cat`,
//! `punctuate`, `list`, `tupled`), and shortcut methods on `Doc` for the
//! `<+>` / `</>` / `<$>` / `<//>` join operators.

mod doc;
mod format;
mod render;

pub use doc::{
    align, angles, braces, brackets, cat, char, colon, comma, concat, dot, dquote, dquotes,
    enclose, enclose_sep, equals, flat_alt, group, hang, hardline, hcat, hsep, indent, langle,
    lbrace, lbracket, line, list, lparen, nil, parens, punctuate, rangle, rbrace, rbracket, rparen,
    semi, sep, softline, space, squote, squotes, text, trivia, tupled, vcat, vsep, Doc, Side,
    TriviaSlot,
};
pub use format::{with_trivia, Format};
pub use render::{render, RenderOpts};
