# Changelog

## [0.2.1] - 2026-07-04

### Fixed

- `pretty::render`'s group-fit measurement now stops at the end of the current line instead of
  measuring the entire remaining document: a `Line` or `SoftLine` belonging to an already-broken
  enclosing frame, or a `HardLine` anywhere after the group, terminates the line, so a group that
  fits its own line no longer breaks because of long content on later lines. A `HardLine` inside the
  candidate group itself still disqualifies the flat rendering. Fixes staircasing of nested groups
  inside a broken parent (every non-final element of a broken sequence previously over-broke).

## [0.2.0] - 2026-06-16

### Added

- `pretty::doc::flatten` to collapse a document onto a single line.
- `Doc::of` constructor over any `IntoIterator<Item = Doc>`.
- `pretty::doc::block` for rendering open/close-delimited, separated groups.
- `pretty_at` and `pretty_flat` rendering entry points alongside `pretty`.

### Changed

- Flattened internal rendering helpers for a simpler, more direct render path.

## [0.1.4] - 2026-05-22

### Changed

- Generalized the trivia classifier over a `TriviaClass` kind, decoupling trivia classification from
  concrete token types.
- Reworked the attacher, classify, lexer, and table layers around the generic trivia kind.

### Added

- Pre-commit hook configuration.

## [0.1.3] - 2026-05-21

### Added

- Full set of pretty-printing combinators in `pretty::doc`: layout (`align`, `hang`, `flat_alt`),
  concatenation (`hcat`, `hsep`, `vcat`, `vsep`, `sep`, `cat`), punctuation helpers (`comma`,
  `semi`, `colon`, `dot`, `equals`, and bracket/quote characters), enclosure (`enclose`, `parens`,
  `brackets`, `braces`, `angles`, `dquotes`, `squotes`, `enclose_sep`, `list`, `tupled`), and
  `punctuate`.
- `Doc` combinator methods: `space`, `line`, `hardline`, `softline`.

### Fixed

- README corrections.

## [0.1.2] - 2026-05-20

### Added

- More ergonomic API surface for spans, tables, and formatting.
- Format helpers in `pretty::format`.

## [0.1.1] - 2026-05-20

### Added

- Initial release: trivia-preserving parsing and formatting for `logos` + `lalrpop`, including the
  lexer, trivia classification and attachment, CST tables, span handling, and the pretty-printing
  engine.
