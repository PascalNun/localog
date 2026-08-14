use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProjectInput {
    pub name: String,
    pub description: String,
    pub default_language: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMeetingInput {
    pub project_id: String,
    pub title: String,
    pub occurred_at: String,
    pub language: String,
    pub source_name: String,
    pub source_path: Option<String>,
    pub style_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub meeting_count: u32,
    pub default_language: String,
    pub default_style_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MeetingLifecycle {
    Draft,
    SourceReady,
    TranscriptReady,
    ProtocolDraft,
    Reviewed,
    Archived,
}

impl MeetingLifecycle {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(Self::Draft),
            "source_ready" => Some(Self::SourceReady),
            "transcript_ready" => Some(Self::TranscriptReady),
            "protocol_draft" => Some(Self::ProtocolDraft),
            "reviewed" => Some(Self::Reviewed),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingSummary {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub occurred_at: String,
    pub duration_label: Option<String>,
    pub lifecycle: MeetingLifecycle,
    pub language: String,
    pub source_name: Option<String>,
    pub source_byte_count: Option<u64>,
    pub source_media_type: Option<String>,
    pub style_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Queued,
    Running,
    Cancelling,
    Failed,
    Cancelled,
    Interrupted,
    Completed,
}

impl JobState {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "running" => Some(Self::Running),
            "cancelling" => Some(Self::Cancelling),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            "interrupted" => Some(Self::Interrupted),
            "completed" => Some(Self::Completed),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobErrorSummary {
    pub code: String,
    pub title: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobSummary {
    pub id: String,
    pub meeting_id: String,
    pub kind: String,
    pub state: JobState,
    pub outcome: Option<String>,
    pub progress: u8,
    pub progress_bytes: u64,
    pub total_bytes: Option<u64>,
    pub stage: String,
    pub attempt: u32,
    pub error: Option<JobErrorSummary>,
    pub requires_duplicate_confirmation: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub speaker: String,
    pub text: String,
    #[serde(default)]
    pub needs_review: bool,
    /// Words the transcription model itself was unsure of, so the reader can be
    /// asked about the passage rather than discovering the error in a protocol.
    /// Absent from transcripts produced before this was recorded.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uncertain_words: Vec<String>,
}

/// Records whether the optional speaker-separation pass produced usable turns.
/// Older artifacts omit this field and therefore remain explicitly unknown.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerResolution {
    #[default]
    Unknown,
    Unavailable,
    Failed,
    Resolved,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptDocument {
    pub schema_version: u8,
    pub meeting_id: String,
    pub revision_id: String,
    pub language: String,
    pub speaker_resolution: SpeakerResolution,
    pub segments: Vec<TranscriptSegment>,
    pub base_revision_id: String,
    pub is_dirty: bool,
    pub save_state: String,
    pub saved_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolRevisionSummary {
    pub id: String,
    pub ordinal: u32,
    pub status: String,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolDocument {
    pub meeting_id: String,
    pub revision_id: String,
    pub transcript_revision_id: String,
    pub markdown: String,
    pub style_id: String,
    pub review_state: String,
    pub is_dirty: bool,
    pub save_state: String,
    pub saved_at_ms: i64,
    pub revisions: Vec<ProtocolRevisionSummary>,
    /// What the run that produced this draft found out about its own result.
    /// Absent for drafts written before it was recorded.
    pub evidence: Option<ProtocolEvidence>,
}

/// Facts about a draft that can be established without a model and without a
/// reader: a quantity was either stated in the meeting or it was not, and either
/// survives into the protocol or does not.
///
/// This is evidence for review, never a verdict. A machine opinion placed in front
/// of a person asks them to read less carefully, and reading carefully is the only
/// check in this product that reliably works.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolEvidence {
    pub quantities_stated: u32,
    pub quantities_accounted: u32,
    /// Figures the draft states that no part of the meeting did. Wrong under any
    /// style, unlike how much a draft keeps, which is what its style asked for.
    #[serde(default)]
    pub quantities_invented: Vec<String>,
    pub characters_spoken: u32,
    pub characters_written: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolStyle {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Styles follow the meeting language; they do not impose an interface language.
    pub language: String,
    pub density: ProtocolDensity,
}

/// How much room a style spends saying a thing — not which things it says.
///
/// A style is how a person wants information conveyed. It is not a decision
/// about what information belongs in a protocol, because what was decided, asked
/// and agreed belongs in one whatever its form. A terse protocol says the same
/// thing in fewer words; it does not say less.
///
/// The evidence is a meeting for which a person wrote three documents of 8,915,
/// 12,927 and 18,212 characters. The quantities the meeting stated survived into
/// all three at almost the same rate — 14, 15 and 15 of 24 — so what compression
/// removed was elaboration and context, not content.
///
/// The reason to hold to that is not tidiness, it is what the two mistakes cost.
/// A person reviewing a draft deletes an unwanted line in a keystroke. Noticing
/// that something is *absent* means realising that a thing they discussed is not
/// there at all, while reading a document that reads perfectly well without it.
/// Someone may also choose a short style and change their mind once they see the
/// draft, and they can only cut what was put in front of them.
///
/// So the directives below say how much to write per point and never which points
/// to drop, and the checks recorded against a job apply at full strength at every
/// setting: a terser protocol is not entitled to lose anything.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolDensity {
    /// Full prose, with the context a reader who was absent would need.
    Comprehensive,
    /// Plain statements, without elaboration.
    #[default]
    Concise,
    /// A line per point.
    Terse,
}

impl ProtocolDensity {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "comprehensive" => Some(Self::Comprehensive),
            "concise" => Some(Self::Concise),
            "terse" => Some(Self::Terse),
            _ => None,
        }
    }

    /// What the model is told, in the same voice as a style's other instructions.
    ///
    /// Each governs how much is written per point, never which points survive. No
    /// target length is given: a number of words invites padding to reach it, and
    /// a meeting that warrants three pages should not be stretched to five because
    /// the style asked for prose.
    pub fn directive(self) -> &'static str {
        match self {
            Self::Comprehensive => {
                "Write in full prose, giving each point the context a reader who was not present would need to understand it."
            }
            Self::Concise => {
                "Write plainly and without elaboration. State each point, and its reason where one was given, in a sentence or two."
            }
            Self::Terse => {
                "Write as briefly as the meaning allows, roughly a line per point. Omit nothing that was decided, asked, agreed or stated as a figure - say it in fewer words instead."
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyEntry {
    pub id: String,
    pub term: String,
    pub category: String,
    pub scope: String,
    pub project_id: Option<String>,
    /// A switched-off term stays in the library but reaches neither the
    /// transcriber nor the protocol.
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
}

fn enabled_by_default() -> bool {
    true
}

/// A term as the library editor supplies it. Without an `id` this is a new term.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyDraft {
    pub id: Option<String>,
    pub term: String,
    pub category: String,
    pub scope: String,
    pub project_id: Option<String>,
    #[serde(default = "enabled_by_default")]
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot {
    pub projects: Vec<ProjectSummary>,
    pub meetings: Vec<MeetingSummary>,
    pub jobs: Vec<JobSummary>,
    pub transcripts: HashMap<String, TranscriptDocument>,
    pub protocols: HashMap<String, ProtocolDocument>,
    pub styles: Vec<ProtocolStyle>,
    pub vocabulary: Vec<VocabularyEntry>,
    pub active_meeting_id: Option<String>,
    pub active_route: Option<String>,
}
