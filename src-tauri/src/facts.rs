//! What a transcript says that can be found without a model.
//!
//! A protocol is mostly judgement, and judgement needs a model. But some of what
//! a meeting produces is not a matter of opinion at all — a quantity was either
//! said or it was not — and that part can be found by reading the text directly.
//!
//! This matters because it turns an open question into a checkable one. Measured
//! on a real 81-minute meeting, the transcript contained ten quantities, the
//! protocol a person wrote recorded nine of them, and the generated protocol
//! recorded one. Nothing about that gap requires a reader to judge it: the
//! information was present and findable, and it was lost.
//!
//! So the scan runs first, and what it finds becomes a checklist the generated
//! protocol is measured against. See `docs/PROTOCOL_GENERATION.md`.

use crate::domain::TranscriptSegment;

/// A statement of fact taken verbatim from the transcript, with the segment it
/// came from so it can be traced back and played.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Fact {
    /// Exactly as spoken, so a comparison against the protocol is meaningful.
    pub text: String,
    pub segment_id: String,
    /// Where in the segment it starts, so the reader can be taken to it.
    pub offset: usize,
}

/// Units worth recognising, longest first so that "Quadratmeter" is not read as
/// "Meter" with a stray prefix.
///
/// Deliberately short. A unit that is only sometimes a unit — a bare "m", which
/// is also the start of a great many German words — costs more in false alarms
/// than it earns, and this list exists to be trusted rather than to be complete.
const UNITS: &[&str] = &[
    "Quadratmeter",
    "Zentimeter",
    "Millimeter",
    "Kilometer",
    "Prozent",
    "Meter",
    "Euro",
    "EUR",
    "m²",
    "qm",
    "cm",
    "mm",
    "km",
    "%",
    "€",
];

/// Every quantity the transcript states, in the order it was said.
///
/// A quantity is a number followed by a unit, allowing for the space that may or
/// may not be there. German decimals use a comma and thousands a full stop, and
/// both are kept as written because the point is to compare with what the
/// protocol says, not to do arithmetic.
pub(crate) fn quantities(segments: &[TranscriptSegment]) -> Vec<Fact> {
    let mut found = Vec::new();
    for segment in segments {
        for (offset, text) in quantities_in(&segment.text) {
            found.push(Fact {
                text,
                segment_id: segment.id.clone(),
                offset,
            });
        }
    }
    found
}

/// Whether a protocol accounts for a fact.
///
/// Matching is on the number rather than the whole phrase, because a protocol
/// legitimately rewrites "dreissig Prozent" as "30 %" or moves the unit. The
/// number surviving is what matters; how it is worded is the author's business.
///
/// The number must stand on its own. A plain substring search reports "30" as
/// present inside "1930" and inside "Punkt 305", which does not overstate the
/// count a little — it silently inflates the one measurement this project uses
/// to decide whether a protocol is any good. Both sides of the match must
/// therefore end at something that is not part of a number.
pub(crate) fn is_accounted_for(fact: &Fact, protocol: &str) -> bool {
    let Some(number) = leading_number(&fact.text) else {
        return false;
    };
    occurrences(protocol, number).any(|(start, end)| {
        let before = protocol[..start].chars().next_back();
        let after = protocol[end..].chars().next();
        !before.is_some_and(is_number_part) && !after.is_some_and(is_number_part)
    })
}

/// A digit, or a separator that only counts when it sits between digits — so the
/// full stop ending a sentence does not disqualify a number that closes it.
fn is_number_part(character: char) -> bool {
    character.is_ascii_digit()
}

fn occurrences<'a>(
    haystack: &'a str,
    needle: &'a str,
) -> impl Iterator<Item = (usize, usize)> + 'a {
    let mut from = 0;
    std::iter::from_fn(move || {
        let found = haystack[from..].find(needle)? + from;
        from = found + needle.len().max(1);
        Some((found, found + needle.len()))
    })
}

/// The quantities in one piece of text, as (offset, text) pairs.
fn quantities_in(text: &str) -> Vec<(usize, String)> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_digit() {
            index += 1;
            continue;
        }
        // A number may not begin in the middle of a word or another number.
        if index > 0 && (bytes[index - 1].is_ascii_alphanumeric() || bytes[index - 1] == b'.') {
            index += 1;
            continue;
        }
        let start = index;
        while index < bytes.len()
            && (bytes[index].is_ascii_digit() || bytes[index] == b',' || bytes[index] == b'.')
        {
            index += 1;
        }
        // A trailing separator belongs to the sentence, not to the number.
        while index > start && matches!(bytes[index - 1], b',' | b'.') {
            index -= 1;
        }
        let after = skip_spaces(text, index);
        if let Some(unit) = unit_at(text, after) {
            let end = after + unit.len();
            found.push((start, text[start..end].to_string()));
            index = end;
        }
    }
    found
}

fn skip_spaces(text: &str, from: usize) -> usize {
    let bytes = text.as_bytes();
    let mut index = from;
    while index < bytes.len() && bytes[index] == b' ' {
        index += 1;
    }
    index
}

