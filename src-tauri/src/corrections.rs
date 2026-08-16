//! Putting right a name the transcriber never heard correctly.
//!
//! A mis-heard proper noun is mis-heard consistently — the reference meeting says
//! `Klaster` forty times and `Cluster` never — so correcting the stem catches every
//! inflection and compound built on it at once. German compounds by concatenation,
//! which makes this unusually effective: repairing one stem repaired
//! `Klasterwohnung`, `Raumklaster` and `Einraumklaster` without anyone listing them.
//!
//! Measured on the reference meeting, eleven corrected stems put right eighty
//! occurrences, reaching roughly what re-transcribing with a larger model achieved and
//! taking milliseconds rather than seven minutes.
//!
//! No model is involved and none is wanted. The transcript is the evidence the
//! protocol is written from, so it changes only where somebody asked it to, and every
//! change can be shown and undone. What a model could not do here is more interesting
//! than what it could: it cannot be shown to the reader as a list of exact
//! substitutions, and it would quietly tidy prose along the way.

// Reachable from the evaluation harness and its tests, not yet from the application:
// the screen that offers these corrections is designed and not built. Wired up in the
// slice that adds it, at which point this attribute goes.
#![allow(dead_code)]

use crate::domain::TranscriptSegment;

/// One spelling somebody corrected, and what it should say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Correction {
    pub wrong: String,
    pub right: String,
}

impl Correction {
    /// The spellings this correction actually looks for.
    ///
    /// German writes the interior of a compound in lower case — `Raumklaster`, not
    /// `RaumKlaster` — so correcting `Klaster` alone leaves every compound it was
    /// meant to repair untouched. That was found by building this: the same
    /// measurement in a throwaway script had listed both forms by hand without
    /// anybody noticing it was two rules rather than one.
    ///
    /// Only for words that are capitalised rather than shouted. Lowering the first
    /// letter of `HOAI` would produce `hOAI`, and an abbreviation never appears
    /// inside a compound in that form anyway.
    fn forms(&self) -> Vec<(String, String)> {
        let mut forms = vec![(self.wrong.clone(), self.right.clone())];
        if is_capitalised(&self.wrong) && is_capitalised(&self.right) {
            forms.push((lower_initial(&self.wrong), lower_initial(&self.right)));
        }
        forms
    }
}

/// A capitalised word, as against an abbreviation in capitals.
fn is_capitalised(word: &str) -> bool {
    let mut glyphs = word.chars();
    let Some(first) = glyphs.next() else {
        return false;
    };
    first.is_uppercase() && glyphs.next().is_some_and(char::is_lowercase)
}

fn lower_initial(word: &str) -> String {
    let mut glyphs = word.chars();
    match glyphs.next() {
        Some(first) => first.to_lowercase().collect::<String>() + glyphs.as_str(),
        None => String::new(),
    }
}

/// One place a correction would apply, with enough of its sentence to judge it.
///
/// The judging matters. Some wrong spellings are also ordinary words: a participant
/// at the reference meeting is called Halde, and the transcriber wrote `Halle`,
/// which is the German for cross. Every occurrence there happened to be the person,
/// and that will not always hold — so this exists to be looked at before anything is
/// replaced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Match {
    pub segment_id: String,
    pub start_ms: u64,
    /// The sentence around the match, as the reader would hear it.
    pub context: String,
    /// Which correction produced this, as an index into the list given.
    pub correction: usize,
    /// Where in the segment's text this match begins, in bytes.
    pub at: usize,
    /// The exact text matched here, which may be the compound form.
    pub matched: String,
    /// What this occurrence would become.
    pub replacement: String,
}

/// Characters of surrounding text kept with a match. Enough to tell a name from a
/// noun, short enough to scan a list of them.
const CONTEXT: usize = 60;

