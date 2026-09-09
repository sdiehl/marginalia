# Changelog

All notable changes to this project are documented in this file.

## [0.3.0] - 2026-09-09

### Fixed

- `render` measures line width in Unicode scalar values, not UTF-8 bytes.
- `attach` is `O(E log N)` in trivia events and node spans, down from `O(E * N)`.
- `attach` and `TriviaLexer` scan for line breaks bytewise, so non-`char` boundaries are safe.

### Added

- `Doc` implements `Default`, `PartialEq`, `Eq`, `Add`, `FromIterator`, and `From` for strings.
- `TriviaTable` implements `Extend`, `FromIterator`, `IntoIterator` for refs, and gains `iter`.
- `CommentMap` gains `iter`, `dangling_len`, and `comment_len`; the iterator is `attach::Anchors`.
- `Comments::len`, plus a runnable crate-level example covering attach and render.

### Changed

- Every public item is documented and `missing_docs` is enforced.
- `into_table` and `into_parts` are `#[must_use]`, and the README is doctested.

## [0.2.1] - 2026-07-04

### Fixed

- Group fit measurement stops at the end of the line instead of measuring the rest of the document.
- Nested groups no longer staircase inside a broken parent because of long content on later lines.
- A `HardLine` inside the candidate group itself still disqualifies the flat rendering.

## [0.2.0] - 2026-06-16

### Added

- `flatten` collapses a document onto a single line.
- `Doc::of` builds a document from any `IntoIterator<Item = Doc>`.
- `block` and the `Block` builder (`padded`, `trailing`, `nest`) for delimited, separated groups.
- `pretty`, `pretty_at`, and `pretty_flat` rendering shortcuts alongside `render`.

### Changed

- Flattened the internal render helpers for a more direct render path.

## [0.1.4] - 2026-05-22

### Changed

- BREAKING: `Trivia::Line` and `Trivia::Block` collapse into `Trivia::Comment { kind, text }`.
- BREAKING: `TriviaKind` is renamed `BuiltinKind`, and `Trivia::from_kind` is removed.
- `Trivia`, `TriviaTable`, `CommentMap`, and `Classify` are generic over a kind `K = BuiltinKind`.

### Added

- `TriviaClass` trait exposing `is_line_like`, so the renderer can lay out custom kinds.
- `Trivia::kind` accessor.
- Pre-commit hook configuration.

## [0.1.3] - 2026-05-21

### Added

- Layout combinators `align`, `hang`, and `flat_alt`, wired through the renderer.
- Concatenation combinators `hcat`, `hsep`, `vcat`, `vsep`, `sep`, and `cat`.
- Enclosers `enclose`, `parens`, `brackets`, `braces`, `angles`, `dquotes`, and `squotes`.
- `enclose_sep`, `list`, `tupled`, `punctuate`, and punctuation helpers such as `comma` and `semi`.
- `Doc` join methods `space`, `line`, `hardline`, and `softline`.

### Fixed

- README corrections.

## [0.1.2] - 2026-05-20

### Added

- `Span` converts to and from `(usize, usize)` and `Range<usize>`.
- `TriviaTable::events_in` looks up the events falling inside a span.
- `RenderOpts::emit_dangling` appends unattached trivia at the end of the document.

### Fixed

- The renderer deduplicates trivia slots by `(span, side)`, so a comment is emitted at most once.

### Changed

- A comment after the last anchor is now dangling rather than trailing on the previous anchor.

## [0.1.1] - 2026-05-20

### Changed

- MSRV raised to Rust 1.95.
- Releases publish with `cargo publish --locked`, and the `justfile` is gone.

### Added

- Dependabot configuration.

## [0.1.0] - 2026-05-20

### Added

- Initial release: trivia-preserving parsing and formatting for `logos` and `lalrpop`.
- `TriviaLexer` records comments and blank lines on the side while the parser sees only tokens.
- `attach` places trivia events on AST node spans as leading, trailing, or dangling comments.
- `pretty` renders a `Doc` IR whose trivia slots resolve against a `CommentMap`.

[0.3.0]: https://github.com/sdiehl/marginalia/releases/tag/v0.3.0
[0.2.1]: https://github.com/sdiehl/marginalia/releases/tag/v0.2.1
[0.2.0]: https://github.com/sdiehl/marginalia/releases/tag/v0.2.0
[0.1.4]: https://github.com/sdiehl/marginalia/releases/tag/v0.1.4
[0.1.3]: https://github.com/sdiehl/marginalia/releases/tag/v0.1.3
[0.1.2]: https://github.com/sdiehl/marginalia/releases/tag/v0.1.2
[0.1.1]: https://github.com/sdiehl/marginalia/releases/tag/v0.1.1
[0.1.0]: https://github.com/sdiehl/marginalia/releases/tag/v0.1.0