/// The unit beginning at `from`, if there is one. A unit written in letters must
/// end there too, so that "Meterware" is not read as a length.
fn unit_at(text: &str, from: usize) -> Option<&'static str> {
    if from >= text.len() || !text.is_char_boundary(from) {
        return None;
    }
    let rest = &text[from..];
    UNITS.iter().copied().find(|unit| {
        rest.starts_with(unit)
            && (!unit.chars().next().is_some_and(char::is_alphabetic)
                || !rest[unit.len()..]
                    .chars()
                    .next()
                    .is_some_and(char::is_alphanumeric))
    })
}

fn leading_number(text: &str) -> Option<&str> {
    let end = text
        .find(|character: char| !character.is_ascii_digit() && character != ',' && character != '.')
        .unwrap_or(text.len());
    (end > 0).then(|| text[..end].trim_end_matches(['.', ',']))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segment(id: &str, text: &str) -> TranscriptSegment {
        TranscriptSegment {
            id: id.to_string(),
            start_ms: 0,
            end_ms: 1,
            speaker: "Speaker 1".to_string(),
            text: text.to_string(),
            needs_review: false,
            uncertain_words: Vec::new(),
        }
    }

    #[test]
    fn finds_quantities_however_they_are_spaced_and_punctuated() {
        let segments = [
            segment("a", "Die Fläche beträgt 120 m² und der Anteil 30%."),
            segment("b", "Das kostet 1.250,50 Euro, sagt er."),
            segment("c", "Etwa 3,5 Meter breit."),
        ];
        let found = quantities(&segments);
        let texts: Vec<&str> = found.iter().map(|fact| fact.text.as_str()).collect();
        assert_eq!(texts, ["120 m²", "30%", "1.250,50 Euro", "3,5 Meter"]);
        assert_eq!(found[0].segment_id, "a");
        assert_eq!(found[3].segment_id, "c");
    }

    /// The scan exists to be trusted. A word that merely starts with a unit, or a
    /// number that is part of another token, must not become a checklist item
    /// somebody then has to dismiss by hand.
    #[test]
    fn does_not_invent_quantities() {
        let segments = [
            segment("a", "Wir haben 5 Meterware bestellt."),
            segment("b", "Siehe Punkt 3 und Haus 4."),
            segment("c", "Die Norm DIN18040 gilt hier."),
            segment("d", "Um 30 ging es nicht."),
        ];
        assert!(quantities(&segments).is_empty());
    }

    /// Runs the scan over a real transcript and reports the coverage of a real
    /// protocol against it. Ignored by default: it needs meeting material, which
    /// never lives in this repository.
    ///
    /// `LOCALOG_FACTS_TRANSCRIPT=working.json LOCALOG_FACTS_PROTOCOL=protocol.md \
    ///   cargo test --lib facts -- --ignored --nocapture`
    #[test]
    #[ignore = "requires a local transcript"]
    fn coverage_against_a_real_meeting() {
        let transcript = std::env::var("LOCALOG_FACTS_TRANSCRIPT").expect("set the transcript");
        let raw = std::fs::read_to_string(&transcript).unwrap();
        let document: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let segments: Vec<TranscriptSegment> =
            serde_json::from_value(document["segments"].clone()).unwrap();
        let found = quantities(&segments);
        println!(
            "{} segments, {} quantities found",
            segments.len(),
            found.len()
        );
        for fact in &found {
            println!("   {:>16}   in {}", fact.text, fact.segment_id);
        }
        if let Ok(path) = std::env::var("LOCALOG_FACTS_PROTOCOL") {
            let protocol = std::fs::read_to_string(path).unwrap();
            let kept = found
                .iter()
                .filter(|fact| is_accounted_for(fact, &protocol))
                .count();
            println!(
                "\ncoverage: {kept} of {} quantities appear in the protocol",
                found.len()
            );
        }
    }

    #[test]
    fn a_protocol_accounts_for_a_quantity_however_it_rewrites_it() {
        let segments = [segment("a", "Ungefähr 120 m² pro Geschoss.")];
        let fact = &quantities(&segments)[0];
        // The unit moved and the wording changed; the number is what carries.
        assert!(is_accounted_for(fact, "Je Geschoss rund 120 Quadratmeter."));
        // A number closing a sentence still counts.
        assert!(is_accounted_for(fact, "Die Fläche beträgt 120."));
        assert!(!is_accounted_for(
            fact,
            "Je Geschoss rund 140 Quadratmeter."
        ));
        assert!(!is_accounted_for(fact, "Die Fläche wurde besprochen."));
    }

    /// The count decides whether a protocol is considered any good, so a number
    /// found inside a larger one must not be reported as present. A plain
    /// substring search inflates the measurement rather than merely blurring it.
    #[test]
    fn a_number_inside_another_number_does_not_count() {
        let segments = [segment("a", "Ungefähr 30 Prozent der Fläche.")];
        let fact = &quantities(&segments)[0];
        assert!(!is_accounted_for(fact, "Das Gebäude stammt von 1930."));
        assert!(!is_accounted_for(fact, "Siehe Position 305 der Liste."));
        assert!(!is_accounted_for(fact, "Die Norm 4030 gilt."));
        // But a genuine mention anywhere in the document does count.
        assert!(is_accounted_for(
            fact,
            "Von 1930 bis heute, rund 30 % davon."
        ));
    }
}
