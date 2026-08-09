//! Dividing a meeting into the subjects it discussed.
//!
//! A protocol is written subject by subject, and a subject rarely arrives in one
//! piece: a meeting returns to the facade three times over eighty minutes. Finding
//! those subjects, and which parts of the transcript belong to each, is the step
//! that makes everything after it small — the writing then sees a few thousand
//! characters about one thing rather than the whole meeting at once.
//!
//! Everything here is plain code. The model is asked only which subjects a passage
//! discusses; the windowing, the mapping back to real segments, the merging and the
//! accounting for what was left out are all decided here, where they can be tested
//! without a runtime and give the same answer every time.

use std::ops::Range;

/// A subject the meeting discussed, and where it was discussed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Topic {
    pub title: String,
    /// Indices into the transcript, ascending and without repeats.
    pub segments: Vec<usize>,
}

/// Windows of the transcript to read, overlapping so that a subject straddling a
/// boundary is seen whole by at least one of them.
///
/// Sized in characters rather than segments because segments vary from a word to a
/// paragraph. A window that ends mid-subject is the failure this overlap exists to
/// prevent, and paying for it twice is far cheaper than losing a topic.
pub(crate) fn plan_windows(
    segment_chars: &[usize],
    window_chars: usize,
    overlap_segments: usize,
) -> Vec<Range<usize>> {
    if segment_chars.is_empty() {
        return Vec::new();
    }
    let mut windows = Vec::new();
    let mut start = 0;
    while start < segment_chars.len() {
        let mut used = 0;
        let mut end = start;
        while end < segment_chars.len() && (used == 0 || used + segment_chars[end] <= window_chars)
        {
            used += segment_chars[end];
            end += 1;
        }
        windows.push(start..end);
        if end >= segment_chars.len() {
            break;
        }
        // Step back so the next window re-reads the tail of this one.
        start = end.saturating_sub(overlap_segments).max(start + 1);
    }
    windows
}

/// Turn the numbers a model returned for one window into transcript indices.
///
/// The model is shown a window numbered from one, because a small model reproduces
/// "14" reliably and an identifier like `segment-a1b2c3d4-0087` badly. Anything
/// outside the window it was shown is discarded rather than clamped: a number that
/// cannot be right is a mistake, and guessing which segment was meant would put
/// words in a speaker's mouth.
pub(crate) fn resolve(window: &Range<usize>, local: &[i64]) -> Vec<usize> {
    let mut resolved: Vec<usize> = local
        .iter()
        .filter(|number| **number >= 1)
        .map(|number| window.start + (*number as usize) - 1)
        .filter(|index| window.contains(index))
        .collect();
    resolved.sort_unstable();
    resolved.dedup();
    resolved
}

/// Turn window numbers into transcript indices when the window covers a chosen
/// subset of the transcript rather than a stretch of it.
///
/// The pass runs again over whatever no subject claimed, and those leftovers are
/// scattered through the meeting. The model still sees a passage numbered from
/// one; this maps back through the selection it was built from.
pub(crate) fn resolve_within(
    selection: &[usize],
    window: &Range<usize>,
    local: &[i64],
) -> Vec<usize> {
    let mut resolved: Vec<usize> = local
        .iter()
        .filter(|number| **number >= 1)
        .map(|number| window.start + (*number as usize) - 1)
        .filter(|position| window.contains(position))
        .filter_map(|position| selection.get(position).copied())
        .collect();
    resolved.sort_unstable();
    resolved.dedup();
    resolved
}

/// Combine what the windows found into one list of subjects.
///
/// Overlapping windows mean the same subject is reported twice, so topics naming
/// the same thing are joined. Matching is on the title, normalised: two windows
/// that both saw a subject describe it in the same words far more often than not,
/// and merging by overlapping segments instead would join two subjects that merely
/// happened to be discussed together.
pub(crate) fn merge(found: Vec<Topic>) -> Vec<Topic> {
    let mut merged: Vec<Topic> = Vec::new();
    for topic in found {
        let key = normalise(&topic.title);
        if key.is_empty() {
            continue;
        }
        match merged
            .iter_mut()
            .find(|existing| normalise(&existing.title) == key)
        {
            Some(existing) => {
                existing.segments.extend(topic.segments);
                existing.segments.sort_unstable();
                existing.segments.dedup();
            }
            None => merged.push(topic),
        }
    }
    // Read in the order the meeting took them.
    merged.sort_by_key(|topic| topic.segments.first().copied().unwrap_or(usize::MAX));
    merged
}