/// Every place the corrections would apply, in the order somebody would read them.
///
/// Nothing is changed. This is what the person approves or declines.
pub(crate) fn preview(segments: &[TranscriptSegment], corrections: &[Correction]) -> Vec<Match> {
    let mut found = Vec::new();
    for segment in segments {
        for (index, correction) in corrections.iter().enumerate() {
            if correction.wrong.is_empty() {
                continue;
            }
            for (wrong, right) in correction.forms() {
                for (at, _) in segment.text.match_indices(&wrong) {
                    found.push(Match {
                        segment_id: segment.id.clone(),
                        start_ms: segment.start_ms,
                        context: around(&segment.text, at, wrong.len()),
                        correction: index,
                        at,
                        matched: wrong.clone(),
                        replacement: right.clone(),
                    });
                }
            }
        }
    }
    found.sort_by_key(|found| (found.start_ms, found.correction));
    found
}

/// Apply only the occurrences somebody kept.
///
/// This is the one the ambiguous cases need. A participant at the reference meeting
/// is called Halde and the transcriber wrote `Halle`, which is also the German for
/// cross; correcting every occurrence would eventually turn somebody's crucifix into
/// a structural engineer. Declining the correction whole is no use either, because
/// then the three that are the person stay wrong.
///
/// Applied last-first within each segment so that replacing one occurrence does not
/// move the ones not yet replaced.
pub(crate) fn apply_kept(segments: &mut [TranscriptSegment], kept: &[Match]) -> usize {
    let mut done = 0;
    for segment in segments.iter_mut() {
        let mut mine: Vec<&Match> = kept
            .iter()
            .filter(|found| found.segment_id == segment.id)
            .collect();
        mine.sort_by_key(|found| std::cmp::Reverse(found.at));
        for found in mine {
            let end = found.at + found.matched.len();
            if segment.text.get(found.at..end) != Some(found.matched.as_str()) {
                // The transcript moved under the review. Leaving it alone is the only
                // safe answer; replacing by position would corrupt a sentence.
                continue;
            }
            segment
                .text
                .replace_range(found.at..end, &found.replacement);
            done += 1;
        }
    }
    done
}

/// Apply every occurrence of each correction, returning how many each one changed.
///
/// The common case, and what somebody means when they correct a project's name: all
/// forty occurrences of it are the same mistake. Use [`apply_kept`] where a spelling
/// is also an ordinary word and the occurrences have to be told apart.
pub(crate) fn apply(segments: &mut [TranscriptSegment], corrections: &[Correction]) -> Vec<usize> {
    let mut counts = vec![0; corrections.len()];
    for segment in segments.iter_mut() {
        for (index, correction) in corrections.iter().enumerate() {
            if correction.wrong.is_empty() {
                continue;
            }
            for (wrong, right) in correction.forms() {
                if !segment.text.contains(&wrong) {
                    continue;
                }
                counts[index] += segment.text.matches(&wrong).count();
                segment.text = segment.text.replace(&wrong, &right);
            }
        }
    }
    counts
}

