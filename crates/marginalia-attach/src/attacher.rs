use marginalia::{Span, TriviaEvent, TriviaTable};

use crate::map::{CommentMap, HasSpan};

#[derive(Clone, Copy, Debug)]
pub struct AttachOptions {
    pub trailing_same_line: bool,
}

impl Default for AttachOptions {
    fn default() -> Self {
        Self {
            trailing_same_line: true,
        }
    }
}

pub fn attach<N: HasSpan>(
    source: &str,
    table: &TriviaTable,
    nodes: impl IntoIterator<Item = N>,
    opts: AttachOptions,
) -> CommentMap {
    let mut spans: Vec<Span> = nodes.into_iter().map(|n| n.span()).collect();
    spans.sort_unstable();
    spans.dedup();

    let mut map = CommentMap::new();
    for event in table.events() {
        place_event(source, event, &spans, &mut map, opts);
    }
    map
}

fn place_event(
    source: &str,
    event: &TriviaEvent,
    spans: &[Span],
    map: &mut CommentMap,
    opts: AttachOptions,
) {
    let lo = event.span.start;
    let prev = preceding_node(spans, lo);
    let next = following_node(spans, lo);

    if event.trivia.is_blank() {
        if let Some(n) = next {
            map.entry(n).leading.push(event.trivia.clone());
        }
        return;
    }

    if opts.trailing_same_line {
        if let Some(p) = prev {
            if same_line(source, p.end, lo) {
                map.entry(p).trailing.push(event.trivia.clone());
                return;
            }
        }
    }

    if let Some(n) = next {
        map.entry(n).leading.push(event.trivia.clone());
    } else if let Some(p) = prev {
        map.entry(p).trailing.push(event.trivia.clone());
    } else {
        map.push_dangling(event.trivia.clone());
    }
}

fn preceding_node(spans: &[Span], pos: usize) -> Option<Span> {
    spans
        .iter()
        .filter(|s| s.end <= pos)
        .max_by_key(|s| (s.end, std::cmp::Reverse(s.start)))
        .copied()
}

fn following_node(spans: &[Span], pos: usize) -> Option<Span> {
    spans
        .iter()
        .filter(|s| s.start >= pos)
        .min_by_key(|s| (s.start, std::cmp::Reverse(s.end)))
        .copied()
}

fn same_line(source: &str, from: usize, to: usize) -> bool {
    source
        .get(from..to)
        .is_some_and(|s| !s.bytes().any(|b| b == b'\n'))
}
