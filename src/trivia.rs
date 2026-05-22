/// Built-in trivia kinds for the common case where a language only needs to
/// distinguish line from block comments.
///
/// Downstream crates that need finer categories (doc comments, attributes,
/// region markers, etc.) define their own kind enum and parameterize
/// [`Trivia`] / [`TriviaTable`] / [`crate::attach::CommentMap`] with it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum BuiltinKind {
    /// Comment that runs to end-of-line.
    #[default]
    Line,
    /// Bracketed comment with explicit open/close delimiters.
    Block,
}

/// Layout properties a trivia kind exposes to the renderer.
///
/// The renderer is generic over the kind enum but needs one bit of layout
/// information: whether a trailing comment ends the line it sits on. Block
/// comments do not; line comments do. Implement this on your custom kind
/// enum to teach the renderer how to break trailing trivia across lines.
pub trait TriviaClass {
    /// `true` if a comment of this kind terminates the source line it sits on
    /// (so a sibling trailing comment after it must start on a new line).
    fn is_line_like(&self) -> bool;
}

impl TriviaClass for BuiltinKind {
    fn is_line_like(&self) -> bool {
        matches!(self, Self::Line)
    }
}

/// A piece of trivia recorded between two semantic tokens.
///
/// Generic over `K` so callers can carry a richer classification through the
/// pipeline; defaults to [`BuiltinKind`] for the line/block-only case that
/// most languages need.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trivia<K = BuiltinKind> {
    /// A comment, with its kind tag and verbatim source text.
    Comment { kind: K, text: String },
    /// A run of two or more newlines (blank line) between semantic tokens.
    BlankLine,
}

impl<K> Trivia<K> {
    #[must_use]
    pub const fn is_blank(&self) -> bool {
        matches!(self, Self::BlankLine)
    }

    #[must_use]
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Comment { text, .. } => Some(text),
            Self::BlankLine => None,
        }
    }

    #[must_use]
    pub const fn kind(&self) -> Option<&K> {
        match self {
            Self::Comment { kind, .. } => Some(kind),
            Self::BlankLine => None,
        }
    }
}

impl<K: TriviaClass> Trivia<K> {
    /// True for `Comment { kind, .. }` where the kind reports itself as
    /// line-terminating. Used by the renderer to decide how to break trailing
    /// trivia across lines.
    #[must_use]
    pub fn is_line_like(&self) -> bool {
        match self {
            Self::Comment { kind, .. } => kind.is_line_like(),
            Self::BlankLine => false,
        }
    }
}

impl Trivia<BuiltinKind> {
    /// Convenience constructor for a line comment with the built-in kind.
    #[must_use]
    pub fn line(text: impl Into<String>) -> Self {
        Self::Comment {
            kind: BuiltinKind::Line,
            text: text.into(),
        }
    }

    /// Convenience constructor for a block comment with the built-in kind.
    #[must_use]
    pub fn block(text: impl Into<String>) -> Self {
        Self::Comment {
            kind: BuiltinKind::Block,
            text: text.into(),
        }
    }
}
