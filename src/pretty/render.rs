use std::collections::HashSet;

use super::doc::{Doc, Side, TriviaSlot};
use crate::{attach::CommentMap, trivia::TriviaClass, Span, Trivia};

/// Knobs for [`render`].
#[derive(Clone, Copy, Debug)]
pub struct RenderOpts {
    /// Maximum line width, in columns, that groups are fitted against.
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

/// What a pending frame renders: a document, or the items of a
/// [`Doc::Fill`](Doc::Fill) that have not been placed yet.
#[derive(Clone, Copy)]
enum Item<'a> {
    Node(&'a Doc),
    Fill(&'a [Doc]),
}

struct Frame<'a> {
    indent: usize,
    mode: Mode,
    item: Item<'a>,
}

/// Lay out `doc` at the given width, filling its trivia slots from
/// `comments`.
///
/// Each `(span, side)` slot is emitted at most once per document even if the
/// same span is wrapped twice, so [`with_trivia`](super::with_trivia) is safe
/// to apply liberally.
#[must_use]
pub fn render<K: TriviaClass>(doc: &Doc, comments: &CommentMap<K>, opts: RenderOpts) -> String {
    let mut out = String::new();
    let mut stack: Vec<Frame<'_>> = vec![Frame {
        indent: opts.indent,
        mode: Mode::Break,
        item: Item::Node(doc),
    }];
    // Start as if the cursor is already at `indent`, so width budgeting and
    // group-fit decisions account for the slot the caller placed us in.
    let mut col: usize = opts.indent;
    let mut emitted: HashSet<(Span, Side)> = HashSet::new();

    while let Some(Frame { indent, mode, item }) = stack.pop() {
        let doc = match item {
            Item::Node(d) => d,
            Item::Fill(items) => {
                // Each of these is preceded by a separator. Decide it against
                // the column we have actually reached, with the rest of the
                // items already back on the stack so the fit check can see
                // that the line ends after this one either way.
                let Some((head, tail)) = items.split_first() else {
                    continue;
                };
                stack.push(Frame {
                    indent,
                    mode,
                    item: Item::Fill(tail),
                });
                if fits(opts.width.saturating_sub(col + 1), head, &stack) {
                    out.push(' ');
                    col += 1;
                } else {
                    newline(&mut out, &mut col, indent);
                }
                stack.push(Frame {
                    indent,
                    mode: fit_mode(opts.width, col, head, &stack),
                    item: Item::Node(head),
                });
                continue;
            }
        };
        match doc {
            Doc::Nil => {}
            Doc::Text(s) => {
                out.push_str(s);
                col += width(s);
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
                indent: indent.saturating_add_signed(*n),
                mode,
                item: Item::Node(inner),
            }),
            Doc::Align(inner) => stack.push(Frame {
                indent: col,
                mode,
                item: Item::Node(inner),
            }),
            Doc::FlatAlt(flat, broken) => {
                let chosen = match mode {
                    Mode::Flat => flat,
                    Mode::Break => broken,
                };
                stack.push(Frame {
                    indent,
                    mode,
                    item: Item::Node(chosen),
                });
            }
            Doc::Group(inner) => {
                let chosen = if fits(opts.width.saturating_sub(col), inner, &stack) {
                    Mode::Flat
                } else {
                    Mode::Break
                };
                stack.push(Frame {
                    indent,
                    mode: chosen,
                    item: Item::Node(inner),
                });
            }
            Doc::Concat(parts) => {
                for p in parts.iter().rev() {
                    stack.push(Frame {
                        indent,
                        mode,
                        item: Item::Node(p),
                    });
                }
            }
            Doc::Fill(parts) => {
                // The first item takes the current line as it finds it; only
                // the ones after it get a separator to decide.
                if let Some((head, tail)) = parts.split_first() {
                    stack.push(Frame {
                        indent,
                        mode,
                        item: Item::Fill(tail),
                    });
                    stack.push(Frame {
                        indent,
                        mode: fit_mode(opts.width, col, head, &stack),
                        item: Item::Node(head),
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
                *col += width(text);
                out.push('\n');
                *col = 0;
            }
        }
    }
}

/// Whether a fill item placed at `col` can render flat there.
fn fit_mode(width: usize, col: usize, doc: &Doc, rest: &[Frame<'_>]) -> Mode {
    if fits(width.saturating_sub(col), doc, rest) {
        Mode::Flat
    } else {
        Mode::Break
    }
}

fn newline(out: &mut String, col: &mut usize, indent: usize) {
    out.push('\n');
    for _ in 0..indent {
        out.push(' ');
    }
    *col = indent;
}

/// Display width of `s`, in columns.
///
/// Counted in Unicode scalar values rather than bytes, so a non-ASCII
/// identifier or comment budgets the same as its ASCII equivalent. Combining
/// marks and East Asian wide characters still count as one column each; a
/// formatter that needs those exact is better served by measuring its own
/// text and inserting explicit breaks.
fn width(s: &str) -> usize {
    s.chars().count()
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
                let w = width(s);
                if w > remaining {
                    return false;
                }
                remaining -= w;
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
            Doc::Fill(parts) => {
                if let Some(extra) = parts.len().checked_sub(1) {
                    if extra > remaining {
                        return false;
                    }
                    remaining -= extra;
                }
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
        match frame.item {
            Item::Node(d) => tail.push((frame.mode, d)),
            // A fill with items left to place ends this line at the latest
            // where those items go, so nothing beyond it can overflow it.
            // An exhausted one contributes nothing and the walk continues.
            Item::Fill(items) => {
                if items.is_empty() {
                    continue;
                }
                return true;
            }
        }
        while let Some((mode, d)) = tail.pop() {
            match d {
                Doc::Nil | Doc::Trivia(_) => {}
                Doc::Text(s) => {
                    let w = width(s);
                    if w > remaining {
                        return false;
                    }
                    remaining -= w;
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
                Doc::Fill(parts) => {
                    if !parts.is_empty() {
                        return true;
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
    indent: usize,
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
    indent: usize,
) {
    if *col != indent {
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
                *col += width(text);
            }
        }
    }
    newline(out, col, indent);
}

fn emit_trailing<K: TriviaClass>(
    items: &[Trivia<K>],
    out: &mut String,
    col: &mut usize,
    indent: usize,
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
                *col += width(text);
                first = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pretty::{
        block, comma, fill_sep, flatten, group, indent, lparen, pretty, pretty_flat, punctuate_end,
        rparen, text, Block, RenderOpts,
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

    // A fill packs as many items per line as fit, where a group would give
    // each its own line once the whole run overflows.
    #[test]
    fn fill_packs_lines() {
        let items = || punctuate_end(&comma(), (1..=9).map(|i| text(format!("item{i}"))));
        let flat = "item1, item2, item3, item4, item5, item6, item7, item8, item9";
        assert_eq!(pretty(&fill_sep(items()), 80), flat);
        assert_eq!(
            pretty(&fill_sep(items()), 30),
            "item1, item2, item3, item4,\nitem5, item6, item7, item8,\nitem9"
        );
    }

    // An item that does not fit flat takes a fresh line, and breaks internally
    // there if it still does not fit. What follows packs against the line it
    // ended on rather than starting another.
    #[test]
    fn fill_breaks_an_oversized_item() {
        let wide = group(text("(aaa").line(text("bbb)")));
        let d = fill_sep([text("x,"), wide, text("y")]);
        assert_eq!(pretty(&d, 8), "x,\n(aaa\nbbb) y");
    }

    // Continuation lines land on the enclosing indent.
    #[test]
    fn fill_indents_continuations() {
        let items = punctuate_end(&comma(), (1..=4).map(|i| text(format!("aaa{i}"))));
        assert_eq!(
            pretty(&indent(2, fill_sep(items)), 14),
            "aaa1, aaa2,\n  aaa3, aaa4"
        );
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

    // Width is budgeted in columns, not bytes: a multi-byte identifier must
    // not be charged for its UTF-8 length or every group holding one breaks
    // early.
    #[test]
    fn width_counts_columns_not_bytes() {
        let d = group(text("\u{e4}\u{e4}\u{e4}").line(text("b")));
        assert_eq!(pretty(&d, 5), "\u{e4}\u{e4}\u{e4} b");
        assert_eq!(pretty(&d, 4), "\u{e4}\u{e4}\u{e4}\nb");
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
