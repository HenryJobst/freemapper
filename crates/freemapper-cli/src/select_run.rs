//! The run routes of the selection page: start a run, follow it, cancel it and open the project
//! folder (ADR 0090). The page sends the seven options and nothing else; the folder, the cache,
//! kind, scale and title are the ones the selector started with and the saved project holds.

use serde_json::{Value, json};

use crate::jobs::{JobState, JobStatus};
use crate::select::AppState;

/// The state name the page sees: `done` where MCP says `finished`.
fn state_name(state: JobState) -> &'static str {
    match state {
        JobState::Running => "running",
        JobState::Finished => "done",
        JobState::Failed => "failed",
        JobState::Cancelled => "cancelled",
    }
}

/// A job as the page's status answer shows it. The lines are the engine's plain lines, the step
/// is a key (`step-fetch`, ...) that the page turns into its own text.
pub(crate) fn status_json(status: &JobStatus) -> Value {
    let files: Vec<Value> = status
        .report
        .iter()
        .flat_map(|report| &report.files)
        .map(|f| json!({ "kind": f.kind, "path": f.relative_path, "bytes": f.bytes }))
        .collect();
    json!({
        "state": state_name(status.state),
        "job_id": status.job_id,
        "phase": status.phase.map(|(step, index, total)| {
            json!({ "step": step.label_key(), "index": index, "total": total })
        }),
        "lines": status.lines,
        "next": status.next,
        "warnings": status.warnings,
        "files": files,
        "kept_map": status.report.as_ref().is_some_and(|r| r.kept_map),
        "summary": status.report.as_ref().map(|r| r.summary.as_str()),
        "error": status.error,
    })
}

/// The run of this page's folder that is going on now, for `api/project`; null when none runs.
pub(crate) fn running_json(state: &AppState) -> Value {
    state
        .hub
        .current_for(&state.dir)
        .filter(|status| status.state == JobState::Running)
        .map_or(Value::Null, |status| status_json(&status))
}
