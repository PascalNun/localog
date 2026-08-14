//! Assigning speakers to transcript segments.
//!
//! A diariser reports who spoke when; a transcriber reports what was said when.
//! Their boundaries never coincide, because one follows voices and the other
//! follows sentences. This module joins the two by time, and is deliberately
//! independent of any particular diarisation runtime so it can be tested without
//! one.

/// A short piece of one transcript segment, and where it lands in the condensed
/// audio built from all of them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Sample {
    /// Index of the transcript segment this was taken from.
    pub segment: usize,
    pub source_start_ms: u64,
    pub source_end_ms: u64,
    pub condensed_start_ms: u64,
    pub condensed_end_ms: u64,
}

/// How much of each segment to play. Two seconds is roughly what a speaker
/// embedding needs to place a voice; less starts losing quiet or overlapping
/// speakers, and more spends time to learn nothing new.
pub(crate) const SAMPLE_MS: u64 = 2_000;

/// The quiet between samples, long enough for the diariser's segmentation to
/// break there and short enough not to undo the saving.
pub(crate) const GAP_MS: u64 = 300;

/// Below this a segment is not played at all. A half-second of speech is as
/// likely to add a spurious cluster as to identify anybody, and such segments
/// take a neighbour's speaker instead.
pub(crate) const SHORTEST_MS: u64 = 700;

/// Choose what to play the diariser, and work out where each piece will land.
///
/// Identifying a voice needs a couple of seconds of it, not a whole utterance,
/// and separation runs after transcription so the segments are already known. On
/// the reference meeting the diariser embeds seventy-three minutes of speech and
/// takes twenty-six minutes doing it; two seconds of each of its 753 segments is
/// about twenty-five minutes of audio.
///
/// A sample is taken from the middle of a segment, where a voice is steadiest —
/// the edges hold the breath before a sentence and the fade after it. Segments too
/// short to identify anybody are left out and take their speaker from a neighbour
/// afterwards, which is better than feeding the clustering a fragment and letting
/// it invent a voice from it.
///
/// The gap between samples exists so the diariser's own segmentation lands on the
/// boundaries we chose rather than inside them: without it, two speakers spliced
/// together read as one continuous stretch of speech.
pub(crate) fn plan_samples(
    segments: &[(u64, u64)],
    sample_ms: u64,
    gap_ms: u64,
    shortest_ms: u64,
) -> Vec<Sample> {
    let mut samples = Vec::new();
    let mut condensed = 0;
    for (index, (start_ms, end_ms)) in segments.iter().enumerate() {
        let length = end_ms.saturating_sub(*start_ms);
        if length < shortest_ms {
            continue;
        }
        let taken = length.min(sample_ms);
        // Centred, so a sample of a long segment is speech rather than a pause.
        let offset = (length - taken) / 2;
        let source_start_ms = start_ms + offset;
        samples.push(Sample {
            segment: index,
            source_start_ms,
            source_end_ms: source_start_ms + taken,
            condensed_start_ms: condensed,
            condensed_end_ms: condensed + taken,
        });
        condensed += taken + gap_ms;
    }
    samples
}

/// Read the diariser's turns over the condensed audio back onto the transcript.
///
/// Each turn is matched to the samples it covers, and each sample carries its
/// answer home to the segment it came from. The mapping is exact because the
/// condensation is ours: a moment in the condensed file belongs to exactly one
/// sample, or to the silence between two.
///
/// Segments with no sample — too short to have been played — are left without a
/// speaker here and inherit one from a neighbour, which is the caller's business.
pub(crate) fn speakers_from_condensed(
    segment_count: usize,
    samples: &[Sample],
    turns: &[SpeakerTurn],
) -> Vec<Option<String>> {
    let mut found: Vec<Option<String>> = vec![None; segment_count];
    for sample in samples {
        // The turn that covers most of this sample, since a turn boundary may
        // fall inside it.
        let best = turns
            .iter()
            .filter_map(|turn| {
                let overlap = sample
                    .condensed_end_ms
                    .min(turn.end_ms)
                    .saturating_sub(sample.condensed_start_ms.max(turn.start_ms));
                (overlap > 0).then_some((overlap, &turn.speaker))
            })
            .max_by_key(|(overlap, _)| *overlap);
        if let Some((_, speaker)) = best
            && let Some(slot) = found.get_mut(sample.segment)
        {
            *slot = Some(speaker.clone());
        }
    }
    found
}

/// Give a speaker to every segment that has none, from the nearest one that has.
///
/// A segment too short to sample is almost always a fragment of the discussion
/// around it — an interjection, a word of agreement — so the neighbour is a better
/// answer than a label of its own. Where nothing has a speaker at all, nothing is
/// invented.
pub(crate) fn fill_gaps(found: &mut [Option<String>]) {
    let known: Vec<usize> = found
        .iter()
        .enumerate()
        .filter(|(_, speaker)| speaker.is_some())
        .map(|(index, _)| index)
        .collect();
    if known.is_empty() {
        return;
    }
    for index in 0..found.len() {
        if found[index].is_some() {
            continue;
        }
        let nearest = known
            .iter()
            .min_by_key(|candidate| candidate.abs_diff(index))
            .expect("at least one segment has a speaker");
        found[index] = found[*nearest].clone();
    }
}

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
    name_in_order(resolved.into_iter().map(|label| label.map(str::to_string)))
}

