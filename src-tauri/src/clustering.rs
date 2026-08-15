//! Grouping speaker embeddings into voices.
//!
//! One vector per transcript segment arrives from the embedding runtime, and the
//! question is which of them are the same person. That question is arithmetic over
//! a few hundred short vectors, not another pass over the audio, which is the whole
//! reason this exists: the diariser answered it by re-reading the recording, so
//! every different answer cost eight minutes and the number of speakers had to be
//! settled in advance by somebody who often does not know it.
//!
//! Here the merging is done once and every answer read off it afterwards. Asking
//! for eight voices, or eleven, or "however many there are" costs nothing more.

// Not yet on the generation path: nothing produces the vectors inside the
// application, because the embedding runtime is still the study in
// spikes/speaker-embedding/. The arithmetic is landed first and separately
// because it is the part that can be proven without a runtime, and it is proven
// against that study's measurements on a real meeting.
#![allow(dead_code)]

/// Above this, a merge is judged to be joining a person to themselves rather than
/// to somebody else, so the count is read as the number of groups left when the
/// merging first falls below it.
///
/// Fitted to what has been measured and not to a principle. On a fixture with
/// recorded ground truth the merge similarities fall 0.810, 0.790, 0.777, then
/// 0.255, so anything between those two is right there. On the reference meeting,
/// which is real videoconference audio and has no known speaker count, this finds
/// twelve voices where the diariser's own automatic mode finds sixty-seven.
///
/// One fixture and one unlabelled meeting is not enough to fix a constant. It is
/// used to offer an estimate the user can change, never to assert a number.
pub(crate) const SAME_VOICE_FLOOR: f32 = 0.20;

/// The order in which segments merged into voices, and how alike each pair was.
///
/// Built once. Any number of speakers can then be read from it, which is what
/// makes offering the choice after the fact affordable.
pub(crate) struct Merged {
    /// For each step, the two groups joined and how similar they were. Groups are
    /// named by the lowest segment they contain, which keeps the naming stable.
    steps: Vec<Step>,
    count: usize,
}

struct Step {
    into: usize,
    from: usize,
    similarity: f32,
}

/// Average-linkage agglomerative clustering over cosine similarity.
///
/// Average linkage rather than nearest or furthest neighbour because a voice
/// drifts across a long meeting — different microphones, connection quality,
/// compression — and nearest-neighbour chains those drifting samples into one
/// enormous cluster while furthest-neighbour refuses to admit the same person
/// twice. Average asks whether a group as a whole sounds like another group.
///
/// Distances are updated by the Lance-Williams rule rather than recomputed, so
/// this is quadratic in the number of segments rather than cubic. A meeting of
/// several hundred segments is milliseconds either way, but a day-long recording
/// is not.
pub(crate) fn merge(vectors: &[Vec<f32>]) -> Merged {
    let count = vectors.len();
    let normalized: Vec<Vec<f32>> = vectors.iter().map(|vector| normalize(vector)).collect();
    let mut similarity = vec![0.0f32; count * count];
    for i in 0..count {
        for j in (i + 1)..count {
            let value = dot(&normalized[i], &normalized[j]);
            similarity[i * count + j] = value;
            similarity[j * count + i] = value;
        }
    }

    let mut size = vec![1usize; count];
    let mut alive: Vec<usize> = (0..count).collect();
    let mut steps = Vec::new();
    while alive.len() > 1 {
        let mut best = (0usize, 0usize);
        let mut score = f32::NEG_INFINITY;
        for (position, &i) in alive.iter().enumerate() {
            for &j in &alive[position + 1..] {
                let value = similarity[i * count + j];
                if value > score {
                    score = value;
                    best = (i, j);
                }
            }
        }
        let (into, from) = best;
        steps.push(Step {
            into,
            from,
            similarity: score,
        });
        for &other in &alive {
            if other == into || other == from {
                continue;
            }
            // The joined group's likeness to anything else is the size-weighted
            // mean of its parts', which is what average linkage means.
            let merged = (size[into] as f32 * similarity[into * count + other]
                + size[from] as f32 * similarity[from * count + other])
                / (size[into] + size[from]) as f32;
            similarity[into * count + other] = merged;
            similarity[other * count + into] = merged;
        }
        size[into] += size[from];
        alive.retain(|&group| group != from);
    }

    Merged { steps, count }
}

