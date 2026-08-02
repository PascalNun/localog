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
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VocabularyEntry {
    pub id: String,
    pub term: String,
    pub category: String,
    pub scope: String,
    pub project_id: Option<String>,
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