/// Apply a grouping of subjects, given as one-based positions into `topics`.
///
/// Merging by title alone joins only what two windows worded identically, and a
/// meeting that returns to the facade six times produces six facade subjects with
/// six different names. Deciding which of those name the same thing is judgement,
/// so it is asked of the model — but only that, and only over a list of titles.
/// Which segments then belong where, and what happens to a subject the grouping
/// forgot, are settled here.
///
/// A subject named in no group survives on its own. A position that does not exist
/// is ignored. Neither is treated as a reason to lose a subject, because losing one
/// silently is the failure this whole pass is built to avoid.
pub(crate) fn group(topics: Vec<Topic>, groups: &[(String, Vec<i64>)]) -> Vec<Topic> {
    let mut taken = vec![false; topics.len()];
    let mut grouped: Vec<Topic> = Vec::new();
    for (title, positions) in groups {
        let mut segments = Vec::new();
        for position in positions {
            let Some(index) = usize::try_from(*position - 1)
                .ok()
                .filter(|i| *i < topics.len())
            else {
                continue;
            };
            if std::mem::replace(&mut taken[index], true) {
                continue;
            }
            segments.extend(topics[index].segments.iter().copied());
        }
        if segments.is_empty() {
            continue;
        }
        segments.sort_unstable();
        segments.dedup();
        grouped.push(Topic {
            title: title.trim().to_string(),
            segments,
        });
    }
    // Anything the grouping did not mention keeps its own place.
    for (index, topic) in topics.into_iter().enumerate() {
        if !taken[index] {
            grouped.push(topic);
        }
    }
    grouped.sort_by_key(|topic| topic.segments.first().copied().unwrap_or(usize::MAX));
    grouped
}

/// Fold subjects too small to be sections into the ones around them.
///
/// A window that mentions a thing once reports it as a subject, and it is right to
/// — it was discussed. But a protocol section built from one segment is a sentence
/// wearing a heading, and a document of those reads as a transcript with titles.
///
/// The small subject is not discarded. Its segments join the nearest subject in
/// time, which is almost always the discussion it was a remark within, so the
/// material survives and only the heading goes. When there is nothing to join, the
/// subject is kept as it is: losing it would be worse than a short section.
pub(crate) fn absorb_small(topics: Vec<Topic>, minimum: usize) -> Vec<Topic> {
    let (mut kept, small): (Vec<Topic>, Vec<Topic>) = topics
        .into_iter()
        .partition(|topic| topic.segments.len() >= minimum);
    if kept.is_empty() {
        return small;
    }
    for topic in small {
        let Some(anchor) = topic.segments.first().copied() else {
            continue;
        };
        let nearest = kept
            .iter_mut()
            .min_by_key(|candidate| {
                candidate
                    .segments
                    .iter()
                    .map(|index| index.abs_diff(anchor))
                    .min()
                    .unwrap_or(usize::MAX)
            })
            .expect("at least one subject remains");
        nearest.segments.extend(topic.segments);
        nearest.segments.sort_unstable();
        nearest.segments.dedup();
    }
    kept.sort_by_key(|topic| topic.segments.first().copied().unwrap_or(usize::MAX));
    kept
}

/// Segments no subject claimed.
///
/// This is the whole safety net. A subject the pass failed to name would otherwise
/// vanish silently, and a reader cannot notice an absence in a document that reads
/// perfectly well without it. Here the leftovers are visible, so they can be
/// written up as further points rather than lost, and counted so the reader is told.
pub(crate) fn unclaimed(segment_count: usize, topics: &[Topic]) -> Vec<usize> {
    let mut claimed = vec![false; segment_count];
    for index in topics.iter().flat_map(|topic| topic.segments.iter()) {
        if let Some(slot) = claimed.get_mut(*index) {
            *slot = true;
        }
    }
    claimed
        .iter()
        .enumerate()
        .filter(|(_, taken)| !**taken)
        .map(|(index, _)| index)
        .collect()
}

