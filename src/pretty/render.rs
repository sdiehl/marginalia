use std::collections::HashSet;

use super::doc::{Doc, Side, TriviaSlot};
use crate::{attach::CommentMap, Span, Trivia};

#[derive(Clone, Copy, Debug)]
pub struct RenderOpts {
    pub width: usize,
    pub indent: usize,
    /// Append unattached (dangling) trivia at the end of the document.
    pub emit_dangling: bool,
}

impl Default for RenderOpts {
    fn default() -> Self {
        Self {
            width: 80,
            indent: 2,
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
pub fn render(doc: &Doc, comments: &CommentMap, opts: RenderOpts) -> String {
    let mut out = String::new();
    let mut stack: Vec<Frame<'_>> = vec![Frame {
        indent: 0,
        mode: Mode::Break,
        doc,
    }];
    let mut col: usize = 0;
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

fn emit_dangling(items: &[Trivia], out: &mut String, col: &mut usize) {
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
            Trivia::Line(s) | Trivia::Block(s) => {
                if *col != 0 {
                    out.push('\n');
                    *col = 0;
                }
                out.push_str(s);
                *col += s.len();
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

fn fits(mut remaining: usize, doc: &Doc, rest: &[Frame<'_>]) -> bool {
    let mut local: Vec<&Doc> = vec![doc];
    let mut rest_iter = rest.iter().rev();

    loop {
        let next = local.pop().or_else(|| rest_iter.next().map(|f| f.doc));
        let Some(d) = next else {
            return true;
        };
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
            Doc::HardLine => return true,
            Doc::Indent(_, inner) | Doc::Group(inner) => local.push(inner),
            Doc::Concat(parts) => {
                for p in parts.iter().rev() {
                    local.push(p);
                }
            }
        }
    }
}

fn emit_trivia(
    slot: TriviaSlot,
    comments: &CommentMap,
    out: &mut String,
    col: &mut usize,
    indent: isize,
) {
    let items: &[Trivia] = match slot.side {
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

fn emit_leading(items: &[Trivia], out: &mut String, col: &mut usize, indent: isize) {
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
            Trivia::Line(s) | Trivia::Block(s) => {
                if i > 0 {
                    newline(out, col, indent);
                }
                out.push_str(s);
                *col += s.len();
            }
        }
    }
    newline(out, col, indent);
}

fn emit_trailing(items: &[Trivia], out: &mut String, col: &mut usize, indent: isize) {
    let mut first = true;
    for t in items {
        match t {
            Trivia::BlankLine => {}
            Trivia::Line(s) => {
                if first {
                    out.push(' ');
                    *col += 1;
                } else {
                    newline(out, col, indent);
                }
                out.push_str(s);
                *col += s.len();
                first = false;
            }
            Trivia::Block(s) => {
                out.push(' ');
                *col += 1;
                out.push_str(s);
                *col += s.len();
                first = false;
            }
        }
    }
}
