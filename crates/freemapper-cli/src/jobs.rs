//! The one table of pipeline jobs of a process. The MCP tools and the selection pages share it, so
//! a run started in one is visible in the other and only one run exists at a time (ADR 0090).

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde_json::{Value, json};

use crate::pipeline::{Event, RunControl, RunReport, Step};

/// Progress lines kept per job.
const KEPT_LINES: usize = 200;
/// Longest progress line in characters; longer ones are cut.
const LINE_CHARS: usize = 500;
/// Progress lines the MCP status shows (the last ones).
const MCP_LINES: usize = 50;

pub type JobId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    Running,
    Finished,
    Failed,
    Cancelled,
}

impl JobState {
    pub fn name(self) -> &'static str {
        match self {
            JobState::Running => "running",
            JobState::Finished => "finished",
            JobState::Failed => "failed",
            JobState::Cancelled => "cancelled",
        }
    }
}

/// What a job runs: gets the cancel flag and a progress sink.
pub type Runner =
    Box<dyn FnOnce(&RunControl, &mut dyn FnMut(Event)) -> Result<RunReport, String> + Send>;

#[derive(Debug)]
struct Job {
    dir: PathBuf,
    state: JobState,
    phase: Option<(Step, u8, u8)>,
    lines: VecDeque<String>,
    /// Lines ever sent; the first kept line has the number `sent - lines.len()`.
    sent: usize,
    warnings: Vec<String>,
    report: Option<RunReport>,
    error: Option<String>,
    control: RunControl,
}

/// A copy of a job at one moment.
#[derive(Debug, Clone)]
pub struct JobStatus {
    pub job_id: JobId,
    pub dir: PathBuf,
    pub state: JobState,
    pub phase: Option<(Step, u8, u8)>,
    /// The kept lines from number `since` on.
    pub lines: Vec<String>,
    /// The number to ask for next time.
    pub next: usize,
    pub warnings: Vec<String>,
    pub report: Option<RunReport>,
    pub error: Option<String>,
}

impl JobStatus {
    /// The MCP status: the last lines, the summary as `result`, and the phase.
    pub fn to_json(&self) -> Value {
        let first = self.lines.len().saturating_sub(MCP_LINES);
        json!({
            "job_id": self.job_id,
            "dir": self.dir,
            "state": self.state.name(),
            "phase": self.phase.map(|(step, index, total)| {
                json!({ "step": step.label_key(), "index": index, "total": total })
            }),
            "lines": &self.lines[first..],
            "next": self.next,
            "warnings": self.warnings,
            "result": self.report.as_ref().map(|r| r.summary.as_str()),
            "error": self.error,
        })
    }
}

#[derive(Debug, Default)]
struct Table {
    next: u64,
    jobs: HashMap<JobId, Job>,
}

/// The pipeline jobs of this process.
#[derive(Debug, Default)]
pub struct JobHub {
    table: Mutex<Table>,
}

/// One spelling per folder (`..`, symlinks and letter case resolved as far as the folder or its
/// nearest existing parent exists), to find the page or job of a folder.
pub fn folder_key(dir: &Path) -> PathBuf {
    for ancestor in dir.ancestors() {
        if let Ok(canonical) = std::fs::canonicalize(ancestor) {
            let rest = dir.strip_prefix(ancestor).unwrap_or(Path::new(""));
            return canonical.join(rest);
        }
    }
    dir.to_path_buf()
}

fn cut(line: &str) -> String {
    match line.char_indices().nth(LINE_CHARS) {
        Some((end, _)) => line[..end].to_string(),
        None => line.to_string(),
    }
}

