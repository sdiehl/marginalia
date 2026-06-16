use crate::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Leading,
    Trailing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TriviaSlot {
    pub span: Span,
    pub side: Side,
}

#[derive(Clone, Debug)]
pub enum Doc {
    Nil,
    Text(String),
    Line,
    SoftLine,
    HardLine,
    Indent(isize, Box<Doc>),
    Align(Box<Doc>),
    FlatAlt(Box<Doc>, Box<Doc>),
    Group(Box<Doc>),
    Concat(Vec<Doc>),
    Trivia(TriviaSlot),
}

#[must_use]
pub const fn nil() -> Doc {
    Doc::Nil
}

#[must_use]
pub fn text(s: impl Into<String>) -> Doc {
    Doc::Text(s.into())
}

#[must_use]
pub fn char(c: char) -> Doc {
    Doc::Text(c.to_string())
}

#[must_use]
pub const fn line() -> Doc {
    Doc::Line
}

#[must_use]
pub const fn softline() -> Doc {
    Doc::SoftLine
}

#[must_use]
pub const fn hardline() -> Doc {
    Doc::HardLine
}

#[must_use]
pub fn indent(n: isize, d: Doc) -> Doc {
    Doc::Indent(n, Box::new(d))
}

#[must_use]
pub fn align(d: Doc) -> Doc {
    Doc::Align(Box::new(d))
}

#[must_use]
pub fn hang(n: isize, d: Doc) -> Doc {
    align(indent(n, d))
}

#[must_use]
pub fn flat_alt(flat: Doc, broken: Doc) -> Doc {
    Doc::FlatAlt(Box::new(flat), Box::new(broken))
}

#[must_use]
pub fn group(d: Doc) -> Doc {
    Doc::Group(Box::new(d))
}

#[must_use]
pub fn concat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    Doc::Concat(parts.into_iter().collect())
}

#[must_use]
pub const fn trivia(span: Span, side: Side) -> Doc {
    Doc::Trivia(TriviaSlot { span, side })
}

#[must_use]
pub fn space() -> Doc {
    text(" ")
}
#[must_use]
pub fn comma() -> Doc {
    text(",")
}
#[must_use]
pub fn semi() -> Doc {
    text(";")
}
#[must_use]
pub fn colon() -> Doc {
    text(":")
}
#[must_use]
pub fn dot() -> Doc {
    text(".")
}
#[must_use]
pub fn equals() -> Doc {
    text("=")
}
#[must_use]
pub fn lparen() -> Doc {
    text("(")
}
#[must_use]
pub fn rparen() -> Doc {
    text(")")
}
#[must_use]
pub fn lbracket() -> Doc {
    text("[")
}
#[must_use]
pub fn rbracket() -> Doc {
    text("]")
}
#[must_use]
pub fn lbrace() -> Doc {
    text("{")
}
#[must_use]
pub fn rbrace() -> Doc {
    text("}")
}
#[must_use]
pub fn langle() -> Doc {
    text("<")
}
#[must_use]
pub fn rangle() -> Doc {
    text(">")
}
#[must_use]
pub fn dquote() -> Doc {
    text("\"")
}
#[must_use]
pub fn squote() -> Doc {
    text("'")
}

#[must_use]
pub fn hcat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    concat(parts)
}

#[must_use]
pub fn hsep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    interleave(parts, space)
}

#[must_use]
pub fn vcat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    interleave(parts, hardline)
}

#[must_use]
pub fn vsep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    interleave(parts, line)
}

#[must_use]
pub fn sep<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    group(vsep(parts))
}

#[must_use]
pub fn cat<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    group(interleave(parts, softline))
}

#[must_use]
pub fn punctuate<I: IntoIterator<Item = Doc>>(sep: &Doc, parts: I) -> Vec<Doc> {
    let items: Vec<Doc> = parts.into_iter().collect();
    let last = items.len().saturating_sub(1);
    items
        .into_iter()
        .enumerate()
        .flat_map(|(i, d)| {
            if i == last {
                vec![d]
            } else {
                vec![d, sep.clone()]
            }
        })
        .collect()
}

#[must_use]
pub fn enclose(left: Doc, right: Doc, body: Doc) -> Doc {
    concat([left, body, right])
}

#[must_use]
pub fn parens(d: Doc) -> Doc {
    enclose(lparen(), rparen(), d)
}
#[must_use]
pub fn brackets(d: Doc) -> Doc {
    enclose(lbracket(), rbracket(), d)
}
#[must_use]
pub fn braces(d: Doc) -> Doc {
    enclose(lbrace(), rbrace(), d)
}
#[must_use]
pub fn angles(d: Doc) -> Doc {
    enclose(langle(), rangle(), d)
}
#[must_use]
pub fn dquotes(d: Doc) -> Doc {
    enclose(dquote(), dquote(), d)
}
#[must_use]
pub fn squotes(d: Doc) -> Doc {
    enclose(squote(), squote(), d)
}

#[must_use]
pub fn enclose_sep<I: IntoIterator<Item = Doc>>(left: Doc, right: Doc, sep: &Doc, parts: I) -> Doc {
    enclose(left, right, concat(punctuate(sep, parts)))
}

#[must_use]
pub fn list<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    enclose_sep(lbracket(), rbracket(), &text(", "), parts)
}

#[must_use]
pub fn tupled<I: IntoIterator<Item = Doc>>(parts: I) -> Doc {
    enclose_sep(lparen(), rparen(), &text(", "), parts)
}

fn interleave<I: IntoIterator<Item = Doc>, F: Fn() -> Doc>(parts: I, sep: F) -> Doc {
    let mut out = Vec::new();
    for (i, p) in parts.into_iter().enumerate() {
        if i > 0 {
            out.push(sep());
        }
        out.push(p);
    }
    concat(out)
}

impl Doc {
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

    /// `self <+> other` — concatenate with a single space between.
    #[must_use]
    pub fn space(self, other: Doc) -> Doc {
        self.append(space()).append(other)
    }

    /// `self </> other` — concatenate with `line` between (space when flat,
    /// newline when broken).
    #[must_use]
    pub fn line(self, other: Doc) -> Doc {
        self.append(line()).append(other)
    }

    /// `self <$> other` — concatenate with `hardline` between.
    #[must_use]
    pub fn hardline(self, other: Doc) -> Doc {
        self.append(hardline()).append(other)
    }

    /// `self <//> other` — concatenate with `softline` between (empty when
    /// flat, newline when broken).
    #[must_use]
    pub fn softline(self, other: Doc) -> Doc {
        self.append(softline()).append(other)
    }
}

/// Force a subtree onto one line: `line` becomes a space, `softline` vanishes,
/// and `group` / `flat_alt` collapse to their flat layout. A `hardline` still
/// breaks — a mandatory break cannot be flattened — matching Wadler `flatten`.
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
/// per line (hanging-indented two spaces) when it does not — the everyday
/// `(a, b)`-style layout. Reach for [`Block`] when you need padded braces, a
/// trailing separator, or a different indent.
#[must_use]
pub fn block<I: IntoIterator<Item = Doc>>(open: Doc, close: Doc, sep: &Doc, items: I) -> Doc {
    Block::default().of(open, close, sep, items)
}
