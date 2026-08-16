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

/// Quantities the protocol states that the meeting did not.
///
/// This is the check that holds whatever style is in use, and it is the one that
/// matters most. How much a protocol keeps is a matter of what was asked for: a
/// formal record keeps nearly everything, and a set of brief notes deliberately
/// keeps very little, so coverage is a target a style sets rather than a virtue
/// in itself. Inventing a figure is a defect under every style there could be.
///
/// A number is treated as invented when the protocol states it as a quantity and
/// no segment of the transcript states the same number. Numbering the sections of
/// a document is not stating a quantity, so only numbers carrying a unit are
/// considered on either side.
pub(crate) fn invented(segments: &[TranscriptSegment], protocol: &str) -> Vec<String> {
    let stated: Vec<String> = segments
        .iter()
        .flat_map(|segment| quantities_in(&segment.text))
        .filter_map(|(_, text)| leading_number(&text).map(str::to_string))
        .collect();
    let mut found = Vec::new();
    for (_, text) in quantities_in(protocol) {
        let Some(number) = leading_number(&text) else {
            continue;
        };
        if !stated.iter().any(|said| said == number) && !found.contains(&text) {
            found.push(text);
        }
    }
    found
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
        if let Some(end) = unit_at(text, after) {
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

/// German declines its units. A meeting says "178 Quadratmetern" in the dative
/// and "zehn Metern" likewise, and requiring the unit to end exactly where the
/// dictionary form ends misses both — which is not a small loss, because those
/// are precisely the figures a protocol is judged on.
///
/// Only genuine inflections are allowed, so "Meterware" is still not a length.
const INFLECTIONS: &[&str] = &["", "n", "s", "e", "en", "er", "ern", "es"];

/// The end of the unit beginning at `from`, if there is one.
fn unit_at(text: &str, from: usize) -> Option<usize> {
    if from >= text.len() || !text.is_char_boundary(from) {
        return None;
    }
    let rest = &text[from..];
    let unit = UNITS.iter().copied().find(|unit| rest.starts_with(unit))?;
    let written_in_letters = unit.chars().next().is_some_and(char::is_alphabetic);
    if !written_in_letters {
        return Some(from + unit.len());
    }
    let tail = &rest[unit.len()..];
    INFLECTIONS
        .iter()
        .filter(|ending| tail.starts_with(*ending))
        // Longest first, so "ern" is preferred over "e".
        .max_by_key(|ending| ending.len())
        .filter(|ending| {
            !tail[ending.len()..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric)
        })
        .map(|ending| from + unit.len() + ending.len())
}

fn leading_number(text: &str) -> Option<&str> {
    let end = text
        .find(|character: char| !character.is_ascii_digit() && character != ',' && character != '.')
        .unwrap_or(text.len());
    (end > 0).then(|| text[..end].trim_end_matches(['.', ',']))
}

/// Tasks the protocol records with nobody against them.
///
/// A style that asks for a table of next steps asks for two columns, the task and
/// who carries it. Where the second is empty the protocol is saying that something
/// was agreed and nobody was put on the hook — which is worth a person's attention
/// before the meeting is over, because it is the cheapest thing in the world to fix
/// then and an argument at the next meeting otherwise.
///
/// This is evidence, not a fault. The formal style tells the model plainly never to
/// invent an owner, and to leave a task unattributed where the meeting did not say
/// who was responsible. An empty owner can therefore be an accurate record. What it
/// cannot be is invisible.
///
/// The reading is structural rather than linguistic: a row of a table with a hole in
/// it, wherever the table sits and whatever language it is written in. Matching on
/// words like "Aufgaben" or "Actions" would work in German and English and quietly
/// stop working in Dutch, and this project has already paid once for a check that
/// assumed the protocol's language.
pub(crate) fn unowned_tasks(protocol: &str) -> Vec<String> {
    let mut found = Vec::new();
    let lines: Vec<&str> = protocol.lines().collect();
    let mut index = 0;
    while index < lines.len() {
        // A markdown table is a header row, a row of dashes, then its contents.
        // Requiring the dashes is what stops a sentence containing a pipe from
        // being read as a table.
        if is_row(lines[index]) && index + 1 < lines.len() && is_divider(lines[index + 1]) {
            let mut rows: Vec<Vec<String>> = Vec::new();
            let mut at = index + 2;
            while at < lines.len() && is_row(lines[at]) {
                let cells = cells_of(lines[at]);
                // A single-column table has nowhere to put an owner, so it is not
                // a table of tasks and nothing here applies to it.
                if cells.len() >= 2 {
                    rows.push(cells.iter().map(|cell| cell.trim().to_string()).collect());
                }
                at += 1;
            }
            found.extend(unowned_in(&rows));
            index = at;
            continue;
        }
        index += 1;
    }
    found
}

/// The tasks in one table that nobody is carrying.
///
/// An empty cell, or one holding a mark people write to mean empty. It cannot see
/// the other way a cell says nobody: a model told never to invent an owner writes
/// it in words — `Nicht angegeben`, `not stated`, `TBD` — and this reads that as a
/// filled cell and stays quiet.
///
/// Recognising those words needs the protocol's language, which this deliberately
/// does not have. Finding them by repetition was tried and abandoned: a value
/// filling most of an owner column looks exactly like a placeholder and exactly
/// like the one person who carries most of a meeting's follow-ups, which is the
/// common case. Telling somebody their assigned work is unassigned is a worse
/// failure than missing an unassigned task, so the limit is left in place and
/// written down rather than papered over with a threshold.
fn unowned_in(rows: &[Vec<String>]) -> Vec<String> {
    rows.iter()
        .filter(|row| !row[0].is_empty() && row[1..].iter().any(|cell| is_blank(cell)))
        .map(|row| row[0].clone())
        .collect()
}

fn is_row(line: &str) -> bool {
    let line = line.trim();
    line.starts_with('|') && line.len() > 1
}

/// The `|---|:--:|` line under a table's headings.
fn is_divider(line: &str) -> bool {
    let line = line.trim();
    is_row(line)
        && line.contains('-')
        && line
            .chars()
            .all(|character| matches!(character, '|' | '-' | ':' | ' '))
}

fn cells_of(line: &str) -> Vec<&str> {
    let line = line.trim();
    let inner = line
        .strip_prefix('|')
        .unwrap_or(line)
        .strip_suffix('|')
        .unwrap_or(line);
    inner.split('|').collect()
}

/// Empty, or one of the marks people write to mean empty. A dash in a cell is a
/// person saying "nobody", not a person naming somebody called "-".
fn is_blank(cell: &str) -> bool {
    let cell = cell.trim();
    cell.is_empty()
        || cell.chars().all(|character| {
            matches!(
                character,
                '-' | '\u{2013}' | '\u{2014}' | '.' | '?' | '/' | '\u{a0}'
            )
        })
}

#[cfg(test)]
mod tests {

    /// The case this exists for: a table of next steps where somebody was named
    /// and somebody was not.
    #[test]
    fn a_task_with_an_empty_owner_is_reported() {
        let protocol = "\
## Nächste Schritte

| Aufgabe | Verantwortlich |
| --- | --- |
| Brandschutzklappen abstimmen | Herr Seidel |
| Heizlastberechnung nachreichen |  |
| Angebot einholen | - |
";
        assert_eq!(
            unowned_tasks(protocol),
            vec![
                "Heizlastberechnung nachreichen".to_string(),
                "Angebot einholen".to_string()
            ]
        );
    }

    /// What this cannot see, recorded as a test so the limit is visible rather than
    /// discovered again. A model told never to invent an owner writes the absence in
    /// words, and those rows stay quiet. Reading a real protocol is what revealed
    /// it; counting never would have.
    #[test]
    fn an_owner_written_as_words_is_missed_and_that_is_known() {
        let protocol = "\
| Aufgabe | Verantwortlich |
| --- | --- |
| Grundrisse korrigieren | Fachplanung |
| Abweichungen bei den Bädern klären | Nicht angegeben |
| Raster beim Balkonhersteller prüfen | Nicht angegeben |
| Fassade mit dem Team besprechen | Team |
| Trennwände bei Haus A prüfen | Nicht angegeben |
";
        // Three of these five tasks name nobody, and none is reported.
        assert!(unowned_tasks(protocol).is_empty());
    }

    /// The reason the gap above is left open. Finding those words by repetition was
    /// tried, and it calls this person a placeholder: one owner carrying most of the
    /// follow-ups is the common shape of a real meeting.
    #[test]
    fn one_person_carrying_several_tasks_is_never_reported() {
        let protocol = "\
| Task | Owner |
| --- | --- |
| Send the drawings | Mira |
| Confirm the survey | Mira |
| Book the crane | Tomas |
| Update the schedule | Mira |
";
        assert!(
            unowned_tasks(protocol).is_empty(),
            "Mira is a person, not a placeholder"
        );
    }

    /// Nothing is read from the words, so a protocol in a language nobody
    /// anticipated is read the same way.
    #[test]
    fn the_language_of_the_table_does_not_matter() {
        let protocol = "\
| Task | Owner | Due |
| --- | --- | --- |
| Send the drawings | Mira | Friday |
| Confirm the survey | | Friday |
";
        assert_eq!(
            unowned_tasks(protocol),
            vec!["Confirm the survey".to_string()]
        );
    }

    /// A sentence with a pipe in it is not a table, and a table needs its rule.
    #[test]
    fn prose_containing_a_pipe_is_not_read_as_a_table() {
        let protocol = "Die Wand | die Decke wurde besprochen, ohne Ergebnis.\n";
        assert!(unowned_tasks(protocol).is_empty());
    }

    /// A table of one column has nowhere to name anybody, so it says nothing about
    /// ownership and must not be reported as though it did.
    #[test]
    fn a_single_column_table_reports_nothing() {
        let protocol = "| Punkt |\n| --- |\n| Lüftung |\n";
        assert!(unowned_tasks(protocol).is_empty());
    }

    /// Every task carrying a name is the good case, and it must be quiet.
    #[test]
    fn a_fully_attributed_table_reports_nothing() {
        let protocol = "\
| Aufgabe | Verantwortlich |
| --- | --- |
| Termin bestätigen | Frau Netzel |
| Unterlagen prüfen | Herr Balk |
";
        assert!(unowned_tasks(protocol).is_empty());
    }

    /// More than one table in a document, which a long protocol will have.
    #[test]
    fn tables_after_the_first_are_read_too() {
        let protocol = "\
| Thema | Stand |
| --- | --- |
| Lüftung | offen |

Zwischentext.

| Aufgabe | Verantwortlich |
| --- | --- |
| Protokoll versenden | |
";
        assert_eq!(
            unowned_tasks(protocol),
            vec!["Protokoll versenden".to_string()]
        );
    }

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
        // German declines its units, and the dative is what a meeting speaks in.
        let declined = [segment("d", "Von 178 Quadratmetern auf 12 Metern Breite.")];
        let texts: Vec<String> = quantities(&declined)
            .into_iter()
            .map(|fact| fact.text)
            .collect();
        assert_eq!(texts, ["178 Quadratmetern", "12 Metern"]);
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
            segment("e", "Die 7 Meterlatte liegt dort."),
            segment("b", "Siehe Punkt 3 und Haus 4."),
            segment("c", "Die Norm DIN18040 gilt hier."),
            segment("d", "Um 30 ging es nicht."),
        ];
        assert!(quantities(&segments).is_empty());
    }

    #[test]
    fn a_figure_the_meeting_never_stated_is_reported_as_invented() {
        let segments = [
            segment("a", "Die Fläche beträgt 120 m² pro Geschoss."),
            segment("b", "Der Anteil liegt bei 30 Prozent."),
        ];
        // Numbering a section is not stating a quantity, and a figure that was
        // said may be rewritten freely.
        assert!(
            invented(
                &segments,
                "## 1. Flächen\n\nRund 120 Quadratmeter, davon 30 %."
            )
            .is_empty()
        );
        // A quantity nobody said is a defect under any style.
        assert_eq!(
            invented(&segments, "Die Fläche beträgt 450 m² insgesamt."),
            vec!["450 m²".to_string()]
        );
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
            let made_up = invented(&segments, &protocol);
            println!(
                "invented: {} not stated by the meeting {made_up:?}",
                made_up.len()
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