/// Compare titles without punctuation, case or spacing getting in the way.
fn normalise(title: &str) -> String {
    title
        .chars()
        .filter(|character| character.is_alphanumeric() || character.is_whitespace())
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn topic(title: &str, segments: &[usize]) -> Topic {
        Topic {
            title: title.to_string(),
            segments: segments.to_vec(),
        }
    }

    #[test]
    fn windows_overlap_so_a_subject_at_a_boundary_is_seen_whole() {
        let sizes = vec![100; 10];
        let windows = plan_windows(&sizes, 300, 1);
        assert!(windows.len() > 1, "ten segments should not fit one window");
        for pair in windows.windows(2) {
            assert!(
                pair[1].start < pair[0].end,
                "windows {:?} and {:?} do not overlap",
                pair[0],
                pair[1]
            );
        }
        assert_eq!(
            windows.last().unwrap().end,
            10,
            "the last window must reach the end"
        );
    }

    /// A segment longer than the whole window still has to be read.
    #[test]
    fn a_segment_larger_than_the_window_is_still_placed_in_one() {
        let windows = plan_windows(&[5_000, 100], 1_000, 1);
        assert_eq!(windows.first().unwrap().start, 0);
        assert!(windows.iter().any(|window| window.contains(&0)));
        assert!(windows.iter().any(|window| window.contains(&1)));
    }

    #[test]
    fn no_windows_for_an_empty_transcript() {
        assert!(plan_windows(&[], 1_000, 2).is_empty());
    }

    #[test]
    fn window_numbers_become_transcript_indices() {
        // The window covers segments 10..15, so its "1" is segment 10.
        assert_eq!(resolve(&(10..15), &[1, 3, 5]), vec![10, 12, 14]);
    }

    /// A number the model could not have meant is dropped, never guessed at.
    #[test]
    fn impossible_numbers_are_discarded_rather_than_clamped() {
        assert_eq!(resolve(&(10..15), &[0, -4, 6, 99]), Vec::<usize>::new());
        assert_eq!(resolve(&(10..15), &[2, 2, 1]), vec![10, 11]);
    }

    #[test]
    fn the_same_subject_seen_by_two_windows_becomes_one() {
        let merged = merge(vec![
            topic("Fassade und Fenster", &[4, 5]),
            topic("Erschließung", &[1, 2]),
            topic("fassade und fenster.", &[5, 6]),
        ]);
        assert_eq!(merged.len(), 2);
        // Ordered as the meeting took them.
        assert_eq!(merged[0].title, "Erschließung");
        assert_eq!(merged[1].segments, vec![4, 5, 6]);
    }

    #[test]
    fn subjects_discussed_together_are_not_merged() {
        let merged = merge(vec![topic("Kosten", &[3, 4]), topic("Termine", &[3, 4])]);
        assert_eq!(merged.len(), 2, "shared segments are not shared subjects");
    }

    #[test]
    fn grouping_joins_the_subjects_a_reader_would_call_one() {
        let topics = vec![
            topic("Fassade und Wettbewerb", &[4, 5]),
            topic("Erschließung", &[1]),
            topic("Fassadengestaltung und Elemente", &[9]),
        ];
        let grouped = group(topics, &[("Fassade".to_string(), vec![1, 3])]);
        assert_eq!(grouped.len(), 2);
        // Untouched subjects keep their place, and order follows the meeting.
        assert_eq!(grouped[0].title, "Erschließung");
        assert_eq!(grouped[1].title, "Fassade");
        assert_eq!(grouped[1].segments, vec![4, 5, 9]);
    }

    /// A grouping that forgets a subject, names one twice, or points at nothing
    /// must not cost a subject. Losing one silently is the failure this pass exists
    /// to prevent, so every defect here fails towards keeping things.
    #[test]
    fn a_faulty_grouping_never_loses_a_subject() {
        let topics = vec![
            topic("Kosten", &[1]),
            topic("Termine", &[2]),
            topic("Fassade", &[3]),
        ];
        let grouped = group(
            topics,
            &[
                ("Kosten und Termine".to_string(), vec![1, 2, 2, 99, -1]),
                ("Nichts".to_string(), vec![404]),
            ],
        );
        assert_eq!(grouped.len(), 2, "the forgotten subject survives");
        assert_eq!(grouped[0].segments, vec![1, 2], "a repeat is counted once");
        assert_eq!(grouped[1].title, "Fassade");
    }

    #[test]
    fn a_subject_too_small_to_be_a_section_joins_the_nearest_one() {
        let topics = vec![
            topic("Erschließung", &[0, 1, 2, 3]),
            topic("Nebenbemerkung", &[4]),
            topic("Fassade", &[20, 21, 22, 23]),
        ];
        let folded = absorb_small(topics, 3);
        assert_eq!(folded.len(), 2);
        // The remark belonged to the discussion it interrupted, and its material
        // survives even though its heading does not.
        assert_eq!(folded[0].segments, vec![0, 1, 2, 3, 4]);
        assert_eq!(folded[1].title, "Fassade");
    }

    /// Folding must never be able to empty the meeting.
    #[test]
    fn nothing_is_folded_away_when_everything_is_small() {
        let topics = vec![topic("Eins", &[0]), topic("Zwei", &[5])];
        let folded = absorb_small(topics, 4);
        assert_eq!(folded.len(), 2, "keeping a short section beats losing it");
    }

    /// Reading the leftovers again means the passage in front of the model is no
    /// longer a stretch of the meeting, so its numbers map back through the
    /// selection rather than through a range.
    #[test]
    fn window_numbers_map_back_through_a_selection() {
        let leftovers = [7, 8, 40, 41, 90];
        assert_eq!(
            resolve_within(&leftovers, &(0..5), &[1, 3, 5]),
            vec![7, 40, 90]
        );
        // The second window of that selection starts at its third entry.
        assert_eq!(resolve_within(&leftovers, &(2..5), &[1, 2]), vec![40, 41]);
        assert!(resolve_within(&leftovers, &(0..5), &[99, 0]).is_empty());
    }

    /// The safety net: a subject the pass missed must be visible, not silent.
    #[test]
    fn segments_no_subject_claimed_are_reported() {
        let topics = vec![topic("Kosten", &[0, 1]), topic("Termine", &[3])];
        assert_eq!(unclaimed(6, &topics), vec![2, 4, 5]);
        assert!(unclaimed(4, &[topic("Alles", &[0, 1, 2, 3])]).is_empty());
    }
}
