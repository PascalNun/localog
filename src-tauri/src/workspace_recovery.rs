//! Recovery belongs to process startup, not every frontend reload/read.
use crate::{
    domain::WorkspaceSnapshot,
    storage::{Result, StorageError, WorkspaceRepository},
};
use std::path::Path;
use std::sync::{Arc, Mutex};
#[derive(Clone, Default)]
pub(crate) struct WorkspaceRecovery(Arc<Mutex<bool>>);
impl WorkspaceRecovery {
    pub fn load(&self, root: &Path) -> Result<WorkspaceSnapshot> {
        let mut recovered = self
            .0
            .lock()
            .map_err(|_| StorageError::InvalidData("workspaceNotOpened"))?;
        if !*recovered {
            crate::models::discard_staged_downloads(root);
            crate::imports::reconcile_imports(root)?;
            let snapshot = crate::processing::reconcile(root)?;
            *recovered = true;
            Ok(snapshot)
        } else {
            WorkspaceRepository::open(root)?.workspace_snapshot()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{NewMeetingInput, NewProjectInput};
    #[test]
    fn reading_again_does_not_interrupt_a_live_job() {
        let temp = tempfile::tempdir().unwrap();
        let recovery = WorkspaceRecovery::default();
        recovery.load(temp.path()).unwrap();
        let mut repo = WorkspaceRepository::open(temp.path()).unwrap();
        let project = repo
            .create_project(NewProjectInput {
                name: "Synthetic".into(),
                description: String::new(),
                default_language: "English".into(),
                names: vec![],
            })
            .unwrap();
        let source = temp.path().join("synthetic.wav");
        std::fs::write(&source, b"synthetic").unwrap();
        repo.create_meeting(NewMeetingInput {
            project_id: project.id,
            title: "Synthetic".into(),
            occurred_at: "2026-10-23".into(),
            language: "English".into(),
            source_name: "synthetic.wav".into(),
            source_path: Some(source.to_string_lossy().into()),
            style_id: "style-formal".into(),
        })
        .unwrap();
        repo.connection
            .execute(
                "UPDATE jobs SET state = 'running', stage = 'copying_source'",
                [],
            )
            .unwrap();
        recovery.load(temp.path()).unwrap();
        assert_eq!(
            repo.connection
                .query_row("SELECT state FROM jobs", [], |r| r.get::<_, String>(0))
                .unwrap(),
            "running"
        );
    }
}
