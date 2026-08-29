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

use crate::domain::TranscriptSegment;
use std::collections::HashMap;

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

/// A word the transcriber never got right, offered as a possible name.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Candidate {
    pub heard: String,
    pub occurrences: usize,
    /// One place it appears, because a mis-heard word is unfamiliar by definition
    /// and cannot be recognised without its sentence.
    pub context: String,
    /// Whether the model writing the protocol said it did not recognise this.
    ///
    /// Worth telling the reader apart from the rest, because these are the ones that
    /// *look* right. A garbled word explains itself on sight; a plain wrong spelling
    /// the transcriber was confident about needs somebody to be told why it is being
    /// offered at all.
    pub questioned: bool,
}

/// How many candidates are worth offering.
///
/// A short list somebody finishes beats a long one they abandon, and anything missed
/// is caught at the next meeting. Measured on an eighty-minute meeting the filter
/// below yields about six, so this is a guard against an unusual transcript rather
/// than a limit that normally bites.
const MOST_CANDIDATES: usize = 12;

/// Occurrences before a word is worth offering. One is usually a stumble.
const LEAST_OCCURRENCES: usize = 2;

/// Letters before a word is worth offering. Shorter ones are particles.
const LEAST_LETTERS: usize = 4;

/// The words the transcriber was never sure of, most frequent first.
///
/// The filter is deliberately strict: a word qualifies only if **every** time it was
/// heard, the transcriber was unsure of it. A word it usually gets right and fumbles
/// once is a stumble; a word it never gets right is a word it does not know, which is
/// what a name is.
///
/// Measured on the reference meeting this turns 1,941 distinct words into six, of
/// which two or three are the names that matter. The same transcript flags 322 of its
/// 675 segments as containing something uncertain, which is why the count that panel
/// used to show was not a task anybody started.
///
/// It cannot catch a name the transcriber was confident about and wrong — nothing
/// reading confidence can. Those turn up in the protocol model's own notes instead.
pub(crate) fn name_candidates(
    segments: &[TranscriptSegment],
    protocol_markdown: &str,
) -> Vec<Candidate> {
    let mut heard: HashMap<String, usize> = HashMap::new();
    let mut unsure: HashMap<String, usize> = HashMap::new();
    let mut first_seen: HashMap<String, (usize, usize)> = HashMap::new();

    for (index, segment) in segments.iter().enumerate() {
        for (at, word) in words(&segment.text) {
            let key = word.to_lowercase();
            *heard.entry(key.clone()).or_default() += 1;
            first_seen.entry(key).or_insert((index, at));
        }
        for word in &segment.uncertain_words {
            let trimmed = word.trim_matches(|glyph: char| !glyph.is_alphabetic());
            if trimmed.chars().count() >= LEAST_LETTERS {
                *unsure.entry(trimmed.to_lowercase()).or_default() += 1;
            }
        }
    }

    let mut candidates: Vec<Candidate> = unsure
        .iter()
        .filter(|(word, doubted)| {
            let total = heard.get(word.as_str()).copied().unwrap_or(**doubted);
            total >= LEAST_OCCURRENCES && **doubted >= total
        })
        .filter_map(|(word, _)| {
            let &(index, at) = first_seen.get(word)?;
            let segment = segments.get(index)?;
            let spelled = segment.text.get(at..)?.split_whitespace().next()?;
            Some(Candidate {
                heard: spelled
                    .trim_matches(|glyph: char| !glyph.is_alphabetic())
                    .to_string(),
                occurrences: heard.get(word).copied().unwrap_or(0),
                context: around(&segment.text, at, spelled.len()),
                questioned: false,
            })
        })
        .filter(|candidate| candidate.heard.chars().count() >= LEAST_LETTERS)
        .collect();

    // Most first, then alphabetically so the list does not reshuffle between runs.
    candidates.sort_by(|left, right| {
        right
            .occurrences
            .cmp(&left.occurrences)
            .then_with(|| left.heard.cmp(&right.heard))
    });

    // The names the protocol model questioned go in front, and before the truncation.
    //
    // Not because there are many — four drafts in thirteen carried one — but because
    // they are the only ones nothing else can find, and a list cut to twelve must not
    // drop the one entry that had no other way of getting here.
    for name in names_the_protocol_questioned(protocol_markdown) {
        if let Some(already) = candidates
            .iter_mut()
            .find(|candidate| candidate.heard.eq_ignore_ascii_case(&name))
        {
            already.questioned = true;
            continue;
        }
        // Only a word the transcript actually contains. A model quoting something it
        // invented must not become a spelling somebody is invited to correct, and
        // without a place in the transcript there would be nothing to show them.
        let Some(placed) = place_in_transcript(segments, &name) else {
            continue;
        };
        candidates.insert(0, placed);
    }

    candidates.truncate(MOST_CANDIDATES);
    candidates
}