impl JobHub {
    /// The table; a panic elsewhere never makes it unusable.
    fn table(&self) -> MutexGuard<'_, Table> {
        self.table.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Registers a running job for `dir`; refused while any job runs, because all projects
    /// share the download cache.
    pub(crate) fn register(&self, dir: &Path) -> Result<JobId, String> {
        let mut table = self.table();
        if let Some((id, job)) = table
            .jobs
            .iter()
            .find(|(_, job)| job.state == JobState::Running)
        {
            return Err(format!(
                "{id} is still running for {}; one pipeline runs at a time because projects \
                 share the download cache. Start this one when project_status reports finished \
                 or failed",
                job.dir.display()
            ));
        }
        table.next += 1;
        let id = format!("job-{}", table.next);
        table.jobs.insert(
            id.clone(),
            Job {
                dir: dir.to_path_buf(),
                state: JobState::Running,
                phase: None,
                lines: VecDeque::new(),
                sent: 0,
                warnings: Vec::new(),
                report: None,
                error: None,
                control: RunControl::default(),
            },
        );
        Ok(id)
    }

    fn control(&self, id: &str) -> Option<RunControl> {
        self.table().jobs.get(id).map(|job| job.control.clone())
    }

    fn progress(&self, id: &str, event: Event) {
        let mut table = self.table();
        let Some(job) = table.jobs.get_mut(id) else {
            return;
        };
        match event {
            Event::Phase { step, index, total } => job.phase = Some((step, index, total)),
            Event::Line(line) => {
                let line = cut(line);
                if line.starts_with("warning:") {
                    job.warnings.push(line.clone());
                }
                if job.lines.len() == KEPT_LINES {
                    job.lines.pop_front();
                }
                job.lines.push_back(line);
                job.sent += 1;
            }
            Event::Written(_) => {}
        }
    }

    fn finish(&self, id: &str, outcome: Result<RunReport, String>) {
        let mut table = self.table();
        let Some(job) = table.jobs.get_mut(id) else {
            return;
        };
        match outcome {
            Ok(report) => {
                job.state = JobState::Finished;
                job.report = Some(report);
            }
            Err(error) if job.control.is_cancelled() => {
                job.state = JobState::Cancelled;
                job.error = Some(error);
            }
            Err(error) => {
                job.state = JobState::Failed;
                job.error = Some(error);
            }
        }
    }