/// The text around a match, cut on character boundaries rather than bytes.
fn around(text: &str, at: usize, length: usize) -> String {
    let start = text[..at]
        .char_indices()
        .rev()
        .take(CONTEXT)
        .last()
        .map_or(0, |(index, _)| index);
    let after = at + length;
    let end = text[after..]
        .char_indices()
        .take(CONTEXT)
        .last()
        .map_or(after, |(index, glyph)| after + index + glyph.len_utf8());
    let mut context = String::new();
    if start > 0 {
        context.push('…');
    }
    context.push_str(text[start..end].trim());
    if end < text.len() {
        context.push('…');
    }
    context
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(id: &str, start_ms: u64, text: &str) -> TranscriptSegment {
        TranscriptSegment {
            id: id.into(),
            start_ms,
            end_ms: start_ms + 1000,
            speaker: "Speaker 1".into(),
            text: text.into(),
            needs_review: false,
            uncertain_words: Vec::new(),
        }
    }

    fn correction(wrong: &str, right: &str) -> Correction {
        Correction {
            wrong: wrong.into(),
            right: right.into(),
        }
    }

    /// The property the whole approach rests on: German compounds by concatenation,
    /// so correcting the stem corrects every word built on it, unlisted.
    #[test]
    fn correcting_a_stem_corrects_every_compound_built_on_it() {
        let mut segments = vec![
            segment("a", 0, "Die Klasterwohnung im Norden."),
            segment("b", 1000, "Ein Raumklaster und ein Einraumklaster."),
            segment("c", 2000, "Das Klaster selbst bleibt."),
        ];
        let counts = apply(&mut segments, &[correction("Klaster", "Cluster")]);

        assert_eq!(counts, vec![4]);
        assert_eq!(segments[0].text, "Die Clusterwohnung im Norden.");
        assert_eq!(segments[1].text, "Ein Raumcluster und ein Einraumcluster.");
        assert_eq!(segments[2].text, "Das Cluster selbst bleibt.");
    }

    /// An abbreviation must not be lowered into a compound form. `HOAI` would
    /// become `hOAI`, and no German compound carries an abbreviation that way.
    #[test]
    fn an_abbreviation_is_corrected_only_in_the_form_it_is_written() {
        let mut segments = vec![
            segment("a", 0, "Hoai liefert das System."),
            segment("b", 1000, "Die EFB-Grenze ist erreicht."),
        ];
        let counts = apply(
            &mut segments,
            &[correction("Hoai", "HOAI"), correction("EFB", "IFB")],
        );
        assert_eq!(counts, vec![1, 1]);
        assert_eq!(segments[0].text, "HOAI liefert das System.");
        assert_eq!(segments[1].text, "Die IFB-Grenze ist erreicht.");
        assert!(!segments[0].text.contains("hOAI"));
    }

    #[test]
    fn the_compound_form_is_offered_for_review_as_well() {
        let segments = vec![segment("a", 0, "Ein Raumklaster neben dem Klaster.")];
        let found = preview(&segments, &[correction("Klaster", "Cluster")]);
        assert_eq!(found.len(), 2, "{found:?}");
    }

    /// The case the whole review step exists for: one spelling, two meanings.
    #[test]
    fn only_the_occurrences_somebody_kept_are_changed() {
        let mut segments = vec![
            segment("a", 5000, "Jetzt fehlen noch Herr Halle und Frau Bauphysik."),
            segment("b", 9000, "Am Halle vorbei geht es zur Baustelle."),
        ];
        let found = preview(&segments, &[correction("Halle", "Halde")]);
        assert_eq!(found.len(), 2);

        // The person is kept; the crucifix is not.
        let kept: Vec<Match> = found
            .into_iter()
            .filter(|found| found.context.contains("Herr"))
            .collect();
        assert_eq!(apply_kept(&mut segments, &kept), 1);

        assert_eq!(
            segments[0].text,
            "Jetzt fehlen noch Herr Halde und Frau Bauphysik."
        );
        assert_eq!(segments[1].text, "Am Halle vorbei geht es zur Baustelle.");
    }

    /// Replacing one occurrence must not move the ones not yet replaced.
    #[test]
    fn several_kept_occurrences_in_one_segment_all_land() {
        let mut segments = vec![segment("a", 0, "Klaster, dann Klaster, dann Klaster.")];
        let found = preview(&segments, &[correction("Klaster", "Cluster")]);
        assert_eq!(apply_kept(&mut segments, &found), 3);
        assert_eq!(segments[0].text, "Cluster, dann Cluster, dann Cluster.");
    }

    /// A replacement longer than what it replaces still leaves the earlier matches
    /// where the review found them.
    #[test]
    fn a_longer_replacement_does_not_disturb_earlier_matches() {
        let mut segments = vec![segment("a", 0, "Hoai und Hoai.")];
        let found = preview(&segments, &[correction("Hoai", "HOAI GmbH")]);
        assert_eq!(apply_kept(&mut segments, &found), 2);
        assert_eq!(segments[0].text, "HOAI GmbH und HOAI GmbH.");
    }

    /// If the transcript was edited after the review was shown, the stale match is
    /// skipped rather than applied to whatever now sits at that position.
    #[test]
    fn a_match_that_no_longer_matches_is_left_alone() {
        let segments = vec![segment("a", 0, "Das Klaster im Norden.")];
        let found = preview(&segments, &[correction("Klaster", "Cluster")]);

        let mut moved = vec![segment("a", 0, "Ganz andere Worte hier drin.")];
        assert_eq!(apply_kept(&mut moved, &found), 0);
        assert_eq!(moved[0].text, "Ganz andere Worte hier drin.");
    }

    #[test]
    fn a_correction_that_matches_nothing_changes_nothing() {
        let mut segments = vec![segment("a", 0, "Die Fassade ist ruhig.")];
        let counts = apply(&mut segments, &[correction("Klaster", "Cluster")]);
        assert_eq!(counts, vec![0]);
        assert_eq!(segments[0].text, "Die Fassade ist ruhig.");
    }

    /// The reason nothing is applied without being looked at first.
    #[test]
    fn every_match_is_offered_with_enough_sentence_to_judge_it() {
        let segments = vec![
            segment(
                "a",
                5000,
                "Jetzt fehlen noch Herr Halle und Frau Bauphysik im Termin.",
            ),
            segment("b", 9000, "Am Halle vorbei geht es zur Baustelle."),
        ];
        let found = preview(&segments, &[correction("Halle", "Halde")]);

        assert_eq!(found.len(), 2);
        assert!(found[0].context.contains("Herr Halle"), "{:?}", found[0]);
        assert!(
            found[1].context.contains("Am Halle vorbei"),
            "{:?}",
            found[1]
        );
        // One is the participant and one is a crucifix; the person decides, not this.
        assert_eq!(found[0].start_ms, 5000);
        assert_eq!(found[1].start_ms, 9000);
    }

    #[test]
    fn matches_are_ordered_as_the_recording_runs() {
        let segments = vec![
            segment("late", 9000, "Hoai liefert."),
            segment("early", 1000, "Klaster im Norden."),
        ];
        let found = preview(
            &segments,
            &[
                correction("Hoai", "HOAI"),
                correction("Klaster", "Cluster"),
            ],
        );
        assert_eq!(found[0].start_ms, 1000);
        assert_eq!(found[1].start_ms, 9000);
    }

    #[test]
    fn several_occurrences_in_one_segment_are_each_offered() {
        let segments = vec![segment(
            "a",
            0,
            "Klaster hier, Klaster dort, Klaster überall.",
        )];
        assert_eq!(
            preview(&segments, &[correction("Klaster", "Cluster")]).len(),
            3
        );
    }

    /// Cutting a context window by bytes would split a multi-byte character and
    /// panic. German supplies plenty of them.
    #[test]
    fn context_is_cut_on_characters_not_bytes() {
        let long = format!("{} Klaster {}", "ä".repeat(80), "ö".repeat(80));
        let segments = vec![segment("a", 0, &long)];
        let found = preview(&segments, &[correction("Klaster", "Cluster")]);
        assert_eq!(found.len(), 1);
        assert!(found[0].context.contains("Klaster"));
        assert!(found[0].context.starts_with('…'));
        assert!(found[0].context.ends_with('…'));
    }

    #[test]
    fn an_empty_correction_is_ignored_rather_than_matching_everywhere() {
        let mut segments = vec![segment("a", 0, "Die Fassade.")];
        assert!(preview(&segments, &[correction("", "X")]).is_empty());
        assert_eq!(apply(&mut segments, &[correction("", "X")]), vec![0]);
        assert_eq!(segments[0].text, "Die Fassade.");
    }
}