/// A name, counted and placed in the transcript that used it.
///
/// `None` when the transcript never says it, which is the filter that keeps a model's
/// invention out of a list of spellings to correct.
fn place_in_transcript(segments: &[TranscriptSegment], name: &str) -> Option<Candidate> {
    let wanted = name.to_lowercase();
    let mut occurrences = 0;
    let mut first: Option<(usize, usize, usize)> = None;
    for (index, segment) in segments.iter().enumerate() {
        for (at, word) in words(&segment.text) {
            if word.to_lowercase() == wanted {
                occurrences += 1;
                first.get_or_insert((index, at, word.len()));
            }
        }
    }
    let (index, at, length) = first?;
    let segment = segments.get(index)?;
    Some(Candidate {
        heard: name.to_string(),
        occurrences,
        context: around(&segment.text, at, length),
        questioned: true,
    })
}

/// A word the deterministic pass could not settle, with the sentence it sits in.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Unsettled {
    /// Where it is, so a proposal can be placed back without searching again.
    pub segment_id: String,
    /// The word as the transcript spells it.
    pub heard: String,
    /// The sentence around it, and nothing more.
    ///
    /// The whole segment, which averages about seven seconds of speech. Deciding
    /// whether `Halle` is a surname or the word for a cross needs one sentence, not
    /// eighty minutes — this is the one stage in the pipeline where a long context is
    /// provably unnecessary.
    pub passage: String,
}

/// How many unsettled words are worth asking about.
///
/// Measured on the reference meeting, substitution left about three genuinely wrong;
/// this is a guard against an unusual transcript rather than a limit that normally
/// bites. It also bounds the cost: one small request each.
const MOST_UNSETTLED: usize = 8;

/// The uncertain words substitution cannot reach.
///
/// What is left after the candidate list is a different kind of problem, not a
/// smaller amount of the same one. A candidate is a word mis-heard the *same* way
/// every time, which is what makes correcting one stem repair forty occurrences. What
/// remains is the word whose mis-hearing varied — no consistent stem to catch, one
/// occurrence each — and there is nothing deterministic left to do with it.
///
/// This function is worth having even if nothing is ever built on top of it. The plan
/// gates the model pass on a measurement nobody had made: whether the leftover is
/// consistently three-ish words, in which case a person fixes them faster than a
/// suggestion could be built. Running this on a real meeting is that measurement.
pub(crate) fn unsettled(
    segments: &[TranscriptSegment],
    already_offered: &[Candidate],
) -> Vec<Unsettled> {
    let offered: Vec<String> = already_offered
        .iter()
        .map(|candidate| candidate.heard.to_lowercase())
        .collect();
    let mut seen: Vec<String> = Vec::new();
    let mut left: Vec<Unsettled> = Vec::new();

    for segment in segments {
        for word in &segment.uncertain_words {
            let heard = word.trim_matches(|glyph: char| !glyph.is_alphabetic());
            if heard.chars().count() < LEAST_LETTERS {
                continue;
            }
            let key = heard.to_lowercase();
            // Anything the candidate list already offers is settled by substitution,
            // which is exact, instant and undoable. A model has nothing to add there
            // and every reason not to be asked.
            if offered.contains(&key) || seen.contains(&key) {
                continue;
            }
            seen.push(key);
            left.push(Unsettled {
                segment_id: segment.id.clone(),
                heard: heard.to_string(),
                passage: segment.text.clone(),
            });
            if left.len() >= MOST_UNSETTLED {
                return left;
            }
        }
    }
    left
}

