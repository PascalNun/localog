//! Cutting a recording without cutting the recording.
//!
//! A person trims the wait before a meeting starts, the goodbyes after it ends,
//! and the stretch in the middle where the call broke down and everybody talked
//! about nothing. None of that touches the file they recorded: the cuts are kept
//! as a description of what to keep, and the working audio is built from it.
//!
//! That is the same promise the application already makes about imported files,
//! for the same reason. Somebody who trims two minutes and then discovers the
//! decision was in them has lost nothing, and a recording of a professional
//! meeting is not a thing to overwrite because an interface offered a button.
//!
//! Editing happens before transcription, which is what keeps this simple: no
//! transcript exists yet whose timestamps would have to be reconciled with a
//! timeline that just got shorter.

// Not yet reachable from the interface: nothing stores edits or offers a place to
// make them. The model and the cutting land first because they are the part that
// can be proven exactly, and a review screen built on arithmetic nobody checked
// would be a review screen that quietly loses minutes of a meeting.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// A stretch of a recording, in milliseconds from its start.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Span {
    pub from_ms: u64,
    pub to_ms: u64,
}

impl Span {
    pub(crate) fn length_ms(self) -> u64 {
        self.to_ms.saturating_sub(self.from_ms)
    }

    fn is_empty(self) -> bool {
        self.to_ms <= self.from_ms
    }
}

/// What somebody decided to leave out.
///
/// Held as trims plus removals rather than as a list of kept pieces, because that
/// is how a person describes it — "start here, end there, and drop that bit in the
/// middle" — and because it is what the interface can show back to them as
/// separate, undoable decisions.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Edits {
    /// Where the meeting really starts. Zero means at the beginning.
    #[serde(default)]
    pub start_ms: u64,
    /// Where it really ends. Absent means at the end of the recording.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_ms: Option<u64>,
    /// Stretches dropped from the middle, in the order they were removed so the
    /// interface can list and undo them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<Span>,
}

impl Edits {
    pub(crate) fn is_untouched(&self) -> bool {
        self.start_ms == 0 && self.end_ms.is_none() && self.removed.is_empty()
    }
}

/// The stretches of the original that survive, in order and not overlapping.
///
/// This is the whole of the edit model: everything else — how long the result is,
/// where a moment in it came from, what audio to build — is read from this list.
/// Removals may overlap each other, sit outside the trim, run backwards or repeat,
/// because they arrive from a person dragging at a waveform, and all of that has to
/// produce one sensible answer rather than a special case.
pub(crate) fn kept(duration_ms: u64, edits: &Edits) -> Vec<Span> {
    let window = Span {
        from_ms: edits.start_ms.min(duration_ms),
        to_ms: edits.end_ms.unwrap_or(duration_ms).min(duration_ms),
    };
    if window.is_empty() {
        return Vec::new();
    }

    // Normalise the removals first: put each the right way round, clip it to the
    // window, drop the empty ones, then merge everything that touches or overlaps.
    let mut cuts: Vec<Span> = edits
        .removed
        .iter()
        .map(|span| Span {
            from_ms: span.from_ms.min(span.to_ms).max(window.from_ms),
            to_ms: span.to_ms.max(span.from_ms).min(window.to_ms),
        })
        .filter(|span| !span.is_empty())
        .collect();
    cuts.sort_by_key(|span| span.from_ms);
    let mut merged: Vec<Span> = Vec::with_capacity(cuts.len());
    for cut in cuts {
        match merged.last_mut() {
            Some(last) if cut.from_ms <= last.to_ms => last.to_ms = last.to_ms.max(cut.to_ms),
            _ => merged.push(cut),
        }
    }

    // What is left of the window once the merged cuts are taken out of it.
    let mut kept = Vec::with_capacity(merged.len() + 1);
    let mut at = window.from_ms;
    for cut in merged {
        if cut.from_ms > at {
            kept.push(Span {
                from_ms: at,
                to_ms: cut.from_ms,
            });
        }
        at = at.max(cut.to_ms);
    }
    if at < window.to_ms {
        kept.push(Span {
            from_ms: at,
            to_ms: window.to_ms,
        });
    }
    kept
}

/// How long the edited recording runs.
pub(crate) fn kept_duration_ms(duration_ms: u64, edits: &Edits) -> u64 {
    kept(duration_ms, edits)
        .iter()
        .map(|span| span.length_ms())
        .sum()
}