impl Merged {
    /// Which voice each segment belongs to, if there are this many voices.
    ///
    /// Numbered by the order they first speak, so the first person to talk is
    /// voice zero. Without that the numbering would follow the merge order, which
    /// is arbitrary and changes when a segment changes.
    pub(crate) fn voices(&self, wanted: usize) -> Vec<usize> {
        let wanted = wanted.clamp(1, self.count.max(1));
        let mut belongs: Vec<usize> = (0..self.count).collect();
        // Replay the merges, stopping when the wanted number of groups remain.
        let stop = self.count.saturating_sub(wanted);
        for step in self.steps.iter().take(stop) {
            let (into, from) = (step.into, step.from);
            for group in belongs.iter_mut() {
                if *group == from {
                    *group = into;
                }
            }
        }
        let mut order: Vec<usize> = Vec::new();
        for group in &belongs {
            if !order.contains(group) {
                order.push(*group);
            }
        }
        belongs
            .iter()
            .map(|group| order.iter().position(|known| known == group).unwrap_or(0))
            .collect()
    }

    /// How many voices the audio holds, by stopping where the merging stops
    /// joining people to themselves and starts joining them to each other.
    ///
    /// Measured on a fixture with recorded ground truth, the similarity of each
    /// merge falls off a cliff exactly there: 0.810, 0.790, 0.777, then 0.255. The
    /// floor is where that fall is judged to have happened.
    ///
    /// This is an estimate and is offered as one. The floor is a constant chosen
    /// against recordings, and no amount of arithmetic turns it into knowledge of
    /// who was in the room.
    pub(crate) fn voices_above(&self, floor: f32) -> usize {
        let joined = self
            .steps
            .iter()
            .take_while(|step| step.similarity >= floor)
            .count();
        self.count.saturating_sub(joined).max(1)
    }

    /// The similarity of each merge, from the most alike downwards. Evidence for a
    /// person deciding whether the estimate is believable, not a number to act on.
    pub(crate) fn similarities(&self) -> Vec<f32> {
        self.steps.iter().map(|step| step.similarity).collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.count
    }
}

/// Read the vectors the embedding sidecar wrote.
///
/// Returns the segment each vector belongs to alongside it. Segments too short to
/// place a voice from are absent rather than zeroed, so a gap here means "nothing
/// was heard" rather than "a voice of silence", and the caller can leave those
/// segments unattributed instead of grouping them with whoever else happens to be
/// quiet.
pub(crate) fn read_vectors(path: &std::path::Path) -> Result<(Vec<u32>, Vec<Vec<f32>>), String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() < 16 || &bytes[0..4] != b"LLEM" {
        return Err("The speaker pass did not write recognisable embeddings.".into());
    }
    let word =
        |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    let version = word(4);
    if version != 1 {
        return Err(format!(
            "These embeddings are version {version}, which this build does not read."
        ));
    }
    let count = word(8) as usize;
    let dimensions = word(12) as usize;
    if dimensions == 0 {
        return Err("The embeddings describe no dimensions.".into());
    }
    let stride = 4 + dimensions * 4;
    if bytes.len() < 16 + count * stride {
        return Err("The embeddings are shorter than they claim to be.".into());
    }
    let mut segments = Vec::with_capacity(count);
    let mut vectors = Vec::with_capacity(count);
    for row in 0..count {
        let at = 16 + row * stride;
        segments.push(word(at));
        let mut vector = Vec::with_capacity(dimensions);
        for value in 0..dimensions {
            let of = at + 4 + value * 4;
            vector.push(f32::from_le_bytes([
                bytes[of],
                bytes[of + 1],
                bytes[of + 2],
                bytes[of + 3],
            ]));
        }
        vectors.push(vector);
    }
    Ok((segments, vectors))
}

