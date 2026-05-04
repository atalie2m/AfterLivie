use crate::{CommentEvent, CommentSource};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibilityQuery {
    pub time_ms: i64,
    pub comment_window_ms: i64,
    pub max_visible_comments: usize,
    pub global_offset_ms: i64,
}

pub fn effective_timestamp_ms(
    comment: &CommentEvent,
    source_offsets: &BTreeMap<String, i64>,
    global_offset_ms: i64,
) -> i64 {
    comment.timestamp_ms
        + source_offsets
            .get(&comment.source_id)
            .copied()
            .unwrap_or_default()
        + global_offset_ms
}

pub fn sort_comments(
    comments: &mut [CommentEvent],
    sources: &[CommentSource],
    global_offset_ms: i64,
) {
    let source_offsets = sources
        .iter()
        .map(|source| (source.source_id.clone(), source.offset_ms))
        .collect::<BTreeMap<_, _>>();
    let source_order = sources
        .iter()
        .enumerate()
        .map(|(index, source)| (source.source_id.clone(), index))
        .collect::<BTreeMap<_, _>>();

    comments.sort_by(|a, b| {
        effective_timestamp_ms(a, &source_offsets, global_offset_ms)
            .cmp(&effective_timestamp_ms(
                b,
                &source_offsets,
                global_offset_ms,
            ))
            .then_with(|| a.kind.priority().cmp(&b.kind.priority()))
            .then_with(|| {
                source_order
                    .get(&a.source_id)
                    .copied()
                    .unwrap_or(usize::MAX)
                    .cmp(
                        &source_order
                            .get(&b.source_id)
                            .copied()
                            .unwrap_or(usize::MAX),
                    )
            })
            .then_with(|| a.import_order.cmp(&b.import_order))
            .then_with(|| a.id.cmp(&b.id))
    });
}

pub fn visible_comments(
    comments: &[CommentEvent],
    sources: &[CommentSource],
    query: VisibilityQuery,
) -> Vec<CommentEvent> {
    let enabled_sources = sources
        .iter()
        .filter(|source| source.enabled)
        .map(|source| source.source_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let source_offsets = sources
        .iter()
        .map(|source| (source.source_id.clone(), source.offset_ms))
        .collect::<BTreeMap<_, _>>();
    let start = query.time_ms - query.comment_window_ms;

    let mut visible = comments
        .iter()
        .filter(|comment| enabled_sources.contains(comment.source_id.as_str()))
        .filter(|comment| {
            let effective =
                effective_timestamp_ms(comment, &source_offsets, query.global_offset_ms);
            effective <= query.time_ms && effective >= start
        })
        .cloned()
        .collect::<Vec<_>>();

    sort_comments(&mut visible, sources, query.global_offset_ms);
    if visible.len() > query.max_visible_comments {
        visible = visible[visible.len() - query.max_visible_comments..].to_vec();
    }
    visible
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AssetRef, CommentBody, CommentKind, ImportFingerprint, TimestampBasis};

    fn source_with_id(source_id: &str, offset_ms: i64, enabled: bool) -> CommentSource {
        CommentSource {
            source_id: source_id.into(),
            display_name: source_id.into(),
            platform: None,
            importer_id: "canonical-json".into(),
            original_file_ref: AssetRef {
                path: "comments.json".into(),
                hash: None,
            },
            enabled,
            offset_ms,
            timestamp_basis: TimestampBasis::RelativeToVideoStart,
            diagnostics_ref: None,
            visual_style: None,
            metadata: Default::default(),
            import_fingerprint: ImportFingerprint {
                algorithm: "sha256".into(),
                value: "abc".into(),
            },
        }
    }

    fn source(offset_ms: i64, enabled: bool) -> CommentSource {
        source_with_id("main", offset_ms, enabled)
    }

    fn comment_with_source(
        source_id: &str,
        id: &str,
        timestamp_ms: i64,
        import_order: u64,
    ) -> CommentEvent {
        CommentEvent {
            id: id.into(),
            source_id: source_id.into(),
            platform: None,
            timestamp_ms,
            original_timestamp: None,
            kind: CommentKind::Message,
            author: None,
            body: CommentBody { text: id.into() },
            style: None,
            badges: vec![],
            emotes: vec![],
            source: None,
            raw_ref: None,
            import_order,
        }
    }

    fn comment(id: &str, timestamp_ms: i64, import_order: u64) -> CommentEvent {
        comment_with_source("main", id, timestamp_ms, import_order)
    }

    #[test]
    fn visibility_respects_offsets_and_window() {
        let comments = vec![comment("a", 0, 0), comment("b", 10_000, 1)];
        let visible = visible_comments(
            &comments,
            &[source(1_000, true)],
            VisibilityQuery {
                time_ms: 11_000,
                comment_window_ms: 5_000,
                max_visible_comments: 10,
                global_offset_ms: 0,
            },
        );
        assert_eq!(
            visible.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
            ["b"]
        );
    }

    #[test]
    fn disabled_sources_are_hidden() {
        let comments = vec![comment("a", 0, 0)];
        let visible = visible_comments(
            &comments,
            &[source(0, false)],
            VisibilityQuery {
                time_ms: 1_000,
                comment_window_ms: 5_000,
                max_visible_comments: 10,
                global_offset_ms: 0,
            },
        );
        assert!(visible.is_empty());
    }

    #[test]
    fn source_priority_breaks_effective_timestamp_ties() {
        let mut comments = vec![
            comment_with_source("secondary", "b", 1_000, 0),
            comment_with_source("primary", "a", 1_000, 0),
        ];
        let sources = vec![
            source_with_id("primary", 0, true),
            source_with_id("secondary", 0, true),
        ];
        sort_comments(&mut comments, &sources, 0);
        assert_eq!(
            comments
                .iter()
                .map(|comment| comment.id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b"]
        );
    }

    #[test]
    fn source_offsets_are_non_destructive() {
        let comments = vec![comment_with_source("late", "a", 1_000, 0)];
        let sources = vec![source_with_id("late", 2_000, true)];
        let visible = visible_comments(
            &comments,
            &sources,
            VisibilityQuery {
                time_ms: 3_000,
                comment_window_ms: 500,
                max_visible_comments: 10,
                global_offset_ms: 0,
            },
        );
        assert_eq!(comments[0].timestamp_ms, 1_000);
        assert_eq!(visible[0].id, "a");
    }
}