    /// Runs a registered job to its end and records the outcome; a panic is recorded as a
    /// failure, so the job never stays running.
    pub(crate) fn run_registered(&self, id: &str, runner: Runner) {
        let Some(control) = self.control(id) else {
            return;
        };
        let mut progress = |event: Event| self.progress(id, event);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runner(&control, &mut progress)
        }))
        .unwrap_or_else(|panic| {
            let message = panic
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panic.downcast_ref::<String>().cloned())
                .unwrap_or_default();
            Err(format!("internal error: {message}"))
        });
        self.finish(id, outcome);
    }

    /// Starts `runner` on a blocking thread of the tokio runtime and returns its job id. Refused
    /// while a job runs.
    pub fn start(self: &Arc<Self>, dir: &Path, runner: Runner) -> Result<JobId, String> {
        let id = self.register(dir)?;
        let hub = Arc::clone(self);
        let job_id = id.clone();
        tokio::task::spawn_blocking(move || hub.run_registered(&job_id, runner));
        Ok(id)
    }

    /// The job `id` with the lines from number `since` on.
    pub fn status(&self, id: &str, since: usize) -> Option<JobStatus> {
        let table = self.table();
        let job = table.jobs.get(id)?;
        Some(Self::snapshot(id, job, since))
    }

    fn snapshot(id: &str, job: &Job, since: usize) -> JobStatus {
        let first = job.sent - job.lines.len();
        let skip = since.saturating_sub(first).min(job.lines.len());
        JobStatus {
            job_id: id.to_string(),
            dir: job.dir.clone(),
            state: job.state,
            phase: job.phase,
            lines: job.lines.iter().skip(skip).cloned().collect(),
            next: job.sent,
            warnings: job.warnings.clone(),
            report: job.report.clone(),
            error: job.error.clone(),
        }
    }

    /// The job of the folder `dir` that runs, or else its latest one. Jobs of other folders (the
    /// hub is shared by pages and MCP) are not shown.
    pub fn current_for(&self, dir: &Path) -> Option<JobStatus> {
        let key = folder_key(dir);
        let table = self.table();
        let mine = || {
            table
                .jobs
                .iter()
                .filter(|(_, job)| folder_key(&job.dir) == key)
        };
        let (id, job) = mine()
            .find(|(_, job)| job.state == JobState::Running)
            .or_else(|| {
                mine().max_by_key(|(id, _)| id.trim_start_matches("job-").parse::<u64>().ok())
            })?;
        Some(Self::snapshot(id, job, 0))
    }

    /// Asks the running job `id` to stop; false when it does not run.
    pub fn cancel(&self, id: &str) -> bool {
        let table = self.table();
        match table.jobs.get(id) {
            Some(job) if job.state == JobState::Running => {
                job.control.cancel();
                true
            }
            _ => false,
        }
    }

    /// Whether a job runs for `dir`.
    pub fn running_for(&self, dir: &Path) -> bool {
        let key = folder_key(dir);
        self.table()
            .jobs
            .values()
            .any(|job| job.state == JobState::Running && folder_key(&job.dir) == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(summary: &str) -> RunReport {
        RunReport {
            summary: summary.to_string(),
            warnings: Vec::new(),
            files: Vec::new(),
            kept_map: false,
        }
    }

    fn ok(summary: &'static str) -> Runner {
        Box::new(move |_, _| Ok(report(summary)))
    }

    #[test]
    fn only_one_pipeline_runs_at_a_time() {
        // Jobs share the download cache, so a second one waits even for another project.
        let hub = JobHub::default();
        let first = hub.register(Path::new("/projects/school")).unwrap();
        assert_eq!(first, "job-1");
        for dir in ["/projects/school", "/projects/other"] {
            let refused = hub.register(Path::new(dir)).unwrap_err();
            assert!(refused.contains("job-1"), "{refused}");
        }
        hub.finish(&first, Err("offline".into()));
        assert_eq!(hub.register(Path::new("/projects/other")).unwrap(), "job-2");
    }

    #[test]
    fn panicking_job_is_reported_as_failed() {
        let hub = JobHub::default();
        let id = hub.register(Path::new("/p")).unwrap();
        hub.run_registered(&id, Box::new(|_, _| panic!("bad tile data")));
        let status = hub.status(&id, 0).unwrap().to_json();
        assert_eq!(status["state"], "failed");
        assert!(
            status["error"].as_str().unwrap().contains("bad tile data"),
            "{status}"
        );
    }

    #[test]
    fn job_progress_and_outcome_are_recorded() {
        let hub = JobHub::default();
        let id = hub.register(Path::new("/p")).unwrap();
        hub.run_registered(
            &id,
            Box::new(|_, progress| {
                progress(Event::Line("warning: no nDSM for tile 33421-5821"));
                Ok(report("Wrote the map"))
            }),
        );
        let status = hub.status(&id, 0).unwrap().to_json();
        assert_eq!(status["state"], "finished");
        assert_eq!(status["warnings"].as_array().unwrap().len(), 1);
        assert_eq!(status["result"], "Wrote the map");
    }

    #[test]
    fn job_keeps_the_last_lines_and_the_warnings() {
        let hub = JobHub::default();
        let id = hub.register(Path::new("/p")).unwrap();
        hub.progress(&id, Event::Line("warning: no nDSM for tile 33421-5821"));
        for i in 0..260 {
            hub.progress(&id, Event::Line(&format!("line {i}")));
        }
        let status = hub.status(&id, 0).unwrap();
        assert_eq!(status.state, JobState::Running);
        assert_eq!(status.lines.len(), KEPT_LINES);
        assert_eq!(status.lines[0], "line 60");
        assert_eq!(status.warnings.len(), 1);
        // The MCP status shows the last 50.
        let json = status.to_json();
        assert_eq!(json["lines"].as_array().unwrap().len(), MCP_LINES);
        assert_eq!(json["lines"][0], "line 210");
        assert_eq!(json["state"], "running");
        hub.finish(&id, Ok(report("Wrote the map")));
        let status = hub.status(&id, 0).unwrap().to_json();
        assert_eq!(status["state"], "finished");
        assert_eq!(status["result"], "Wrote the map");
    }

    #[test]
    fn status_since_returns_only_new_lines() {
        let hub = JobHub::default();
        let id = hub.register(Path::new("/p")).unwrap();
        for i in 0..3 {
            hub.progress(&id, Event::Line(&format!("line {i}")));
        }
        let first = hub.status(&id, 0).unwrap();
        assert_eq!(first.next, 3);
        hub.progress(&id, Event::Line("line 3"));
        let second = hub.status(&id, first.next).unwrap();
        assert_eq!(second.lines, ["line 3"]);
        assert_eq!(second.next, 4);
        // A number from before the kept window starts at the oldest kept line.
        for i in 4..300 {
            hub.progress(&id, Event::Line(&format!("line {i}")));
        }
        let late = hub.status(&id, 0).unwrap();
        assert_eq!(late.lines.len(), KEPT_LINES);
        assert!(hub.status(&id, 300).unwrap().lines.is_empty());
    }

    #[test]
    fn cancel_sets_the_flag_and_state_becomes_cancelled() {
        let hub = JobHub::default();
        let id = hub.register(Path::new("/p")).unwrap();
        assert!(!hub.cancel("job-99"));
        assert!(hub.cancel(&id));
        hub.run_registered(
            &id,
            Box::new(|control, _| {
                assert!(control.is_cancelled());
                Err("cancelled".into())
            }),
        );
        assert_eq!(hub.status(&id, 0).unwrap().state, JobState::Cancelled);
        // A finished job cannot be cancelled again.
        assert!(!hub.cancel(&id));
    }

    #[test]
    fn line_longer_than_500_characters_is_cut() {
        let hub = JobHub::default();
        let id = hub.register(Path::new("/p")).unwrap();
        hub.progress(&id, Event::Line(&"ä".repeat(900)));
        let status = hub.status(&id, 0).unwrap();
        assert_eq!(status.lines[0].chars().count(), LINE_CHARS);
    }

    #[test]
    fn phase_is_recorded_and_shown() {
        let hub = JobHub::default();
        let id = hub.register(Path::new("/p")).unwrap();
        hub.progress(
            &id,
            Event::Phase {
                step: Step::Derive,
                index: 2,
                total: 3,
            },
        );
        let json = hub.status(&id, 0).unwrap().to_json();
        assert_eq!(json["phase"]["step"], "step-derive");
        assert_eq!(json["phase"]["index"], 2);
        assert_eq!(json["phase"]["total"], 3);
    }

    #[test]
    fn running_for_compares_folders_not_spellings() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("a")).unwrap();
        let hub = JobHub::default();
        hub.register(&root.path().join("a")).unwrap();
        assert!(hub.running_for(&root.path().join("a/../a")));
        assert!(!hub.running_for(&root.path().join("b")));
    }

    #[test]
    fn current_prefers_the_running_job() {
        let hub = JobHub::default();
        assert!(hub.current_for(Path::new("/p")).is_none());
        let first = hub.register(Path::new("/p")).unwrap();
        hub.finish(&first, Err("offline".into()));
        assert_eq!(hub.current_for(Path::new("/p")).unwrap().job_id, "job-1");
        hub.register(Path::new("/p")).unwrap();
        assert_eq!(hub.current_for(Path::new("/p")).unwrap().job_id, "job-2");
    }

    #[test]
    fn current_for_ignores_the_jobs_of_other_folders() {
        let hub = JobHub::default();
        let mine = hub.register(Path::new("/mine")).unwrap();
        hub.finish(&mine, Err("offline".into()));
        hub.register(Path::new("/other")).unwrap();
        // The latest and the running job belong to another folder.
        assert_eq!(hub.current_for(Path::new("/mine")).unwrap().job_id, "job-1");
        assert!(hub.current_for(Path::new("/third")).is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn start_runs_the_job_on_a_blocking_thread() {
        let hub = Arc::new(JobHub::default());
        let id = hub.start(Path::new("/p"), ok("done")).unwrap();
        for _ in 0..200 {
            if hub.status(&id, 0).unwrap().state != JobState::Running {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        let status = hub.status(&id, 0).unwrap();
        assert_eq!(status.state, JobState::Finished);
        assert_eq!(status.report.unwrap().summary, "done");
    }
}