/// Names the protocol model itself questioned, taken from what it already wrote.
///
/// A transcriber's confidence cannot flag an error it is confident about, and that is
/// the dangerous class: a plain wrong spelling reaches a client's inbox looking
/// correct, while the catastrophically mangled form gets flagged and fixed. The model
/// writing the protocol does sometimes notice, unprompted. Found by accident at the
/// foot of a draft, about the client's own name:
///
/// ```text
/// [Note: The term "Klinker-Nord" is used in the source text; it is unclear if
/// this refers to a specific project name or location.]
/// ```
///
/// whisper had flagged the mangled `Lärgedorf-Bildes-Fropette-Reit` and not this,
/// because it was confident about it. So this is the only one of the three sources of
/// candidates that can see this class at all, and it costs nothing: the notes are
/// already written.
///
/// **Read by punctuation rather than by words**, for the reason the actions-table
/// check is read that way. A bracket and a quotation mark look the same in German; a
/// pattern matching "it is unclear" would find nothing in the language this product
/// is actually for. What is looked for is a bracketed aside holding a quoted term —
/// which is the shape of the note, in any language the model writes it in.
///
/// Weak on its own, and that is fine. Four drafts in thirteen carried such a note,
/// each naming one thing.
pub(crate) fn names_the_protocol_questioned(markdown: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for aside in bracketed_asides(markdown) {
        for quoted in quoted_runs(aside) {
            let term = quoted
                .trim()
                .trim_matches(|glyph: char| !glyph.is_alphanumeric());
            if !looks_like_a_name(term) {
                continue;
            }
            if !found.iter().any(|kept| kept.eq_ignore_ascii_case(term)) {
                found.push(term.to_string());
            }
        }
    }
    found
}

/// The `[...]` spans, minus the ones that are markdown links.
///
/// A link is `[text](url)`, so a bracket followed by a parenthesis is not an aside.
/// Nesting is not handled and does not need to be: a note about a name does not
/// contain another bracket, and the worst case of getting it wrong is one candidate
/// too many in a list somebody is reading anyway.
fn bracketed_asides(markdown: &str) -> Vec<&str> {
    let bytes = markdown.as_bytes();
    let mut asides = Vec::new();
    let mut opened: Option<usize> = None;
    for (at, glyph) in markdown.char_indices() {
        match glyph {
            '[' => opened = Some(at + 1),
            ']' => {
                if let Some(from) = opened.take() {
                    let is_link = bytes.get(at + 1) == Some(&b'(');
                    if !is_link && at > from {
                        asides.push(&markdown[from..at]);
                    }
                }
            }
            _ => {}
        }
    }
    asides
}

/// What sits between a pair of quotation marks, in the several shapes they take.
///
/// German writes `„so"` and sometimes `»so«`; English writes `"so"` or `“so”`. The
/// straight apostrophe is deliberately not a quotation mark here, because it is far
/// more often a possessive than a quotation.
fn quoted_runs(text: &str) -> Vec<&str> {
    const PAIRS: [(char, &[char]); 5] = [
        ('"', &['"']),
        ('\u{201c}', &['\u{201d}', '\u{201c}']),
        ('\u{201e}', &['\u{201c}', '\u{201d}', '"']),
        ('\u{bb}', &['\u{ab}']),
        ('\u{ab}', &['\u{bb}']),
    ];
    let mut runs = Vec::new();
    let mut opened: Option<(usize, &[char])> = None;
    for (at, glyph) in text.char_indices() {
        match opened {
            Some((from, closers)) if closers.contains(&glyph) => {
                if at > from {
                    runs.push(&text[from..at]);
                }
                opened = None;
            }
            Some(_) => {}
            None => {
                if let Some((_, closers)) = PAIRS.iter().find(|(open, _)| *open == glyph) {
                    opened = Some((at + glyph.len_utf8(), closers));
                }
            }
        }
    }
    runs
}

