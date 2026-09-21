use std::ops::Add;

use crate::Span;

/// Which side of an anchor span a [`TriviaSlot`] fills.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    /// Comments rendered before the anchor, each on its own line.
    Leading,
    /// Comments rendered after the anchor, on the same line.
    Trailing,
}

/// A placeholder in a [`Doc`] that the renderer fills from a
/// [`CommentMap`](crate::attach::CommentMap).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TriviaSlot {
    /// The anchor span to look up.
    pub span: Span,
    /// Which of the anchor's two comment slots to emit.
    pub side: Side,
}

/// The document IR.
///
/// A `Doc` describes layout intent, not output: `Line` and `SoftLine` become
/// spaces or newlines depending on whether the enclosing [`Group`](Doc::Group)
/// fits, and [`Trivia`](Doc::Trivia) slots are resolved at render time. Build
/// one with the combinators in this module rather than the variants directly.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Doc {
    /// The empty document.
    #[default]
    Nil,
    /// Literal text, which must not contain newlines.
    Text(String),
    /// A space when flat, a newline when broken.
    Line,
    /// Nothing when flat, a newline when broken.
    SoftLine,
    /// Always a newline, and it disqualifies any enclosing group from staying
    /// flat.
    HardLine,
    /// Shift the indent of broken lines inside by `n` columns.
    Indent(isize, Box<Doc>),
    /// Indent broken lines inside to the current column.
    Align(Box<Doc>),
    /// Render the first alternative when flat, the second when broken.
    FlatAlt(Box<Doc>, Box<Doc>),
    /// Render flat if the contents fit on the current line, broken otherwise.
    Group(Box<Doc>),
    /// Documents rendered one after another.
    Concat(Vec<Doc>),
    /// Documents packed onto as many lines as they need, separated by a space
    /// where the next one still fits and a newline where it does not.
    Fill(Vec<Doc>),
    /// A slot filled from the comment map at render time.
    Trivia(TriviaSlot),
}

/// The empty document.
#[must_use]
pub const fn nil() -> Doc {
    Doc::Nil
}

/// Literal text. Must not contain newlines: use [`hardline`] for those, so the
/// renderer can track the column.
#[must_use]
pub fn text(s: impl Into<String>) -> Doc {
    Doc::Text(s.into())
}

/// A single character as text.
#[must_use]
pub fn char(c: char) -> Doc {
    Doc::Text(c.to_string())
}

/// A space when flat, a newline when broken.
#[must_use]
pub const fn line() -> Doc {
    Doc::Line
}

/// Nothing when flat, a newline when broken.
#[must_use]
pub const fn softline() -> Doc {
    Doc::SoftLine
}

/// An unconditional newline, which also forces any enclosing group to break.
#[must_use]
pub const fn hardline() -> Doc {
    Doc::HardLine
}

/// Shift broken lines inside `d` by `n` columns. Negative values dedent.
#[must_use]
pub fn indent(n: isize, d: Doc) -> Doc {
    Doc::Indent(n, Box::new(d))
}

/// Indent broken lines inside `d` to the column where `d` starts.
#[must_use]
pub fn align(d: Doc) -> Doc {
    Doc::Align(Box::new(d))
}

/// [`align`] plus [`indent`]: continuation lines land `n` past the start
/// column.
#[must_use]
pub fn hang(n: isize, d: Doc) -> Doc {
    align(indent(n, d))
}

/// Pick `flat` when the enclosing group fits on one line, `broken` otherwise.
#[must_use]
pub fn flat_alt(flat: Doc, broken: Doc) -> Doc {
    Doc::FlatAlt(Box::new(flat), Box::new(broken))
}

/// Render `d` on one line if it fits, otherwise break every `line` and
/// `softline` directly inside it.
#[must_use]
pub fn group(d: Doc) -> Doc {
    Doc::Group(Box::new(d))
}

/// Concatenate documents with nothing between them.
#[must_use]
pub fn concat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    Doc::Concat(parts.into_iter().collect())
}

