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

    /// The safety net: a subject the pass missed must be visible, not silent.
    #[test]
    fn segments_no_subject_claimed_are_reported() {
        let topics = vec![topic("Kosten", &[0, 1]), topic("Termine", &[3])];
        assert_eq!(unclaimed(6, &topics), vec![2, 4, 5]);
        assert!(unclaimed(4, &[topic("Alles", &[0, 1, 2, 3])]).is_empty());
    }
}