/// Whether a quoted run is plausibly a name rather than a sentence.
///
/// Models quote whole clauses as readily as words, and a clause offered as a spelling
/// to correct is noise in a list whose whole value is that it is short.
fn looks_like_a_name(term: &str) -> bool {
    let letters = term.chars().filter(|glyph| glyph.is_alphabetic()).count();
    letters >= LEAST_LETTERS && term.chars().count() <= 60 && term.split_whitespace().count() <= 4
}

/// Words of a passage, with where each one starts.
fn words(text: &str) -> Vec<(usize, &str)> {
    let mut found = Vec::new();
    let mut start = None;
    for (at, glyph) in text.char_indices() {
        let letter = glyph.is_alphabetic() || glyph == '-';
        match (letter, start) {
            (true, None) => start = Some(at),
            (false, Some(from)) => {
                found.push((from, &text[from..at]));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(from) = start {
        found.push((from, &text[from..]));
    }
    found.retain(|(_, word)| word.chars().count() >= LEAST_LETTERS);
    found
}

/// One place a correction would apply, with enough of its sentence to judge it.
///
/// The judging matters. Some wrong spellings are also ordinary words: a participant
/// at the reference meeting is called Halde, and the transcriber wrote `Halle`,
/// which is the German for cross. Every occurrence there happened to be the person,
/// and that will not always hold — so this exists to be looked at before anything is
/// replaced.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
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
mod replacing_a_name_in_a_protocol {
    use super::replace_in_text;

    /// The reason this does not use a plain find and replace.
    ///
    /// German writes the interior of a compound in lower case, so a firm called
    /// Klinker appears inside "klinkerfassade" — and a literal replace walks past it.
    /// The rule is the transcript corrections' own, over prose rather than segments.
    #[test]
    fn catches_the_compound_form_as_well_as_the_name() {
        let text = "Klinker plant den Umbau. Das klinkerfassade Team ist zuständig.";
        let (found, written) = replace_in_text(text, "Klinker", "Nordenstadt");
        assert_eq!(found.len(), 2);
        assert_eq!(
            written,
            "Nordenstadt plant den Umbau. Das nordenstadter Team ist zuständig."
        );
    }

    /// Lowering the first letter of an abbreviation would produce hOAI, and an
    /// abbreviation never appears inside a compound that way.
    #[test]
    fn leaves_an_abbreviation_in_capitals_alone() {
        let (found, written) = replace_in_text("HOAI und hoai", "HOAI", "IBC");
        assert_eq!(found.len(), 1);
        assert_eq!(written, "IBC und hoai");
    }

    #[test]
    fn reports_the_line_and_the_words_around_each_change() {
        let text = "# Protokoll\n\nFrau Bauleitung von Klinker nannte die Frist.";
        let (found, _) = replace_in_text(text, "Klinker", "Nordenstadt");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].line, 3);
        assert!(found[0].context.contains("Bauleitung"));
        assert_eq!(found[0].matched, "Klinker");
        assert_eq!(found[0].replacement, "Nordenstadt");
    }

    #[test]
    fn changes_every_occurrence_on_a_line() {
        let (found, written) = replace_in_text("Klinker, Klinker, Klinker", "Klinker", "X");
        assert_eq!(found.len(), 3);
        assert_eq!(written, "X, X, X");
    }

    #[test]
    fn leaves_a_document_without_the_name_exactly_as_it_was() {
        let text = "Nichts hier trägt den Namen.\nAuch hier nicht.";
        let (found, written) = replace_in_text(text, "Klinker", "Nordenstadt");
        assert!(found.is_empty());
        assert_eq!(written, text);
    }

    #[test]
    fn does_nothing_for_an_empty_search() {
        let (found, written) = replace_in_text("Klinker", "  ", "X");
        assert!(found.is_empty());
        assert_eq!(written, "Klinker");
    }

    /// Line endings are the document's, and a replace must not rewrite them.
    #[test]
    fn keeps_the_shape_of_the_document() {
        let text = "Eins\n\nKlinker\n\nDrei";
        let (_, written) = replace_in_text(text, "Klinker", "Zwei");
        assert_eq!(written, "Eins\n\nZwei\n\nDrei");
    }
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

    fn doubted(id: &str, text: &str, unsure: &[&str]) -> TranscriptSegment {
        TranscriptSegment {
            id: id.into(),
            start_ms: 0,
            end_ms: 1000,
            speaker: "Speaker 1".into(),
            text: text.into(),
            needs_review: !unsure.is_empty(),
            uncertain_words: unsure.iter().map(|word| word.to_string()).collect(),
        }
    }

    /// The distinction the filter rests on: a word it usually gets right and fumbles
    /// once is a stumble; a word it never gets right is a word it does not know.
    #[test]
    fn only_a_word_never_heard_confidently_is_offered() {
        let segments = vec![
            doubted("a", "Das Trakwerk liegt darüber.", &["Trakwerk"]),
            doubted("b", "Das Trakwerk bleibt so.", &["Trakwerk"]),
            // Fumbled once out of three, so the transcriber does know this word.
            doubted("c", "Die Wohnungen im Norden.", &["Wohnungen"]),
            doubted("d", "Die Wohnungen sind fertig.", &[]),
            doubted("e", "Wohnungen überall.", &[]),
        ];
        let found = name_candidates(&segments, "");

        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].heard, "Trakwerk");
        assert_eq!(found[0].occurrences, 2);
        assert!(found[0].context.contains("Trakwerk"), "{:?}", found[0]);
    }

    /// What substitution leaves behind is a different problem, not less of the same
    /// one. A candidate is mis-heard the *same* way every time, which is what makes
    /// correcting one stem repair forty occurrences; this is the word whose
    /// mis-hearing varied, so there is no stem and nothing exact left to do.
    #[test]
    fn what_the_candidate_list_already_offers_is_not_asked_about_again() {
        let segments = vec![
            doubted("a", "Das Trakwerk liegt darüber.", &["Trakwerk"]),
            doubted("b", "Das Trakwerk bleibt so.", &["Trakwerk"]),
            doubted("c", "Herr Halle übernimmt das.", &["Halle"]),
        ];
        let offered = name_candidates(&segments, "");
        assert_eq!(offered.len(), 1, "Trakwerk is the deterministic one");

        let left = unsettled(&segments, &offered);
        assert_eq!(left.len(), 1, "{left:?}");
        assert_eq!(left[0].heard, "Halle");
        assert_eq!(left[0].segment_id, "c");
        assert_eq!(
            left[0].passage, "Herr Halle übernimmt das.",
            "the sentence it sits in, and nothing else"
        );
    }

    #[test]
    fn one_word_is_asked_about_once_however_often_it_was_doubted() {
        let segments = vec![
            doubted("a", "Die Nukera liefert.", &["Nukera"]),
            doubted(
                "b",
                "Nukera bestätigt, und Vermessung auch.",
                &["Nukera", "Vermessung"],
            ),
        ];
        let left = unsettled(&segments, &[]);
        assert_eq!(
            left.iter()
                .map(|word| word.heard.as_str())
                .collect::<Vec<_>>(),
            vec!["Nukera", "Vermessung"]
        );
    }

    #[test]
    fn a_transcript_with_nothing_left_asks_nothing() {
        let segments = vec![doubted("a", "Die Fassade bleibt.", &[])];
        assert!(unsettled(&segments, &[]).is_empty());
    }

    /// A guard against an unusual transcript rather than a limit that normally bites:
    /// substitution left about three genuinely wrong on the reference meeting. It also
    /// bounds the cost, since each of these is a request.
    #[test]
    fn the_number_asked_about_is_capped() {
        // Distinct and alphabetic: the trim strips digits along with punctuation, so
        // `Fremdwort001` and `Fremdwort002` are the same word by the time it counts.
        let segments: Vec<_> = (b'a'..b'a' + 30)
            .map(|letter| {
                let word = format!("Fremdwort{}", letter as char);
                doubted(
                    &format!("s{letter}"),
                    &format!("Hier steht {word} im Satz."),
                    &[word.as_str()],
                )
            })
            .collect();
        assert_eq!(unsettled(&segments, &[]).len(), MOST_UNSETTLED);
    }

    /// The class nothing else can reach. whisper flagged the mangled form of this
    /// client's name and not the plain wrong spelling, because it was confident about
    /// it — and a confidently wrong name is the one that reaches a client's inbox
    /// looking correct.
    #[test]
    fn a_name_the_transcriber_was_sure_of_is_found_in_the_protocols_own_note() {
        let segments = vec![
            doubted("a", "Klinker-Nord hat zugestimmt.", &[]),
            doubted("b", "Der Termin mit Klinker-Nord steht.", &[]),
        ];
        let protocol = "## Beschluss\n\nEs bleibt dabei.\n\n[Note: The term \
             \"Klinker-Nord\" is used in the source text; it is unclear if this \
             refers to a specific project name or location.]";

        let found = name_candidates(&segments, protocol);

        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].heard, "Klinker-Nord");
        assert_eq!(
            found[0].occurrences, 2,
            "counted in the transcript, not the note"
        );
        assert!(
            found[0].questioned,
            "and shown as the model's doubt, not whisper's"
        );
        assert!(found[0].context.contains("Klinker-Nord"));
    }

    /// The reason it is read by punctuation. A note written in German shares no words
    /// with the English one, and this product is for German offices — a pattern
    /// matching "it is unclear" would find nothing where it matters most.
    #[test]
    fn the_same_note_written_in_german_is_read_just_as_well() {
        let segments = vec![
            doubted("a", "Die Nukera liefert die Bauteile.", &[]),
            doubted("b", "Nukera bestätigt den Termin.", &[]),
        ];
        let protocol = "## Beschluss\n\n[Anmerkung: Der Begriff „Nukera“ kommt im \
             Ausgangstext vor; unklar, ob damit eine Firma gemeint ist.]";

        let found = name_candidates(&segments, protocol);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].heard, "Nukera");
        assert!(found[0].questioned);
    }

    /// The filter that keeps a model's invention out of a list of spellings somebody
    /// is invited to apply to their own record.
    #[test]
    fn a_name_the_transcript_never_says_is_not_offered() {
        let segments = vec![doubted("a", "Die Fassade bleibt.", &[])];
        let protocol = "[Note: \"Marchetti-Sauer\" may be a firm.]";
        assert!(name_candidates(&segments, protocol).is_empty());
    }

    /// Both sources finding the same word is one entry, marked as questioned. Two
    /// rows for one spelling would read as two different problems.
    #[test]
    fn a_word_both_sources_find_is_offered_once() {
        let segments = vec![
            doubted("a", "Das Trakwerk liegt darüber.", &["Trakwerk"]),
            doubted("b", "Das Trakwerk bleibt so.", &["Trakwerk"]),
        ];
        let found = name_candidates(&segments, "[Note: \"Trakwerk\" is unfamiliar.]");

        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].questioned);
        assert_eq!(found[0].occurrences, 2);
    }

    #[test]
    fn what_the_protocol_questioned_is_read_without_being_fooled_by_prose() {
        // A markdown link is not an aside, and a quoted clause is not a name.
        let markdown = "See [the note](https://example.invalid/\"x\") and \
             [Note: \"the entire discussion of the facade was hard to follow\"] and \
             [Note: \"ab\" is short] and [Note: \"HOAI\" is unfamiliar.]";
        assert_eq!(names_the_protocol_questioned(markdown), vec!["HOAI"]);
    }

    #[test]
    fn a_protocol_with_no_notes_at_all_adds_nothing() {
        assert!(names_the_protocol_questioned("# Protokoll\n\nEs wurde gesprochen.").is_empty());
        assert!(names_the_protocol_questioned("").is_empty());
    }

    #[test]
    fn a_word_heard_once_is_a_stumble_rather_than_a_name() {
        let segments = vec![doubted("a", "Ein Propositionen hier.", &["Propositionen"])];
        assert!(name_candidates(&segments, "").is_empty());
    }

    #[test]
    fn candidates_come_most_frequent_first() {
        let segments = vec![
            doubted("a", "Nukera und Trakwerk.", &["Nukera", "Trakwerk"]),
            doubted("b", "Nukera und Trakwerk.", &["Nukera", "Trakwerk"]),
            doubted("c", "Nukera nochmal.", &["Nukera"]),
        ];
        let found = name_candidates(&segments, "");
        assert_eq!(found[0].heard, "Nukera");
        assert_eq!(found[0].occurrences, 3);
        assert_eq!(found[1].heard, "Trakwerk");
    }

    /// The panel shows a list somebody finishes. An unusual transcript must not turn
    /// it back into the 322-item chore it replaced.
    #[test]
    fn the_list_stays_short_enough_to_finish() {
        // Distinct alphabetic words: digits end a word, so Wortform1 and Wortform2
        // would both be counted as "Wortform" and the list would be one item long.
        let letters = [
            "aaaa", "bbbb", "cccc", "dddd", "eeee", "ffff", "gggg", "hhhh", "iiii", "jjjj", "kkkk",
            "llll", "mmmm", "nnnn", "oooo", "pppp", "qqqq", "rrrr", "ssss", "tttt",
        ];
        let segments: Vec<TranscriptSegment> = letters
            .iter()
            .enumerate()
            .flat_map(|(index, word)| {
                let text = format!("Ein {word} hier.");
                [
                    doubted(&format!("a{index}"), &text, &[word]),
                    doubted(&format!("b{index}"), &text, &[word]),
                ]
            })
            .collect();
        assert_eq!(name_candidates(&segments, "").len(), MOST_CANDIDATES);
    }

    #[test]
    fn a_transcript_with_no_uncertainty_offers_nothing() {
        let segments = vec![doubted("a", "Alles war deutlich zu hören.", &[])];
        assert!(name_candidates(&segments, "").is_empty());
    }

    /// German capitalises nouns, so the same word appears both ways; counting them
    /// separately would split a candidate in half.
    #[test]
    fn a_word_is_counted_however_it_is_capitalised() {
        let segments = vec![
            doubted("a", "Das Trakwerk hier.", &["Trakwerk"]),
            doubted("b", "trakwerk kommt später.", &["trakwerk"]),
        ];
        let found = name_candidates(&segments, "");
        assert_eq!(found.len(), 1, "{found:?}");
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

/// One place in a protocol where a name would change.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextMatch {
    /// The line it is on, so somebody can see where in the document it falls.
    pub line: u32,
    /// The words around it, which is what makes a preview worth reading.
    pub context: String,
    pub matched: String,
    pub replacement: String,
}

