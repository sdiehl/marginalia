use std::collections::HashSet;

use super::doc::{Doc, Side, TriviaSlot};
use crate::{attach::CommentMap, trivia::TriviaClass, Span, Trivia};

#[derive(Clone, Copy, Debug)]
pub struct RenderOpts {
    pub width: usize,
    /// Starting indentation level, in columns. The document renders as if the
    /// cursor already sits at this column: width is budgeted from here and
    /// broken lines pad to at least this much, but the first line is *not*
    /// pre-padded (the caller positions it). This is what lets a `Doc` render
    /// into an already-indented slot of a larger, string-built output.
    pub indent: usize,
    /// Append unattached (dangling) trivia at the end of the document.
    pub emit_dangling: bool,
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            width: 80,
            indent: 0,
            emit_dangling: true,
        }
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Flat,
    Break,
}

struct Frame<'a> {
    indent: isize,
    mode: Mode,
    doc: &'a Doc,
}

#[must_use]
pub fn render<K: TriviaClass>(doc: &Doc, comments: &CommentMap<K>, opts: RenderOpts) -> String {
    let base = isize::try_from(opts.indent).unwrap_or(0);
    let mut out = String::new();
    let mut stack: Vec<Frame<'_>> = vec![Frame {
        indent: base,
        mode: Mode::Break,
        doc,
    }];
    // Start as if the cursor is already at `indent`, so width budgeting and
    // group-fit decisions account for the slot the caller placed us in.
    let mut col: usize = opts.indent;
    let mut emitted: HashSet<(Span, Side)> = HashSet::new();

    while let Some(Frame { indent, mode, doc }) = stack.pop() {
        match doc {
            Doc::Nil => {}
            Doc::Text(s) => {
                out.push_str(s);
                col += s.len();
            }
            Doc::Line => match mode {
                Mode::Flat => {
                    out.push(' ');
                    col += 1;
                }
                Mode::Break => newline(&mut out, &mut col, indent),
            },
            Doc::SoftLine => match mode {
                Mode::Flat => {}
                Mode::Break => newline(&mut out, &mut col, indent),
            },
            Doc::HardLine => newline(&mut out, &mut col, indent),
            Doc::Indent(n, inner) => stack.push(Frame {
                indent: indent + n,
                mode,
                doc: inner,
            }),
            Doc::Align(inner) => stack.push(Frame {
                indent: isize::try_from(col).unwrap_or(indent),
                mode,
                doc: inner,
            }),
            Doc::FlatAlt(flat, broken) => {
                let chosen = match mode {
                    Mode::Flat => flat,
                    Mode::Break => broken,
                };
                stack.push(Frame {
                    indent,
                    mode,
                    doc: chosen,
                });
            }
            Doc::Group(inner) => {
                let inner_indent = usize::try_from(indent.max(0)).unwrap_or(0);
                let chosen = if fits(opts.width.saturating_sub(col), inner, &stack) {
                    Mode::Flat
                } else {
                    let _ = inner_indent;
                    Mode::Break
                };
                stack.push(Frame {
                    indent,
                    mode: chosen,
                    doc: inner,
                });
            }
            Doc::Concat(parts) => {
                for p in parts.iter().rev() {
                    stack.push(Frame {
                        indent,
                        mode,
                        doc: p,
                    });
                }
            }
            Doc::Trivia(slot) => {
                if emitted.insert((slot.span, slot.side)) {
                    emit_trivia(*slot, comments, &mut out, &mut col, indent);
                }
            }
        }
    }

    if opts.emit_dangling {
        emit_dangling(comments.dangling(), &mut out, &mut col);
    }

    out
}

/// Render a `Doc` at `width` with no comments — the common trivia-free case,
/// without having to spell out a [`CommentMap`] and [`RenderOpts`].
#[must_use]
pub fn pretty(doc: &Doc, width: usize) -> String {
    pretty_at(doc, width, 0)
}

/// Like [`pretty`], but starting in a slot already indented `indent` columns —
/// the shortcut for dropping a `Doc` into an already-indented position of a
/// larger, string-built output. See [`RenderOpts::indent`].
#[must_use]
pub fn pretty_at(doc: &Doc, width: usize, indent: usize) -> String {
    render(
        doc,
        &CommentMap::<crate::BuiltinKind>::default(),
        RenderOpts {
            width,
            indent,
            emit_dangling: false,
        },
    )
}

/// Render a `Doc` flattened onto a single line (soft breaks collapsed). A
/// `hardline` still breaks. Equivalent to rendering [`super::flatten`] of the
/// document at unbounded width.
#[must_use]
pub fn pretty_flat(doc: &Doc) -> String {
    pretty(&super::doc::flatten(doc), usize::MAX)
}

