# marginalia

Trivia-preserving parsing and formatting for [logos] + [lalrpop] grammars.

`marginalia` plugs into the standard Rust parsing stack and keeps comments and blank lines around
the AST, so you can write a formatter for your language without losing the user's notes.

The crate has three layers, all in one place:

- `TriviaLexer` adapts any `Iterator<Item = Result<(usize, Tok, usize), E>>` and records line,
  block, and blank-line trivia in a `TriviaTable` while the parser sees only semantic tokens.
- `marginalia::attach` places those trivia events on AST node spans as leading, trailing, or
  dangling comments.
- `marginalia::pretty` is a small `Doc` IR with explicit trivia slots that the renderer resolves
  against a `CommentMap`.

The shape of an integration:

```rust,ignore
use marginalia::TriviaLexer;
use marginalia::attach::{attach, AttachOptions};
use marginalia::pretty::{render, RenderOpts};

let raw = my_logos_lexer(source);
let mut lex = TriviaLexer::new(raw, source);
let program = MyParser::new().parse(&mut lex)?;
let table = lex.into_table();

let map = attach(source, &table, my_spans(&program), AttachOptions::default());
let doc = program.doc();
let formatted = render(&doc, &map, RenderOpts::default());
```

The lexer must yield `Result<(usize, Tok, usize), E>` and `Tok` must implement
`marginalia::Classify` so marginalia knows which tokens are comments.

## Example

[`examples/calc`](examples/calc) is a 200-line calculator language with full lex + parse + format
roundtrip and comment preservation. Read it end-to-end as the canonical integration template:

```bash
just calc examples/calc/tests/input.calc
```

## License

MIT. See [LICENSE](LICENSE).

[logos]: https://crates.io/crates/logos
[lalrpop]: https://crates.io/crates/lalrpop