/// A slot the renderer fills with `span`'s comments on `side`. Prefer
/// [`with_trivia`](super::with_trivia), which brackets a body with both.
#[must_use]
pub const fn trivia(span: Span, side: Side) -> Doc {
    Doc::Trivia(TriviaSlot { span, side })
}

macro_rules! literals {
    ($($name:ident => $lit:literal),* $(,)?) => {
        $(
            #[doc = concat!("The literal `", $lit, "`.")]
            #[must_use]
            pub fn $name() -> Doc {
                Doc::Text($lit.to_owned())
            }
        )*
    };
}

literals! {
    space => " ",
    comma => ",",
    semi => ";",
    colon => ":",
    dot => ".",
    equals => "=",
    lparen => "(",
    rparen => ")",
    lbracket => "[",
    rbracket => "]",
    lbrace => "{",
    rbrace => "}",
    langle => "<",
    rangle => ">",
    dquote => "\"",
    squote => "'",
}

/// Concatenate with nothing between. Alias for [`concat()`].
#[must_use]
pub fn hcat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    concat(parts)
}

/// Concatenate with a space between each.
#[must_use]
pub fn hsep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    interleave(parts, space)
}

/// Concatenate with a [`hardline`] between each.
#[must_use]
pub fn vcat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    interleave(parts, hardline)
}

/// Concatenate with a [`line()`] between each.
#[must_use]
pub fn vsep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    interleave(parts, line)
}

/// [`vsep`] in a [`group`]: spaces on one line if it fits, one item per line
/// otherwise.
#[must_use]
pub fn sep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    group(vsep(parts))
}

/// [`softline`]-separated in a [`group`]: run together on one line if it fits,
/// one item per line otherwise.
#[must_use]
pub fn cat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    group(interleave(parts, softline))
}

/// Pack the items onto as few lines as possible: a space before each one that
/// still fits on the current line, a newline before each one that does not.
///
/// The difference from [`sep`] is per-item rather than all-or-nothing. A list
/// of short items that overflows one line becomes a few full lines here, where
/// `sep` would give each item a line of its own. That suits homogeneous runs
/// (enum cases, numeric tables, a long chain of small arguments) and suits
/// structured items badly: an item that breaks internally leaves the next one
/// packed against its last line.
///
/// Separators belong on the items, via [`punctuate_end`]: a fill places each
/// item on its own, so a separator passed as an item of its own would be free
/// to start a line.
#[must_use]
pub fn fill_sep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    Doc::Fill(parts.into_iter().collect())
}

/// Interpose `sep` between the items, leaving the result unconcatenated so the
/// caller can still lay it out.
#[must_use]
pub fn punctuate<I: IntoIterator<Item = Doc>>(sep: &Doc, parts: I) -> Vec<Doc> {
    let mut parts = parts.into_iter();
    let mut out = Vec::with_capacity(parts.size_hint().0.saturating_mul(2));
    out.extend(parts.next());
    for part in parts {
        out.push(sep.clone());
        out.push(part);
    }
    out
}

/// Append `sep` to every item but the last, leaving the result
/// unconcatenated. The counterpart of [`punctuate`] for layouts that place
/// items individually, such as [`fill_sep`], where a separator of its own
/// could end up starting a line.
#[must_use]
pub fn punctuate_end<I: IntoIterator<Item = Doc>>(sep: &Doc, parts: I) -> Vec<Doc> {
    let mut out: Vec<Doc> = parts.into_iter().collect();
    if let Some((_, rest)) = out.split_last_mut() {
        for part in rest {
            *part = concat([part.clone(), sep.clone()]);
        }
    }
    out
}

/// Wrap `body` in `left` and `right`.
#[must_use]
pub fn enclose(left: Doc, right: Doc, body: Doc) -> Doc {
    concat([left, body, right])
}

/// Wrap in `(` and `)`.
#[must_use]
pub fn parens(d: Doc) -> Doc {
    enclose(lparen(), rparen(), d)
}

