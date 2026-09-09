use std::cmp::Reverse;

use super::map::{CommentMap, HasSpan};
use crate::{Span, TriviaEvent, TriviaTable};

/// Knobs for [`attach`].
#[derive(Clone, Copy, Debug)]
pub struct AttachOptions {
    /// Attach a comment as *trailing* on the preceding node when it sits on
    /// the same source line. With this off every comment becomes leading on
    /// the following node (or dangling).
    pub trailing_same_line: bool,
}

impl Default for AttachOptions {
    fn default() -> Self {
        Self {
            trailing_same_line: true,
        }
    }
}

/// Place each trivia event on the nearest enclosing anchor span.
///
/// `nodes` supplies the spans of the AST nodes that are allowed to carry
/// comments; duplicates are fine. Cost is `O(E log N)` for `E` events and `N`
/// distinct node spans.
pub fn attach<N: HasSpan, K: Clone + Default>(
    source: &str,
    table: &TriviaTable<K>,
    nodes: impl IntoIterator<Item = N>,
    opts: AttachOptions,
) -> CommentMap<K> {
    let index = Index::new(nodes.into_iter().map(|n| n.span()).collect());

    let mut map = CommentMap::new();
    for event in table.events() {
        place_event(source, event, &index, &mut map, opts);
    }
    map
}

/// Node spans indexed for nearest-neighbour lookup in both directions.
///
/// Neither query is monotonic under `Span`'s `(start, end)` ordering, because
/// spans nest and so `end` does not increase with `start`. Each direction
/// therefore gets its own ordering, chosen so the answer is one
/// `partition_point` away.
struct Index {
    /// Ascending `(end, Reverse(start))`.
    by_end: Vec<Span>,
    /// Ascending `(start, Reverse(end))`.
    by_start: Vec<Span>,
}

impl Index {
    fn new(mut spans: Vec<Span>) -> Self {
        spans.sort_unstable_by_key(|s| (s.end, Reverse(s.start)));
        spans.dedup();
        let mut by_start = spans.clone();
        by_start.sort_unstable_by_key(|s| (s.start, Reverse(s.end)));
        Self {
            by_end: spans,
            by_start,
        }
    }

    /// The node closing latest at or before `pos`, preferring the outermost
    /// (smallest start) when several end together.
    fn preceding(&self, pos: usize) -> Option<Span> {
        let i = self.by_end.partition_point(|s| s.end <= pos);
        i.checked_sub(1).map(|i| self.by_end[i])
    }

    /// The node opening earliest at or after `pos`, preferring the outermost
    /// (largest end) when several open together.
    fn following(&self, pos: usize) -> Option<Span> {
        let i = self.by_start.partition_point(|s| s.start < pos);
        self.by_start.get(i).copied()
    }
}

fn place_event<K: Clone + Default>(
    source: &str,
    event: &TriviaEvent<K>,
    index: &Index,
    map: &mut CommentMap<K>,
    opts: AttachOptions,
) {
    let lo = event.span.start;

    if event.trivia.is_blank() {
        if let Some(n) = index.following(lo) {
            map.entry(n).leading.push(event.trivia.clone());
        }
        return;
    }

    if opts.trailing_same_line {
        if let Some(p) = index.preceding(lo) {
            if same_line(source, p.end, lo) {
                map.entry(p).trailing.push(event.trivia.clone());
                return;
            }
        }
    }

    if let Some(n) = index.following(lo) {
        map.entry(n).leading.push(event.trivia.clone());
    } else {
        map.push_dangling(event.trivia.clone());
    }
}

/// True when no line break separates `from` from `to`.
fn same_line(source: &str, from: usize, to: usize) -> bool {
    source
        .as_bytes()
        .get(from..to)
        .is_some_and(|gap| !gap.contains(&b'\n'))
}

#[cfg(test)]
mod tests {
    use super::{attach, AttachOptions, Index};
    use crate::{span, Span, Trivia, TriviaEvent, TriviaTable};

