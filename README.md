# marginalia

Trivia-preserving parsing and formatting for [logos] + [lalrpop] grammars.

`marginalia` plugs into the standard Rust parsing stack and keeps comments and blank lines around
the AST, so you can write a formatter for your language without losing the user's notes.

The workspace ships three crates:

| crate                 | role                                                                   |
| --------------------- | ---------------------------------------------------------------------- |
| [`marginalia`]        | `TriviaLexer` adapter that records line/block/blank trivia on the side |
| [`marginalia-attach`] | Attaches recorded trivia to AST node spans (leading/trailing/dangling) |
| [`marginalia-pretty`] | A `Doc` IR and renderer that emits trivia at the right slot            |

## Install

```toml
[dependencies]
marginalia = "0.1"
marginalia-attach = "0.1"
marginalia-pretty = "0.1"
```

MSRV: 1.86.

## Quickstart

The shape of an integration:

```rust,ignore
use marginalia::TriviaLexer;
use marginalia_attach::{attach, AttachOptions};
use marginalia_pretty::{render, RenderOpts};

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

## Development

```bash
just test    # cargo test --workspace
just lint    # fmt + clippy
just fmt     # cargo fmt + dprint
```

## License

MIT. See [LICENSE](LICENSE).

[logos]: https://github.com/maciejhirsz/logos
[lalrpop]: https://github.com/lalrpop/lalrpop
[`marginalia`]: crates/marginalia
[`marginalia-attach`]: crates/marginalia-attach
[`marginalia-pretty`]: crates/marginalia-pretty