/// Where a moment in the edited recording came from in the original.
///
/// Needed to play the original at a point somebody clicked in the edited timeline,
/// and to say honestly which part of the recording a passage came from once the
/// middle of it has been dropped.
pub(crate) fn in_original(kept: &[Span], edited_ms: u64) -> Option<u64> {
    let mut passed = 0u64;
    for span in kept {
        let length = span.length_ms();
        if edited_ms < passed + length {
            return Some(span.from_ms + (edited_ms - passed));
        }
        passed += length;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(from_ms: u64, to_ms: u64) -> Span {
        Span { from_ms, to_ms }
    }

    #[test]
    fn an_untouched_recording_is_kept_whole() {
        let edits = Edits::default();
        assert!(edits.is_untouched());
        assert_eq!(kept(60_000, &edits), vec![span(0, 60_000)]);
        assert_eq!(kept_duration_ms(60_000, &edits), 60_000);
    }

    /// The case this exists for: the wait before, the goodbyes after.
    #[test]
    fn trimming_both_ends_keeps_the_middle() {
        let edits = Edits {
            start_ms: 2_140,
            end_ms: Some(3_930_000),
            ..Default::default()
        };
        assert_eq!(kept(4_472_000, &edits), vec![span(2_140, 3_930_000)]);
        assert_eq!(kept_duration_ms(4_472_000, &edits), 3_927_860);
    }

    #[test]
    fn a_removed_stretch_leaves_the_recording_in_two_pieces() {
        let edits = Edits {
            removed: vec![span(1_100_000, 1_205_000)],
            ..Default::default()
        };
        assert_eq!(
            kept(4_472_000, &edits),
            vec![span(0, 1_100_000), span(1_205_000, 4_472_000)]
        );
    }

    /// Removals arrive from somebody dragging at a waveform, so they overlap, touch,
    /// repeat and run backwards. All of it has to give one sensible answer.
    #[test]
    fn overlapping_and_backwards_removals_become_one_clean_cut() {
        let edits = Edits {
            removed: vec![
                span(30_000, 40_000),
                // Backwards, and overlapping the first.
                span(45_000, 35_000),
                // Exactly touching the end of the merged pair.
                span(45_000, 50_000),
                // A repeat of something already gone.
                span(31_000, 33_000),
            ],
            ..Default::default()
        };
        assert_eq!(
            kept(60_000, &edits),
            vec![span(0, 30_000), span(50_000, 60_000)]
        );
    }

    #[test]
    fn a_removal_outside_the_trim_changes_nothing() {
        let edits = Edits {
            start_ms: 10_000,
            end_ms: Some(20_000),
            removed: vec![span(0, 5_000), span(50_000, 55_000)],
        };
        assert_eq!(kept(60_000, &edits), vec![span(10_000, 20_000)]);
    }

    /// Half in and half out of the trim: only the part inside is a cut.
    #[test]
    fn a_removal_reaching_past_the_trim_is_clipped_to_it() {
        let edits = Edits {
            start_ms: 10_000,
            end_ms: Some(20_000),
            removed: vec![span(5_000, 12_000)],
        };
        assert_eq!(kept(60_000, &edits), vec![span(12_000, 20_000)]);
    }

    /// Nonsense must produce nothing rather than a panic or a recording that runs
    /// backwards: an end before a start, a trim past the recording, everything cut.
    #[test]
    fn edits_that_leave_nothing_leave_nothing() {
        let backwards = Edits {
            start_ms: 30_000,
            end_ms: Some(10_000),
            ..Default::default()
        };
        assert!(kept(60_000, &backwards).is_empty());
        assert_eq!(kept_duration_ms(60_000, &backwards), 0);

        let beyond = Edits {
            start_ms: 90_000,
            ..Default::default()
        };
        assert!(kept(60_000, &beyond).is_empty());

        let all_of_it = Edits {
            removed: vec![span(0, 60_000)],
            ..Default::default()
        };
        assert!(kept(60_000, &all_of_it).is_empty());
    }

    /// A trim beyond the end of the recording means the end of the recording, not
    /// silence appended to reach it.
    #[test]
    fn a_trim_past_the_end_stops_at_the_end() {
        let edits = Edits {
            end_ms: Some(90_000),
            ..Default::default()
        };
        assert_eq!(kept(60_000, &edits), vec![span(0, 60_000)]);
    }

    /// The order somebody made their cuts in must not change the result.
    #[test]
    fn the_order_edits_were_made_in_does_not_matter() {
        let one = Edits {
            start_ms: 5_000,
            end_ms: Some(55_000),
            removed: vec![span(40_000, 45_000), span(10_000, 20_000)],
        };
        let other = Edits {
            start_ms: 5_000,
            end_ms: Some(55_000),
            removed: vec![span(10_000, 20_000), span(40_000, 45_000)],
        };
        assert_eq!(kept(60_000, &one), kept(60_000, &other));
    }

    /// Clicking a moment in the edited timeline has to find the right moment in the
    /// recording, or playback lands somewhere nobody asked for.
    #[test]
    fn a_moment_in_the_edit_is_found_in_the_original() {
        let edits = Edits {
            start_ms: 10_000,
            removed: vec![span(20_000, 30_000)],
            ..Default::default()
        };
        let kept = kept(60_000, &edits);
        // The edit runs 10-20s then 30-60s, so it is 40 seconds long.
        assert_eq!(in_original(&kept, 0), Some(10_000));
        assert_eq!(in_original(&kept, 9_999), Some(19_999));
        // The first millisecond after the cut is the first after it in the source.
        assert_eq!(in_original(&kept, 10_000), Some(30_000));
        assert_eq!(in_original(&kept, 39_999), Some(59_999));
        // Past the end of the edit there is nothing to point at.
        assert_eq!(in_original(&kept, 40_000), None);
    }

    /// Edits are stored, so what is written must be readable and small: an
    /// untouched recording should not fill a database column with nothing.
    #[test]
    fn edits_are_stored_compactly_and_read_back() {
        let untouched = serde_json::to_string(&Edits::default()).expect("json");
        assert_eq!(untouched, r#"{"startMs":0}"#);
        let edits = Edits {
            start_ms: 2_140,
            end_ms: Some(3_930_000),
            removed: vec![span(1_100_000, 1_205_000)],
        };
        let written = serde_json::to_string(&edits).expect("json");
        let read: Edits = serde_json::from_str(&written).expect("readable");
        assert_eq!(read, edits);
    }
}