fn emit_dangling<K>(items: &[Trivia<K>], out: &mut String, col: &mut usize) {
    if items.is_empty() {
        return;
    }
    if !out.is_empty() {
        if *col != 0 {
            out.push('\n');
        }
        out.push('\n');
        *col = 0;
    }
    for t in items {
        match t {
            Trivia::BlankLine => {
                out.push('\n');
                *col = 0;
            }
            Trivia::Comment { text, .. } => {
                if *col != 0 {
                    out.push('\n');
                    *col = 0;
                }
                out.push_str(text);
                *col += text.len();
                out.push('\n');
                *col = 0;
            }
        }
    }
}

fn newline(out: &mut String, col: &mut usize, indent: isize) {
    out.push('\n');
    let pad = usize::try_from(indent.max(0)).unwrap_or(0);
    for _ in 0..pad {
        out.push(' ');
    }
    *col = pad;
}

/// Would rendering `doc` flat, followed by the rest of the document, keep the
/// current line within `remaining` columns?
///
/// Two phases with different semantics. The candidate group itself is measured
/// strictly flat: every `Line` is a space and a `HardLine` disqualifies the
/// flat rendering outright. The rest of the document is then walked only to
/// the END of the current line, mode-aware: a `Line`/`SoftLine` under a frame
/// already in break mode, or a `HardLine` anywhere, terminates the line, so
/// whatever follows it cannot overflow this one and the group fits. Without
/// that stop, a group's fit would depend on the entire tail of the document,
/// breaking groups that render well inside an already-broken parent.
fn fits(mut remaining: usize, doc: &Doc, rest: &[Frame<'_>]) -> bool {
    // Phase 1: the candidate, measured flat.
    let mut local: Vec<&Doc> = vec![doc];
    while let Some(d) = local.pop() {
        match d {
            Doc::Nil | Doc::Trivia(_) | Doc::SoftLine => {}
            Doc::Text(s) => {
                if s.len() > remaining {
                    return false;
                }
                remaining -= s.len();
            }
            Doc::Line => {
                if remaining == 0 {
                    return false;
                }
                remaining -= 1;
            }
            Doc::HardLine => return false,
            Doc::Indent(_, inner) | Doc::Group(inner) | Doc::Align(inner) => local.push(inner),
            Doc::FlatAlt(flat, _) => local.push(flat),
            Doc::Concat(parts) => {
                for p in parts.iter().rev() {
                    local.push(p);
                }
            }
        }
    }

    // Phase 2: the rest of the document, up to the end of the current line.
    // Each pending frame carries its own mode; an undecided nested group is
    // measured optimistically flat (it will get its own fit decision when
    // rendering reaches it).
    let mut tail: Vec<(Mode, &Doc)> = Vec::new();
    for frame in rest.iter().rev() {
        tail.push((frame.mode, frame.doc));
        while let Some((mode, d)) = tail.pop() {
            match d {
                Doc::Nil | Doc::Trivia(_) => {}
                Doc::Text(s) => {
                    if s.len() > remaining {
                        return false;
                    }
                    remaining -= s.len();
                }
                Doc::SoftLine => match mode {
                    Mode::Flat => {}
                    Mode::Break => return true,
                },
                Doc::Line => match mode {
                    Mode::Flat => {
                        if remaining == 0 {
                            return false;
                        }
                        remaining -= 1;
                    }
                    Mode::Break => return true,
                },
                Doc::HardLine => return true,
                Doc::Indent(_, inner) | Doc::Align(inner) => tail.push((mode, inner)),
                Doc::Group(inner) => tail.push((Mode::Flat, inner)),
                Doc::FlatAlt(flat, broken) => match mode {
                    Mode::Flat => tail.push((mode, flat)),
                    Mode::Break => tail.push((mode, broken)),
                },
                Doc::Concat(parts) => {
                    for p in parts.iter().rev() {
                        tail.push((mode, p));
                    }
                }
            }
        }
    }
    true
}

fn emit_trivia<K: TriviaClass>(
    slot: TriviaSlot,
    comments: &CommentMap<K>,
    out: &mut String,
    col: &mut usize,
    indent: isize,
) {
    let items: &[Trivia<K>] = match slot.side {
        Side::Leading => comments.leading(slot.span),
        Side::Trailing => comments.trailing(slot.span),
    };
    if items.is_empty() {
        return;
    }
    match slot.side {
        Side::Leading => emit_leading(items, out, col, indent),
        Side::Trailing => emit_trailing(items, out, col, indent),
    }
}

fn emit_leading<K: TriviaClass>(
    items: &[Trivia<K>],
    out: &mut String,
    col: &mut usize,
    indent: isize,
) {
    let leading_needs_newline = *col != usize::try_from(indent.max(0)).unwrap_or(0);
    if leading_needs_newline {
        newline(out, col, indent);
    }
    for (i, t) in items.iter().enumerate() {
        match t {
            Trivia::BlankLine => {
                if i > 0 {
                    out.push('\n');
                    *col = 0;
                }
            }
            Trivia::Comment { text, .. } => {
                if i > 0 {
                    newline(out, col, indent);
                }
                out.push_str(text);
                *col += text.len();
            }
        }
    }
    newline(out, col, indent);
}

fn emit_trailing<K: TriviaClass>(
    items: &[Trivia<K>],
    out: &mut String,
    col: &mut usize,
    indent: isize,
) {
    let mut first = true;
    for t in items {
        match t {
            Trivia::BlankLine => {}
            Trivia::Comment { kind, text } => {
                if first {
                    out.push(' ');
                    *col += 1;
                } else if kind.is_line_like() {
                    newline(out, col, indent);
                } else {
                    out.push(' ');
                    *col += 1;
                }
                out.push_str(text);
                *col += text.len();
                first = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pretty::{
        block, comma, flatten, group, lparen, pretty, pretty_flat, rparen, text, Block, RenderOpts,
    };

    // A group that breaks at a narrow width flattens back onto one line.
    #[test]
    fn flatten_collapses_breaks() {
        let d = group(text("a").line(text("b")).line(text("c")));
        assert_eq!(pretty(&d, 1), "a\nb\nc");
        assert_eq!(pretty_flat(&d), "a b c");
        assert_eq!(pretty(&flatten(&d), 1), "a b c");
    }

    // `block` is tight on one line when it fits.
    #[test]
    fn block_flat() {
        let d = block(
            lparen(),
            rparen(),
            &comma(),
            [text("a"), text("b"), text("c")],
        );
        assert_eq!(pretty(&d, 80), "(a, b, c)");
    }

    // ...and explodes to one item per line (two-space hang) when it does not.
    #[test]
    fn block_broken() {
        let d = block(lparen(), rparen(), &comma(), [text("aaa"), text("bbb")]);
        assert_eq!(pretty(&d, 5), "(\n  aaa,\n  bbb\n)");
    }

    // Padded + trailing is the record shape: inner spaces flat, trailing comma
    // when broken.
    #[test]
    fn block_record_shape() {
        let style = Block::default().padded().trailing();
        let items = || [text("x = 1"), text("y = 2")];
        let flat = style.of(text("{"), text("}"), &comma(), items());
        assert_eq!(pretty(&flat, 80), "{ x = 1, y = 2 }");
        let broken = style.of(text("{"), text("}"), &comma(), items());
        assert_eq!(pretty(&broken, 5), "{\n  x = 1,\n  y = 2,\n}");
    }

    // `RenderOpts.indent` budgets width from the slot and pads continuation
    // lines, without pre-padding the first line.
    #[test]
    fn indent_offsets_continuations() {
        let d = block(lparen(), rparen(), &comma(), [text("aaa"), text("bbb")]);
        let out = pretty_at(&d, 10, 4);
        assert_eq!(out, "(\n      aaa,\n      bbb\n    )");
    }

    fn pretty_at(d: &crate::pretty::Doc, width: usize, indent: usize) -> String {
        crate::pretty::render(
            d,
            &crate::attach::CommentMap::<crate::BuiltinKind>::default(),
            RenderOpts {
                width,
                indent,
                emit_dangling: false,
            },
        )
    }

    // A group's fit stops at the end of the current line: a hard break after
    // the group ends the line, so a long tail on later lines cannot force the
    // group to break.
    #[test]
    fn fit_stops_at_hardline_in_rest() {
        let d =
            group(text("(a").line(text("b)"))).hardline(text("cccccccccccccccccccccccccccccccc"));
        assert_eq!(pretty(&d, 10), "(a b)\ncccccccccccccccccccccccccccccccc");
    }

    // Inside an already-broken parent, each child group is measured against
    // its own line only: the first child stays flat even though a later
    // sibling on the next line is long.
    #[test]
    fn fit_stops_at_break_mode_line_in_rest() {
        let inner1 = group(text("(a").line(text("b)")));
        let inner2 = group(text("(cccccccccc").line(text("dddddddddd)")));
        let d = group(text("[").line(inner1).line(inner2).line(text("]")));
        assert_eq!(pretty(&d, 12), "[\n(a b)\n(cccccccccc\ndddddddddd)\n]");
    }

    // The candidate group itself is still measured strictly flat: a hard line
    // inside the group disqualifies the flat rendering.
    #[test]
    fn hardline_inside_group_still_breaks_it() {
        let d = group(text("a").hardline(text("b")).line(text("c")));
        assert_eq!(pretty(&d, 80), "a\nb\nc");
    }
}