/// What replacing a name through a protocol would actually do.
///
/// The same rule the transcript corrections use, over prose rather than segments:
/// a capitalised name is looked for in its compound form as well, because German
/// writes the interior of a compound in lower case and a firm called `Klinker`
/// appears inside `klinkerfassade` untouched by a literal replace.
///
/// Returns what would change and the text it would become, and stores nothing. The
/// caller decides whether to keep it.
pub(crate) fn replace_in_text(text: &str, wrong: &str, right: &str) -> (Vec<TextMatch>, String) {
    let correction = Correction {
        wrong: wrong.trim().to_string(),
        right: right.to_string(),
    };
    if correction.wrong.is_empty() {
        return (Vec::new(), text.to_string());
    }

    let mut found = Vec::new();
    let mut written = String::with_capacity(text.len());

    for (index, line) in text.split('\n').enumerate() {
        if index > 0 {
            written.push('\n');
        }
        let mut rest = line;
        let mut line_out = String::with_capacity(line.len());
        loop {
            // The earliest match of any form, so a name and its own compound form
            // do not race each other and produce overlapping edits.
            let hit = correction
                .forms()
                .into_iter()
                .filter_map(|(from, to)| rest.find(from.as_str()).map(|at| (at, from, to)))
                .min_by_key(|(at, _, _)| *at);
            let Some((at, from, to)) = hit else {
                break;
            };

            let consumed = line.len() - rest.len();
            line_out.push_str(&rest[..at]);
            line_out.push_str(&to);
            found.push(TextMatch {
                line: index as u32 + 1,
                context: around(line, consumed + at, from.len()),
                matched: from.clone(),
                replacement: to.clone(),
            });
            rest = &rest[at + from.len()..];
        }
        line_out.push_str(rest);
        written.push_str(&line_out);
    }

    (found, written)
}
