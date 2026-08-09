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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptDocument {
    pub schema_version: u8,
    pub meeting_id: String,
    pub revision_id: String,
    pub language: String,
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

/// How much prose a style wants around the facts.
///
/// Measured on one meeting for which a person wrote three documents of different
/// lengths: the quantities the meeting stated survived into all three at almost
/// the same rate — 15, 14 and 15 of 24 — across a two-fold range in length. What
/// compression removes is narrative, context and the account of the discussion,
/// not the figures.
///
/// So this steers volume, and nothing else. It is deliberately not a licence to
/// drop facts, and the checks recorded against a job apply at every density.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolDensity {
    /// A record someone absent could follow: the discussion as well as its result.
    Comprehensive,
    /// The result and enough of the reasoning to understand it.
    #[default]
    Selective,
    /// What was decided and what happens next, and little else.
    Minimal,
}

impl ProtocolDensity {
    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "comprehensive" => Some(Self::Comprehensive),
            "selective" => Some(Self::Selective),
            "minimal" => Some(Self::Minimal),
            _ => None,
        }
    }

    /// What the model is told, in the same voice as a style's other instructions.
    ///
    /// Each says what to leave out rather than how long to be. A length in words
    /// invites padding to reach it, and a meeting that warrants three pages should
    /// not be stretched to five because the style says so.
    pub fn directive(self) -> &'static str {
        match self {
            Self::Comprehensive => {
                "Write a full record. Someone who was absent must be able to follow what was discussed, not only what was concluded. Keep the reasoning, the alternatives considered, and the context."
            }
            Self::Selective => {
                "Write the result and enough of the reasoning to make it understandable. Leave out the back-and-forth of the discussion, but keep why a conclusion was reached."
            }
            Self::Minimal => {
                "Write only what was decided, what remains open, and what happens next. Leave out the discussion entirely."
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