/// Wrap in `[` and `]`.
#[must_use]
pub fn brackets(d: Doc) -> Doc {
    enclose(lbracket(), rbracket(), d)
}

/// Wrap in `{` and `}`.
#[must_use]
pub fn braces(d: Doc) -> Doc {
    enclose(lbrace(), rbrace(), d)
}

/// Wrap in `<` and `>`.
#[must_use]
pub fn angles(d: Doc) -> Doc {
    enclose(langle(), rangle(), d)
}

/// Wrap in double quotes.
#[must_use]
pub fn dquotes(d: Doc) -> Doc {
    enclose(dquote(), dquote(), d)
}

/// Wrap in single quotes.
#[must_use]
pub fn squotes(d: Doc) -> Doc {
    enclose(squote(), squote(), d)
}

/// [`punctuate`] the items with `sep` and wrap them in `left` and `right`,
/// with no line breaks of its own. See [`block`] for the breaking version.
#[must_use]
pub fn enclose_sep<I: IntoIterator<Item = Doc>>(left: Doc, right: Doc, sep: &Doc, parts: I) -> Doc {
    enclose(left, right, concat(punctuate(sep, parts)))
}

/// A comma-separated list in square brackets, all on one line.
#[must_use]
pub fn list<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    enclose_sep(lbracket(), rbracket(), &text(", "), parts)
}

/// A comma-separated list in parentheses, all on one line.
#[must_use]
pub fn tupled<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    enclose_sep(lparen(), rparen(), &text(", "), parts)
}

fn interleave<I: IntoIterator<Item = Doc>, F: Fn() -> Doc>(parts: I, sep: F) -> Doc {
    let parts = parts.into_iter();
    let mut out = Vec::with_capacity(parts.size_hint().0.saturating_mul(2));
    for (i, p) in parts.enumerate() {
        if i > 0 {
            out.push(sep());
        }
        out.push(p);
    }
    Doc::Concat(out)
}

impl Doc {
    /// Concatenate, flattening nested `Concat` nodes as it goes.
    #[must_use]
    pub fn append(self, other: Doc) -> Doc {
        match (self, other) {
            (Doc::Nil, x) | (x, Doc::Nil) => x,
            (Doc::Concat(mut a), Doc::Concat(b)) => {
                a.extend(b);
                Doc::Concat(a)
            }
            (Doc::Concat(mut a), b) => {
                a.push(b);
                Doc::Concat(a)
            }
            (a, Doc::Concat(mut b)) => {
                b.insert(0, a);
                Doc::Concat(b)
            }
            (a, b) => Doc::Concat(vec![a, b]),
        }
    }

    /// `self <+> other`: concatenate with a single space between.
    #[must_use]
    pub fn space(self, other: Doc) -> Doc {
        self.append(space()).append(other)
    }

    /// `self </> other`: concatenate with [`line()`] between (space when flat,
    /// newline when broken).
    #[must_use]
    pub fn line(self, other: Doc) -> Doc {
        self.append(line()).append(other)
    }

    /// `self <$> other`: concatenate with [`hardline`] between.
    #[must_use]
    pub fn hardline(self, other: Doc) -> Doc {
        self.append(hardline()).append(other)
    }

    /// `self <//> other`: concatenate with [`softline`] between (empty when
    /// flat, newline when broken).
    #[must_use]
    pub fn softline(self, other: Doc) -> Doc {
        self.append(softline()).append(other)
    }
}

impl Add for Doc {
    type Output = Doc;

    /// [`Doc::append`].
    fn add(self, other: Doc) -> Doc {
        self.append(other)
    }
}

impl FromIterator<Doc> for Doc {
    fn from_iter<I: IntoIterator<Item = Doc>>(iter: I) -> Self {
        concat(iter)
    }
}

impl From<String> for Doc {
    fn from(s: String) -> Self {
        Doc::Text(s)
    }
}

