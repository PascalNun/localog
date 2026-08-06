//! Assigning speakers to transcript segments.
//!
//! A diariser reports who spoke when; a transcriber reports what was said when.
//! Their boundaries never coincide, because one follows voices and the other
//! follows sentences. This module joins the two by time, and is deliberately
//! independent of any particular diarisation runtime so it can be tested without
//! one.

/// A stretch of audio the diariser attributes to a single voice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SpeakerTurn {
    pub start_ms: u64,
    pub end_ms: u64,
    /// The diariser's own label, such as `speaker_00`.
    pub speaker: String,
}

/// Parse the turns printed by `sherpa-onnx-offline-speaker-diarization`, whose
/// lines look like `0.031 -- 3.187 speaker_01`. Anything unparseable is skipped
/// rather than failing the job: a missing turn costs a speaker label, while a
/// hard failure would cost the whole transcript.
pub(crate) fn parse_turns(output: &str) -> Vec<SpeakerTurn> {
    let mut turns = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        let Some((range, speaker)) = line.rsplit_once(char::is_whitespace) else {
            continue;
        };
        if !speaker.starts_with("speaker") {
            continue;
        }
        let Some((start, end)) = range.split_once("--") else {
            continue;
        };
        let (Ok(start), Ok(end)) = (start.trim().parse::<f64>(), end.trim().parse::<f64>()) else {
            continue;
        };
        if !start.is_finite() || !end.is_finite() || end <= start || start < 0.0 {
            continue;
        }
        turns.push(SpeakerTurn {
            start_ms: (start * 1000.0) as u64,
            end_ms: (end * 1000.0) as u64,
            speaker: speaker.to_string(),
        });
    }
    turns.sort_by_key(|turn| turn.start_ms);
    turns
}

/// The speaker whose turns overlap a segment the most, or `None` when nothing
/// overlaps it. Choosing by greatest overlap rather than by whoever started the
/// segment matters: a transcript segment often begins with the tail of one
/// person's sentence before the next person takes over.
pub(crate) fn speaker_for_segment(
    turns: &[SpeakerTurn],
    start_ms: u64,
    end_ms: u64,
) -> Option<&str> {
    if end_ms <= start_ms {
        return None;
    }
    let mut best: Option<(&str, u64)> = None;
    for turn in turns {
        // Turns are ordered, so nothing further can overlap once we pass the end.
        if turn.start_ms >= end_ms {
            break;
        }
        let overlap = turn
            .end_ms
            .min(end_ms)
            .saturating_sub(turn.start_ms.max(start_ms));
        if overlap == 0 {
            continue;
        }
        match best {
            Some((_, best_overlap)) if best_overlap >= overlap => {}
            _ => best = Some((turn.speaker.as_str(), overlap)),
        }
    }
    best.map(|(speaker, _)| speaker)
}

/// Map the diariser's labels onto stable, human-facing names in the order the
/// speakers first appear, so the first person to talk becomes `Speaker 1`.
/// Without this the numbering would follow the diariser's clustering order,
/// which is arbitrary and changes between runs.
pub(crate) fn assign_speakers<S>(segments: &[S], turns: &[SpeakerTurn]) -> Vec<String>
where
    S: SegmentTiming,
{
    let mut order: Vec<&str> = Vec::new();
    let mut resolved = Vec::with_capacity(segments.len());
    for segment in segments {
        let label = speaker_for_segment(turns, segment.start_ms(), segment.end_ms());
        resolved.push(label);
        if let Some(label) = label
            && !order.contains(&label)
        {
            order.push(label);
        }
    }
    resolved
        .into_iter()
        .map(|label| match label {
            Some(label) => {
                let index = order.iter().position(|known| *known == label).unwrap_or(0);
                format!("Speaker {}", index + 1)
            }
            // A segment no turn covers keeps the neutral label rather than being
            // guessed at; an invented attribution is worse than an unlabelled one.
            None => "Speaker 1".to_string(),
        })
        .collect()
}

/// Implemented by anything with a start and end, so the alignment logic does not
/// depend on the transcript type.
pub(crate) trait SegmentTiming {
    fn start_ms(&self) -> u64;
    fn end_ms(&self) -> u64;
}

impl SegmentTiming for crate::domain::TranscriptSegment {
    fn start_ms(&self) -> u64 {
        self.start_ms
    }
    fn end_ms(&self) -> u64 {
        self.end_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Span(u64, u64);
    impl SegmentTiming for Span {
        fn start_ms(&self) -> u64 {
            self.0
        }
        fn end_ms(&self) -> u64 {
            self.1
        }
    }

    /// Verbatim shape from the spike run against sherpa-onnx.
    const REAL_OUTPUT: &str = "\
0.031 -- 3.187 speaker_01
3.187 -- 6.815 speaker_00
7.203 -- 11.033 speaker_01
10.679 -- 13.970 speaker_02
13.970 -- 14.527 speaker_01
14.999 -- 19.184 speaker_00
19.589 -- 23.521 speaker_01";

    #[test]
    fn parses_the_real_diariser_output() {
        let turns = parse_turns(REAL_OUTPUT);
        assert_eq!(turns.len(), 7);
        assert_eq!(turns[0].start_ms, 31);
        assert_eq!(turns[0].end_ms, 3187);
        assert_eq!(turns[0].speaker, "speaker_01");
        assert_eq!(turns[6].end_ms, 23521);
    }

    #[test]
    fn ignores_lines_that_are_not_turns() {
        let turns = parse_turns(
            "Loading model...\n\
             0.000 -- 1.000 speaker_00\n\
             garbage\n\
             5.000 -- 2.000 speaker_01\n\
             not -- a number speaker_02\n",
        );
        // Only the well-formed, forward-ordered line survives.
        assert_eq!(turns.len(), 1);
        assert_eq!(turns[0].speaker, "speaker_00");
    }

    #[test]
    fn a_segment_takes_the_speaker_it_overlaps_most() {
        let turns = parse_turns(REAL_OUTPUT);
        // Fully inside one turn.
        assert_eq!(speaker_for_segment(&turns, 1000, 2000), Some("speaker_01"));
        // Straddles a boundary: 3.0-3.2 is mostly speaker_01, 3.2-6.0 mostly speaker_00.
        assert_eq!(speaker_for_segment(&turns, 3000, 6000), Some("speaker_00"));
        // Beyond every turn.
        assert_eq!(speaker_for_segment(&turns, 30_000, 31_000), None);
    }

    #[test]
    fn numbering_follows_who_speaks_first_not_the_diariser_ordering() {
        let turns = parse_turns(REAL_OUTPUT);
        // The diariser's first label here is speaker_01, so naive numbering would
        // start at 2. The first voice heard must become Speaker 1.
        let segments = [Span(0, 3000), Span(3200, 6800), Span(11_000, 13_000)];
        let names = assign_speakers(&segments, &turns);
        assert_eq!(names, vec!["Speaker 1", "Speaker 2", "Speaker 3"]);
    }

    #[test]
    fn an_uncovered_segment_is_not_guessed_at() {
        let turns = parse_turns("0.000 -- 1.000 speaker_00");
        let segments = [Span(0, 900), Span(50_000, 51_000)];
        let names = assign_speakers(&segments, &turns);
        assert_eq!(names, vec!["Speaker 1", "Speaker 1"]);
    }

    #[test]
    fn no_turns_at_all_leaves_every_segment_neutral() {
        let segments = [Span(0, 1000), Span(1000, 2000)];
        let names = assign_speakers(&segments, &[]);
        assert_eq!(names, vec!["Speaker 1", "Speaker 1"]);
    }
}
