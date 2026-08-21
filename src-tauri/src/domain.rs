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

/// Something a protocol of this style must actually contain.
///
/// The replacement for `required_sections`, which held English section names while
/// the protocol is written in the meeting's language — matching "Actions" against
/// "Aufgaben" needs something this application does not have, so it was never
/// checked anywhere and could not be.
///
/// These are structural: a table is a table in every language. Only what can be
/// checked belongs here, which is why the list is short and grows only when a check
/// exists to go with it.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StructuralExpectation {
    /// A markdown table, which is what a list of agreed actions is.
    ActionTable,
}

/// How a project's protocols look, as opposed to what they say.
///
/// Kept apart from the protocol style on purpose. A style decides what belongs in
/// the document and in what order; this decides how it is set. Conflating them is
/// how "make the headings smaller" turns into a different protocol.
///
/// Held by the project rather than by each protocol, because the reason anybody
/// sets it is that a firm's protocols should look alike. Every value is one of a
/// short list rather than a number, so that a document cannot end up at 11.5pt
/// Helvetica by accident.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentAppearance {
    pub font: DocumentFont,
    /// The body size in points, as it will print.
    pub body_size: u8,
    pub heading_scale: Scale,
    pub line_spacing: Spacing,
    pub page_width: PageWidth,
}

impl Default for DocumentAppearance {
    fn default() -> Self {
        Self {
            font: DocumentFont::Barlow,
            body_size: 11,
            heading_scale: Scale::Standard,
            line_spacing: Spacing::Comfortable,
            page_width: PageWidth::A4,
        }
    }
}

/// What repeats at the top and bottom of every printed page.
///
/// A document property rather than body content: it is not part of what the meeting
/// said, and editing it in the document would put it in the middle of the text.
///
/// Each slot is a list of fields rather than free text, so that "page 3 of 6" can be
/// counted rather than typed. Custom text is one of the field kinds, which is how
/// anything the list does not cover gets in.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PageFurniture {
    pub header: FurnitureRow,
    pub footer: FurnitureRow,
    /// A title page usually carries its own heading and wants nothing repeated on it.
    pub skip_first_page: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct FurnitureRow {
    pub left: Vec<FurnitureField>,
    pub centre: Vec<FurnitureField>,
    pub right: Vec<FurnitureField>,
}

impl FurnitureRow {
    pub fn is_empty(&self) -> bool {
        self.left.is_empty() && self.centre.is_empty() && self.right.is_empty()
    }
}

/// One thing that can appear in a header or a footer.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind", content = "value")]
pub enum FurnitureField {
    ProjectName,
    MeetingTitle,
    MeetingDate,
    DocumentType,
    ProtocolStatus,
    PageNumber,
    /// "Page 3 of 6", which needs the count and so cannot be two separate fields.
    PageOfCount,
    Text(String),
}

/// The typefaces a protocol may be set in.
///
/// Barlow ships with the application. The others are asked of the system, and are
/// here because a firm's house style is not something LocaLog gets to overrule —
/// each is on every macOS and Windows machine.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentFont {
    Barlow,
    Georgia,
    TimesNewRoman,
    Arial,
    Calibri,
}

impl DocumentFont {
    /// What to ask a browser for, in order.
    pub fn css_stack(self) -> &'static str {
        match self {
            Self::Barlow => "'Barlow', system-ui, sans-serif",
            Self::Georgia => "Georgia, 'Times New Roman', serif",
            Self::TimesNewRoman => "'Times New Roman', Times, serif",
            Self::Arial => "Arial, Helvetica, sans-serif",
            Self::Calibri => "Calibri, Carlito, system-ui, sans-serif",
        }
    }

    /// What to name in a Word document, which asks for one family and no fallback.
    pub fn word_name(self) -> &'static str {
        match self {
            Self::Barlow => "Barlow",
            Self::Georgia => "Georgia",
            Self::TimesNewRoman => "Times New Roman",
            Self::Arial => "Arial",
            Self::Calibri => "Calibri",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Scale {
    Compact,
    Standard,
    Large,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Spacing {
    Compact,
    Comfortable,
    Spacious,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PageWidth {
    Narrow,
    Standard,
    Wide,
    /// The text column of an A4 page, which is what most of these documents become.
    A4,
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
    pub appearance: DocumentAppearance,
    pub furniture: PageFurniture,
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
    /// Tasks the draft records with nobody against them, so a person can put a
    /// name to them while the meeting is still fresh. Not a fault: the styles tell
    /// the model never to invent an owner, so an empty one can be an accurate
    /// record of a meeting that agreed something and assigned it to nobody.
    #[serde(default)]
    pub tasks_unowned: Vec<String>,
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
    /// The word this is stored as, which is also the word the CHECK constraint
    /// and every migration use.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Comprehensive => "comprehensive",
            Self::Concise => "concise",
            Self::Terse => "terse",
        }
    }

    pub fn directive(self) -> &'static str {
        match self {
            Self::Comprehensive => {
                // The second sentence used to sit in the formal style's own
                // instruction list, where it contradicted two of the three settings
                // the moment density became a choice: a style cannot both refuse to
                // compress and be asked for a line per point.
                //
                // It is kept rather than dropped because it was measured to earn its
                // place here. With it, three drafts at this setting landed within
                // 189 characters of one another; without it they scattered across
                // 1,051. It is what stops "full prose" being read as a licence to
                // summarise.
                "Write in full prose, giving each point the context a reader who was not present would need to understand it. Write at whatever length the material requires: this is a record rather than a summary, and a reader who was absent must be able to follow what was discussed and what follows from it."
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

/// A spelling correction somebody approved, and what to do with it.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppliedCorrection {
    pub wrong: String,
    pub right: String,
    /// Segments whose occurrences to change. Empty means every one of them, which is
    /// what somebody means when they correct a project's name.
    #[serde(default)]
    pub kept_segment_ids: Vec<String>,
    /// Whether to keep the spelling for the project's future meetings.
    #[serde(default)]
    pub remember: bool,
}