/// Turn diariser labels into the names a reader sees, numbered by who speaks
/// first. The diariser's own numbering is its clustering order, which is
/// arbitrary and changes between runs, so `speaker_04` becoming `Speaker 1`
/// is the point rather than an accident.
pub(crate) fn name_in_order<I>(labels: I) -> Vec<String>
where
    I: IntoIterator<Item = Option<String>>,
{
    let labels: Vec<Option<String>> = labels.into_iter().collect();
    let mut order: Vec<&str> = Vec::new();
    for label in labels.iter().flatten() {
        if !order.contains(&label.as_str()) {
            order.push(label);
        }
    }
    labels
        .iter()
        .map(|label| match label {
            Some(label) => {
                let index = order
                    .iter()
                    .position(|known| *known == label.as_str())
                    .unwrap_or(0);
                format!("Speaker {}", index + 1)
            }
            // A segment nothing covers keeps the neutral label rather than being
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

/// A plain start and end, for callers that have only the timings and not the text.
impl SegmentTiming for (u64, u64) {
    fn start_ms(&self) -> u64 {
        self.0
    }

    fn end_ms(&self) -> u64 {
        self.1
    }
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

    /// Samples are centred, laid end to end with a gap, and short segments are
    /// left out rather than fed to the clustering as fragments.
    #[test]
    fn sampling_takes_the_middle_of_each_segment_in_order() {
        // 10s, 1s (too short), 3s.
        let segments = [(0, 10_000), (20_000, 21_000), (30_000, 33_000)];
        let samples = plan_samples(&segments, 2_000, 300, 1_500);

        assert_eq!(samples.len(), 2, "the one-second segment is not played");
        assert_eq!(samples[0].segment, 0);
        // Four seconds in, so the sample is speech rather than a sentence's edge.
        assert_eq!(samples[0].source_start_ms, 4_000);
        assert_eq!(samples[0].source_end_ms, 6_000);
        assert_eq!(samples[0].condensed_start_ms, 0);

        assert_eq!(samples[1].segment, 2);
        assert_eq!(samples[1].source_start_ms, 30_500);
        // Follows the first sample and the gap after it.
        assert_eq!(samples[1].condensed_start_ms, 2_300);
    }

    /// A segment shorter than the sample is played whole rather than skipped.
    #[test]
    fn a_short_segment_is_played_in_full() {
        let samples = plan_samples(&[(1_000, 2_600)], 2_000, 300, 1_000);
        assert_eq!(samples[0].source_start_ms, 1_000);
        assert_eq!(samples[0].source_end_ms, 2_600);
        assert_eq!(samples[0].condensed_end_ms, 1_600);
    }

    /// The condensation is ours, so reading the answer back is exact.
    #[test]
    fn each_sample_carries_its_speaker_home() {
        let segments = [(0, 4_000), (10_000, 14_000), (20_000, 24_000)];
        let samples = plan_samples(&segments, 2_000, 300, 1_000);
        // Condensed: 0-2000 first, 2300-4300 second, 4600-6600 third.
        let turns = vec![
            SpeakerTurn {
                start_ms: 0,
                end_ms: 2_100,
                speaker: "speaker_00".into(),
            },
            SpeakerTurn {
                start_ms: 2_200,
                end_ms: 6_800,
                speaker: "speaker_01".into(),
            },
        ];
        let found = speakers_from_condensed(segments.len(), &samples, &turns);
        assert_eq!(
            found,
            vec![
                Some("speaker_00".to_string()),
                Some("speaker_01".to_string()),
                Some("speaker_01".to_string()),
            ]
        );
    }

    /// A turn boundary falling inside a sample gives it to whoever holds most of it.
    #[test]
    fn a_split_sample_goes_to_the_larger_share() {
        let samples = plan_samples(&[(0, 2_000)], 2_000, 300, 500);
        let turns = vec![
            SpeakerTurn {
                start_ms: 0,
                end_ms: 400,
                speaker: "speaker_00".into(),
            },
            SpeakerTurn {
                start_ms: 400,
                end_ms: 2_000,
                speaker: "speaker_01".into(),
            },
        ];
        assert_eq!(
            speakers_from_condensed(1, &samples, &turns),
            vec![Some("speaker_01".to_string())]
        );
    }

    /// A segment too short to play takes its speaker from the discussion around
    /// it, which is nearly always what an interjection belongs to.
    #[test]
    fn segments_without_a_sample_take_a_neighbours_speaker() {
        let mut found = vec![
            Some("speaker_00".to_string()),
            None,
            None,
            Some("speaker_01".to_string()),
        ];
        fill_gaps(&mut found);
        assert_eq!(found[1].as_deref(), Some("speaker_00"));
        assert_eq!(found[2].as_deref(), Some("speaker_01"));
    }

    /// Where nothing was identified, nothing is invented.
    #[test]
    fn nothing_is_filled_in_when_no_speaker_was_found() {
        let mut found = vec![None, None];
        fill_gaps(&mut found);
        assert_eq!(found, vec![None, None]);
    }

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