impl From<&str> for Doc {
    fn from(s: &str) -> Self {
        Doc::Text(s.to_owned())
    }
}

impl From<char> for Doc {
    fn from(c: char) -> Self {
        char(c)
    }
}

/// Force a subtree onto one line: `line` becomes a space, `softline` vanishes,
/// and `group` / `flat_alt` collapse to their flat layout. A `hardline` still
/// breaks, since a mandatory break cannot be flattened, matching Wadler
/// `flatten`.
///
/// Useful when a context forbids layout regardless of width (a bracketed or
/// offside-suppressed region), so you can keep building one `Doc` per construct
/// and flatten it at the boundary rather than maintaining a separate flat path.
#[must_use]
pub fn flatten(d: &Doc) -> Doc {
    match d {
        Doc::Line => Doc::Text(" ".to_owned()),
        Doc::SoftLine => Doc::Nil,
        Doc::FlatAlt(flat, _) => flatten(flat),
        Doc::Group(inner) => flatten(inner),
        Doc::Indent(n, inner) => Doc::Indent(*n, Box::new(flatten(inner))),
        Doc::Align(inner) => Doc::Align(Box::new(flatten(inner))),
        Doc::Concat(parts) => Doc::Concat(parts.iter().map(flatten).collect()),
        Doc::Fill(parts) => interleave(parts.iter().map(flatten), space),
        Doc::Nil | Doc::Text(_) | Doc::HardLine | Doc::Trivia(_) => d.clone(),
    }
}

/// Layout knobs for [`Block::of`] (and the [`block`] shortcut).
///
/// A block is the formatter's bread-and-butter delimited list: it stays on one
/// line when it fits and explodes to one item per line when it does not. The
/// knobs cover the variations real grammars need without a separate function
/// (or a row of unlabelled booleans) per shape.
#[derive(Clone, Copy, Debug)]
pub struct Block {
    /// Hanging indent applied to the items in the broken layout.
    pub nest: isize,
    /// Put a space just inside the delimiters in the flat layout (`{ a, b }`).
    pub pad: bool,
    /// Emit the separator after the final item in the broken layout.
    pub trailing: bool,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            nest: 2,
            pad: false,
            trailing: false,
        }
    }
}

impl Block {
    /// Pad the flat layout with a space just inside each delimiter (`{ a, b
    /// }`).
    #[must_use]
    pub const fn padded(mut self) -> Self {
        self.pad = true;
        self
    }

    /// Emit the separator after the final item when the block breaks.
    #[must_use]
    pub const fn trailing(mut self) -> Self {
        self.trailing = true;
        self
    }

    /// Override the hanging indent (default 2).
    #[must_use]
    pub const fn nest(mut self, n: isize) -> Self {
        self.nest = n;
        self
    }

    /// Build the delimited group. `sep` goes between items (and after the last
    /// when [`Block::trailing`] is set); the line break itself is supplied by
    /// the block, so pass just the punctuation (e.g. [`comma`]).
    #[must_use]
    pub fn of<I: IntoIterator<Item = Doc>>(
        self,
        open: Doc,
        close: Doc,
        sep: &Doc,
        items: I,
    ) -> Doc {
        let edge = if self.pad { line() } else { softline() };
        let between = sep.clone().append(line());
        let body = concat(punctuate(&between, items));
        let trail = if self.trailing {
            flat_alt(nil(), sep.clone())
        } else {
            nil()
        };
        group(concat([
            open,
            indent(self.nest, concat([edge.clone(), body, trail])),
            edge,
            close,
        ]))
    }
}

/// A delimited list that stays on one line when it fits and breaks to one item
/// per line (hanging-indented two spaces) when it does not: the everyday
/// `(a, b)`-style layout. Reach for [`Block`] when you need padded braces, a
/// trailing separator, or a different indent.
#[must_use]
pub fn block<I: IntoIterator<Item = Doc>>(open: Doc, close: Doc, sep: &Doc, items: I) -> Doc {
    Block::default().of(open, close, sep, items)
}