fn normalize(vector: &[f32]) -> Vec<f32> {
    let length = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if length <= f32::EPSILON {
        return vector.to_vec();
    }
    vector.iter().map(|value| value / length).collect()
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Three obvious voices, each a direction in space, with the segments
    /// interleaved the way a conversation actually runs.
    fn three_voices() -> Vec<Vec<f32>> {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let c = vec![0.0, 0.0, 1.0];
        // Slight variation, because the same person never measures identically.
        let nudge = |v: &Vec<f32>, by: f32| vec![v[0] + by, v[1] + by * 0.5, v[2] - by * 0.3];
        vec![
            nudge(&a, 0.02),
            nudge(&b, 0.01),
            nudge(&a, -0.01),
            nudge(&c, 0.03),
            nudge(&b, -0.02),
            nudge(&a, 0.04),
        ]
    }

    #[test]
    fn segments_of_one_voice_end_up_together() {
        let voices = merge(&three_voices()).voices(3);
        // Segments 0, 2 and 5 are the same person; 1 and 4 another; 3 a third.
        assert_eq!(voices[0], voices[2]);
        assert_eq!(voices[0], voices[5]);
        assert_eq!(voices[1], voices[4]);
        assert_ne!(voices[0], voices[1]);
        assert_ne!(voices[0], voices[3]);
        assert_ne!(voices[1], voices[3]);
    }

    /// The first person to speak is the first voice, whatever order the merging
    /// happened to take.
    #[test]
    fn voices_are_numbered_by_who_speaks_first() {
        let voices = merge(&three_voices()).voices(3);
        assert_eq!(voices[0], 0);
        assert_eq!(voices[1], 1);
        assert_eq!(voices[3], 2);
    }

    /// Every count is read from one merge, which is the point of the whole module.
    #[test]
    fn any_number_of_voices_can_be_read_from_one_merge() {
        let merged = merge(&three_voices());
        assert_eq!(distinct(&merged.voices(1)), 1);
        assert_eq!(distinct(&merged.voices(2)), 2);
        assert_eq!(distinct(&merged.voices(3)), 3);
        assert_eq!(distinct(&merged.voices(6)), 6);
    }

    /// Asking for more voices than there are segments, or for none at all, must
    /// give an answer rather than panicking on a meeting nobody expected.
    #[test]
    fn absurd_requests_are_answered_rather_than_fatal() {
        let merged = merge(&three_voices());
        assert_eq!(distinct(&merged.voices(0)), 1);
        assert_eq!(distinct(&merged.voices(99)), 6);
    }

    /// The count falls out of where the merging stops joining a person to
    /// themselves, which is the question a user cannot reliably answer.
    #[test]
    fn the_number_of_voices_is_estimated_without_being_told() {
        let merged = merge(&three_voices());
        assert_eq!(merged.voices_above(0.5), 3);
    }

    /// One segment is a meeting of one, not a crash.
    #[test]
    fn a_single_segment_is_one_voice() {
        let merged = merge(&[vec![1.0, 0.0]]);
        assert_eq!(merged.voices(1), vec![0]);
        assert_eq!(merged.voices_above(0.5), 1);
    }

    /// A vector of nothing has no direction to compare, and must not divide by
    /// its own length.
    #[test]
    fn a_silent_embedding_does_not_divide_by_zero() {
        let merged = merge(&[vec![0.0, 0.0], vec![1.0, 0.0]]);
        assert_eq!(merged.voices(2).len(), 2);
        assert!(merged.similarities().iter().all(|value| value.is_finite()));
    }

    /// A file that is not what it claims must be refused rather than read as
    /// whatever the bytes happen to mean.
    #[test]
    fn embeddings_that_are_not_embeddings_are_refused() {
        let path = std::env::temp_dir().join("localog-not-embeddings.bin");
        std::fs::write(&path, b"not a vector file at all").expect("a file");
        assert!(read_vectors(&path).is_err());
        std::fs::write(
            &path,
            b"LLEM\x09\x00\x00\x00\x01\x00\x00\x00\x02\x00\x00\x00",
        )
        .expect("a file");
        let error = read_vectors(&path).expect_err("a version refusal");
        assert!(error.contains("version 9"), "{error}");
        let _ = std::fs::remove_file(&path);
    }

    /// Check the grouping against the study that justified building it.
    ///
    /// The spike in `spikes/speaker-embedding/` measured the same vectors in
    /// Python: eleven voices gives `388 120 102 17 17 11 10 7 1 1 1`. A port that
    /// agrees with the measurement is a port; one that does not is a different
    /// algorithm wearing its name.
    ///
    /// Set LOCALOG_VECTORS to the JSON `embed-segments` writes.
    #[test]
    #[ignore = "requires embeddings from the speaker-embedding study"]
    fn matches_the_study_that_justified_it() {
        let path = std::env::var("LOCALOG_VECTORS").expect("a vectors file");
        let (segments, vectors) =
            read_vectors(std::path::Path::new(&path)).expect("readable embeddings");
        assert_eq!(segments.len(), vectors.len());

        let merged = merge(&vectors);
        let voices = merged.voices(11);
        let mut sizes = vec![0usize; 11];
        for voice in &voices {
            sizes[*voice] += 1;
        }
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        println!(
            "{} segments, sizes at eleven voices: {sizes:?}",
            merged.len()
        );
        for floor in [0.14f32, 0.16, 0.18, 0.20, 0.25] {
            println!(
                "  floor {floor:.2} -> {} voices",
                merged.voices_above(floor)
            );
        }
        assert_eq!(sizes[0], 388, "the dominant voice should hold 388 segments");
        assert_eq!(&sizes[..3], &[388, 120, 102]);
        // Twelve, not the eleven the study first reported: the Python there
        // counted the groups left *after* the merge it had just refused, and
        // porting it found the off-by-one.
        assert_eq!(merged.voices_above(0.20), 12);
    }

    fn distinct(voices: &[usize]) -> usize {
        let mut seen: Vec<usize> = Vec::new();
        for voice in voices {
            if !seen.contains(voice) {
                seen.push(*voice);
            }
        }
        seen.len()
    }
}