    fn table(events: &[(Span, Trivia)]) -> TriviaTable {
        events
            .iter()
            .map(|(span, trivia)| TriviaEvent {
                span: *span,
                trivia: trivia.clone(),
            })
            .collect()
    }

    // Nested spans end out of order, so `preceding` cannot binary search the
    // `(start, end)` ordering directly. It still picks the latest-closing
    // node, and the outermost of the ones that close together.
    #[test]
    fn index_handles_nested_spans() {
        let outer = span(0, 20);
        let inner = span(4, 8);
        let ix = Index::new(vec![outer, inner, span(10, 14)]);

        assert_eq!(ix.preceding(9), Some(inner));
        assert_eq!(ix.preceding(20), Some(outer));
        assert_eq!(ix.preceding(0), None);
        assert_eq!(ix.following(0), Some(outer));
        assert_eq!(ix.following(9), Some(span(10, 14)));
        assert_eq!(ix.following(21), None);
    }

    // A comment on the same line as the node before it trails that node;
    // one on its own line leads the node after it.
    #[test]
    fn same_line_trails_other_lines_lead() {
        let source = "a; // one\n// two\nb;";
        let (a, b) = (span(0, 2), span(17, 19));
        let events = table(&[
            (span(3, 9), Trivia::line("// one")),
            (span(10, 16), Trivia::line("// two")),
        ]);

        let map = attach(source, &events, [a, b], AttachOptions::default());
        assert_eq!(map.trailing(a), [Trivia::line("// one")]);
        assert_eq!(map.leading(b), [Trivia::line("// two")]);
        assert!(map.dangling().is_empty());
    }

    // With `trailing_same_line` off, the same-line comment leads the next node
    // instead of trailing the previous one.
    #[test]
    fn trailing_same_line_disabled() {
        let source = "a; // one\nb;";
        let (a, b) = (span(0, 2), span(10, 12));
        let events = table(&[(span(3, 9), Trivia::line("// one"))]);

        let opts = AttachOptions {
            trailing_same_line: false,
        };
        let map = attach(source, &events, [a, b], opts);
        assert!(map.trailing(a).is_empty());
        assert_eq!(map.leading(b), [Trivia::line("// one")]);
    }

    // A comment past the last node has nowhere to attach and lands in the
    // dangling bucket, which is what keeps it from being dropped.
    #[test]
    fn trailing_comment_after_last_node_dangles() {
        let source = "a;\n\n// end";
        let a = span(0, 2);
        let events = table(&[(span(4, 10), Trivia::line("// end"))]);

        let map = attach(source, &events, [a], AttachOptions::default());
        assert!(map.leading(a).is_empty() && map.trailing(a).is_empty());
        assert_eq!(map.dangling(), [Trivia::line("// end")]);
    }

    // Blank lines always lead the following node, never trail the previous
    // one, so a gap before a node survives a reformat.
    #[test]
    fn blank_line_leads_following_node() {
        let source = "a;\n\nb;";
        let (a, b) = (span(0, 2), span(4, 6));
        let events = table(&[(span(2, 4), Trivia::BlankLine)]);

        let map = attach(source, &events, [a, b], AttachOptions::default());
        assert!(map.trailing(a).is_empty());
        assert_eq!(map.leading(b), [Trivia::BlankLine]);
    }

    // Every comment must come out exactly once across the three buckets:
    // dropping one silently is the failure mode that matters here.
    #[test]
    fn every_comment_is_placed_exactly_once() {
        let source = "a; // t\n// l\nb;\n// d";
        let (a, b) = (span(0, 2), span(13, 15));
        let events = table(&[
            (span(3, 7), Trivia::line("// t")),
            (span(8, 12), Trivia::line("// l")),
            (span(16, 20), Trivia::line("// d")),
        ]);

        let map = attach(source, &events, [a, b], AttachOptions::default());
        let placed = [a, b]
            .iter()
            .map(|s| map.leading(*s).len() + map.trailing(*s).len())
            .sum::<usize>()
            + map.dangling().len();
        assert_eq!(placed, events.len());
    }
}
