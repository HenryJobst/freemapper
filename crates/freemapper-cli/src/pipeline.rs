//! The pipeline behind `freemapper run`, the MCP tool `start_pipeline` and the run button of the
//! selection page: fetch, derive, write the map, then the optional OCAD and PDF outputs. One
//! function, so the three callers cannot drift apart (ADR 0090).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use freemapper::fetch::{Cache, FetchMode, Fetcher};
use freemapper::project::{
    MapOutcome, OcadOutcome, PdfSources, RunOutcome, RunStage, StarterDocuments,
    export_candidates_pdf_from, export_ocad_from, export_pdf_from, map_path, pdf_path,
    run_project_steps, starter_documents,
};
use serde::Deserialize;

/// What a run does besides fetch, derive and the OOM map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunOptions {
    /// Replace an existing map and keep the old one as `.omap.bak`.
    pub force: bool,
    /// Use the download cache only.
    pub offline: bool,
    pub ocad: bool,
    pub pdf: bool,
    pub pdf_candidates: bool,
    pub pdf_scale_bar: bool,
    pub pdf_sources: PdfSources,
    /// Stop with the first failing OCAD or PDF output (the command line). Without it the failure
    /// is written into the summary and the other outputs still run (MCP and the page).
    pub fail_fast: bool,
}

/// A run as an MCP client or the page asks for it; every field is optional.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    pub force: Option<bool>,
    pub offline: Option<bool>,
    pub ocad: Option<bool>,
    pub pdf: Option<bool>,
    pub pdf_candidates: Option<bool>,
    pub pdf_scale_bar: Option<bool>,
    pub pdf_sources: Option<String>,
}

/// Checks a request and fills in the defaults of the command line.
pub fn parse(request: RunRequest) -> Result<RunOptions, String> {
    let pdf_sources = match request.pdf_sources.as_deref() {
        None | Some("auto") => PdfSources::Auto,
        Some("full") => PdfSources::Full,
        Some("credit-only") => PdfSources::CreditOnly,
        Some(other) => {
            return Err(format!(
                "pdf_sources must be auto, full or credit-only, not {other:?}"
            ));
        }
    };
    Ok(RunOptions {
        force: request.force.unwrap_or(false),
        offline: request.offline.unwrap_or(false),
        ocad: request.ocad.unwrap_or(false),
        pdf: request.pdf.unwrap_or(false),
        pdf_candidates: request.pdf_candidates.unwrap_or(false),
        pdf_scale_bar: request.pdf_scale_bar.unwrap_or(false),
        pdf_sources,
        fail_fast: false,
    })
}

/// One step of a run, in the order they run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Fetch,
    Derive,
    Map,
    Ocad,
    Pdf,
    PdfCandidates,
}

impl Step {
    /// The id of the step name for the page texts.
    pub fn label_key(self) -> &'static str {
        match self {
            Step::Fetch => "step-fetch",
            Step::Derive => "step-derive",
            Step::Map => "step-map",
            Step::Ocad => "step-ocad",
            Step::Pdf => "step-pdf",
            Step::PdfCandidates => "step-pdf-candidates",
        }
    }
}

/// What a run reports while it works.
#[derive(Debug, Clone, Copy)]
pub enum Event<'a> {
    /// `step` starts; `index` counts from 1 among `total` selected steps.
    Phase { step: Step, index: u8, total: u8 },
    /// A progress line of the engine.
    Line(&'a str),
    /// A step's result, sent as soon as it exists, so the command line can print it before a
    /// later step fails.
    Written(Written<'a>),
}

/// What a finished step wrote.
#[derive(Debug, Clone, Copy)]
pub enum Written<'a> {
    /// Templates and map.
    Map(&'a RunOutcome),
    Ocad(&'a OcadOutcome),
    Pdf(&'a Path),
    PdfCandidates(&'a Path),
}

/// A file a run produced, relative to the project folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultFile {
    pub kind: &'static str,
    pub relative_path: String,
    pub bytes: u64,
}

/// What a run did.
#[derive(Debug, Clone)]
pub struct RunReport {
    /// The outcome in words for an agent (English).
    pub summary: String,
    /// The engine lines that start with `warning:`.
    pub warnings: Vec<String>,
    pub files: Vec<ResultFile>,
    /// An existing map was left alone.
    pub kept_map: bool,
}

/// Lets a caller stop a run from another thread. Cancelling takes effect from stage 3 on.
#[derive(Debug, Clone, Default)]
pub struct RunControl {
    cancelled: Arc<AtomicBool>,
}

impl RunControl {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// An error with its causes, e.g. "fetch failed: tile 33421-5821 is not in the cache".
pub fn describe(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        let line = cause.to_string();
        if !message.contains(&line) {
            message = format!("{message}: {line}");
        }
        source = cause.source();
    }
    message
}

/// The steps `options` selects, in running order.
fn selected_steps(options: &RunOptions) -> Vec<Step> {
    let mut steps = vec![Step::Fetch, Step::Derive, Step::Map];
    if options.ocad {
        steps.push(Step::Ocad);
    }
    if options.pdf {
        steps.push(Step::Pdf);
    }
    if options.pdf_candidates {
        steps.push(Step::PdfCandidates);
    }
    steps
}

/// A failing output stops the run (`fail_fast`) or becomes its text.
fn settle<T, E: std::error::Error + Send + Sync + 'static>(
    fail_fast: bool,
    result: Result<T, E>,
) -> Result<Result<T, String>, anyhow::Error> {
    match result {
        Ok(value) => Ok(Ok(value)),
        Err(e) if fail_fast => Err(e.into()),
        Err(e) => Ok(Err(describe(&e))),
    }
}

/// Fetches, derives and writes the map, then the outputs `options` asks for. An existing map
/// stays unless `options.force`.
pub fn run(
    dir: &Path,
    options: &RunOptions,
    cache: &Cache,
    fetcher: &dyn Fetcher,
    _control: &RunControl,
    progress: &mut dyn FnMut(Event),
) -> Result<RunReport, anyhow::Error> {
    let steps = selected_steps(options);
    let total = steps.len() as u8;
    let phase = |step: Step| Event::Phase {
        step,
        index: steps
            .iter()
            .position(|s| *s == step)
            .map_or(0, |i| i as u8 + 1),
        total,
    };
    let mode = if options.offline {
        FetchMode::Offline
    } else {
        FetchMode::Online
    };
    let progress = RefCell::new(progress);
    let warnings = RefCell::new(Vec::new());
    let outcome = run_project_steps(
        dir,
        fetcher,
        cache,
        mode,
        options.force,
        &mut |text| {
            if text.starts_with("warning:") {
                warnings.borrow_mut().push(text.to_string());
            }
            (progress.borrow_mut())(Event::Line(text));
        },
        &mut |stage| {
            let step = match stage {
                RunStage::Fetch => Step::Fetch,
                RunStage::Derive => Step::Derive,
                RunStage::Map => Step::Map,
            };
            (progress.borrow_mut())(phase(step));
        },
    )?;
    (progress.borrow_mut())(Event::Written(Written::Map(&outcome)));
    let mut summary = map_summary(&outcome);
    let (mut ocad, mut pdf, mut pdf_candidates) = (None, None, None);
    if options.ocad || options.pdf || options.pdf_candidates {
        // Built once for the outputs after the run (ADR 0091); a failure is reported by each
        // output that needed them.
        let documents = settle(options.fail_fast, starter_documents(dir))?;
        let fail_fast = options.fail_fast;
        // The OOM project is written already: an output failure is reported, not fatal.
        if options.ocad {
            (progress.borrow_mut())(phase(Step::Ocad));
            let result = match &documents {
                Ok(d) => settle(fail_fast, export_ocad_from(dir, d, options.force))?,
                Err(e) => Err(e.clone()),
            };
            match &result {
                Ok(outcome) => {
                    (progress.borrow_mut())(Event::Written(Written::Ocad(outcome)));
                    summary.push_str(&ocad_summary(outcome));
                }
                Err(e) => summary.push_str(&format!("\nOCAD export failed: {e}")),
            }
            ocad = Some(result);
        }
        if (options.pdf || options.pdf_candidates)
            && options.pdf_sources.leaves_out_required_credits()
        {
            summary.push_str(
                "\nPDF without source credits: ODbL and dl-de/by-2-0 ask you to name the \
                 sources when you pass the map on; see ATTRIBUTION.txt",
            );
        }
        if options.pdf {
            (progress.borrow_mut())(phase(Step::Pdf));
            let result = match &documents {
                Ok(d) => settle(
                    fail_fast,
                    export_pdf_from(dir, d, options.pdf_scale_bar, options.pdf_sources),
                )?,
                Err(e) => Err(e.clone()),
            };
            match &result {
                Ok(path) => {
                    (progress.borrow_mut())(Event::Written(Written::Pdf(path)));
                    summary.push_str(&format!(
                        "\nWrote the map {} (without candidates)",
                        path.display()
                    ));
                }
                Err(e) => summary.push_str(&format!("\nPDF export failed: {e}")),
            }
            pdf = Some(result);
        }
        if options.pdf_candidates {
            (progress.borrow_mut())(phase(Step::PdfCandidates));
            let result = match &documents {
                Ok(d) => settle(
                    fail_fast,
                    export_candidates_pdf_from(dir, d, options.pdf_scale_bar, options.pdf_sources),
                )?,
                Err(e) => Err(e.clone()),
            };
            match &result {
                Ok(path) => {
                    (progress.borrow_mut())(Event::Written(Written::PdfCandidates(path)));
                    summary.push_str(&format!(
                        "\nWrote the training map {} (every candidate drawn as a map object)",
                        path.display()
                    ));
                }
                Err(e) => summary.push_str(&format!("\nCandidates PDF export failed: {e}")),
            }
            pdf_candidates = Some(result);
        }
    }
    let kept_map = matches!(outcome.map, MapOutcome::Kept { .. });
    let files = result_files(dir, &outcome, &ocad, &pdf, &pdf_candidates);
    Ok(RunReport {
        summary,
        warnings: warnings.into_inner(),
        files,
        kept_map,
    })
}

/// The files of a run that exist now, under their fixed names.
fn result_files(
    dir: &Path,
    outcome: &RunOutcome,
    ocad: &Option<Result<OcadOutcome, String>>,
    pdf: &Option<Result<PathBuf, String>>,
    pdf_candidates: &Option<Result<PathBuf, String>>,
) -> Vec<ResultFile> {
    let mut candidates: Vec<(&'static str, PathBuf)> = vec![("map", map_path(dir))];
    for template in &outcome.templates {
        candidates.push(("template", template.clone()));
    }
    if let Some(Ok(ocad)) = ocad {
        candidates.push(("ocad", ocad.map.clone()));
        candidates.push(("ocad-candidates", ocad.candidates.clone()));
    }
    if matches!(pdf, Some(Ok(_))) {
        candidates.push(("pdf", pdf_path(dir, false)));
    }
    if matches!(pdf_candidates, Some(Ok(_))) {
        candidates.push(("pdf-candidates", pdf_path(dir, true)));
    }
    candidates
        .into_iter()
        .filter_map(|(kind, path)| {
            let bytes = std::fs::metadata(&path).ok().filter(|m| m.is_file())?.len();
            let relative = path.strip_prefix(dir).unwrap_or(&path);
            Some(ResultFile {
                kind,
                relative_path: relative.to_string_lossy().replace('\\', "/"),
                bytes,
            })
        })
        .collect()
}

/// What the map step did, in words for the user.
fn map_summary(outcome: &RunOutcome) -> String {
    let mut lines = Vec::new();
    match &outcome.map {
        MapOutcome::Written(map) => lines.push(format!("Wrote the map {}", map.display())),
        MapOutcome::Kept {
            map,
            references_images,
            missing_buildings,
            missing_canopies,
        } => {
            lines.push(format!(
                "Kept the existing map {} (it may contain the mapper's work)",
                map.display()
            ));
            if !references_images {
                lines.push(
                    "The map does not reference the image templates yet. Start the pipeline \
                     with force to replace it and keep the old one as .omap.bak."
                        .to_string(),
                );
            }
            if *missing_buildings {
                lines.push(
                    "The map has no ALKIS buildings yet. Start the pipeline with force to add \
                     them; it replaces the map and keeps the old one as .omap.bak."
                        .to_string(),
                );
            }
            if *missing_canopies {
                lines.push(
                    "The map lacks the ALKIS canopies and carports (522) that are now map \
                     objects. Start the pipeline with force to add them; it replaces the map and \
                     keeps the old one as .omap.bak."
                        .to_string(),
                );
            }
        }
    }
    for template in &outcome.templates {
        lines.push(format!("Wrote {}", template.display()));
    }
    lines.join("\n")
}

/// What the OCAD export did, in words for the user.
fn ocad_summary(outcome: &OcadOutcome) -> String {
    let mut text = if outcome.kept {
        format!(
            "\nKept the existing OCAD map {} (it may contain the mapper's work; force replaces it)",
            outcome.map.display()
        )
    } else {
        format!(
            "\nWrote the OCAD map {} ({} objects)",
            outcome.map.display(),
            outcome.objects
        )
    };
    for warning in &outcome.warnings {
        text.push_str(&format!("\n{warning}"));
    }
    if outcome.skipped > 0 || outcome.dropped_parts > 0 {
        text.push_str(&format!(
            "\nLeft out of the OCAD map: {} objects and {} degenerate parts",
            outcome.skipped, outcome.dropped_parts
        ));
    }
    text.push_str(&format!(
        "\nWrote {}\nIn OCAD, choose Symbol > Load Colors and Symbols From… and pick the \
         official ISSprOM 2019 or school symbol set; it replaces the placeholder symbols by number.",
        outcome.candidates.display()
    ));
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_off() -> RunOptions {
        parse(RunRequest::default()).unwrap()
    }

    #[test]
    fn parse_defaults_match_the_cli() {
        let options = all_off();
        assert!(!options.force && !options.offline && !options.ocad);
        assert!(!options.pdf && !options.pdf_candidates && !options.pdf_scale_bar);
        assert_eq!(options.pdf_sources, PdfSources::Auto);
        assert!(!options.fail_fast);
    }

    #[test]
    fn parse_refuses_unknown_pdf_sources() {
        let request = RunRequest {
            pdf_sources: Some("x".into()),
            ..RunRequest::default()
        };
        assert_eq!(
            parse(request).unwrap_err(),
            "pdf_sources must be auto, full or credit-only, not \"x\""
        );
        for (name, expected) in [
            ("full", PdfSources::Full),
            ("credit-only", PdfSources::CreditOnly),
        ] {
            let request = RunRequest {
                pdf_sources: Some(name.into()),
                ..RunRequest::default()
            };
            assert_eq!(parse(request).unwrap().pdf_sources, expected);
        }
    }

    #[test]
    fn parse_refuses_unknown_fields() {
        assert!(serde_json::from_str::<RunRequest>(r#"{"force":true,"dir":"/x"}"#).is_err());
        assert!(serde_json::from_str::<RunRequest>(r#"{"force":true}"#).is_ok());
    }

    #[test]
    fn total_counts_only_selected_steps() {
        assert_eq!(selected_steps(&all_off()).len(), 3);
        let options = RunOptions {
            pdf: true,
            ocad: true,
            ..all_off()
        };
        assert_eq!(
            selected_steps(&options),
            [Step::Fetch, Step::Derive, Step::Map, Step::Ocad, Step::Pdf]
        );
    }

    #[test]
    fn summary_text_is_unchanged() {
        let outcome = RunOutcome {
            templates: vec![PathBuf::from("/p/orthophoto.png")],
            map: MapOutcome::Written(PathBuf::from("/p/p.omap")),
        };
        assert_eq!(
            map_summary(&outcome),
            "Wrote the map /p/p.omap\nWrote /p/orthophoto.png"
        );
        let ocad = OcadOutcome {
            map: PathBuf::from("/p/ocad/p.ocd"),
            kept: false,
            candidates: PathBuf::from("/p/ocad/candidates.omap"),
            objects: 7,
            skipped: 1,
            dropped_parts: 2,
            warnings: vec!["warning: no hillshade".into()],
        };
        assert_eq!(
            ocad_summary(&ocad),
            "\nWrote the OCAD map /p/ocad/p.ocd (7 objects)\nwarning: no hillshade\n\
             Left out of the OCAD map: 1 objects and 2 degenerate parts\n\
             Wrote /p/ocad/candidates.omap\nIn OCAD, choose Symbol > Load Colors and Symbols \
             From… and pick the official ISSprOM 2019 or school symbol set; it replaces the \
             placeholder symbols by number."
        );
    }

    #[test]
    fn missing_canopies_message_names_no_state() {
        let outcome = RunOutcome {
            templates: Vec::new(),
            map: MapOutcome::Kept {
                map: PathBuf::from("/tmp/x/x.omap"),
                references_images: true,
                missing_buildings: false,
                missing_canopies: true,
            },
        };
        let text = map_summary(&outcome);
        assert!(
            text.contains("The map lacks the ALKIS canopies and carports (522)"),
            "{text}"
        );
        assert!(!text.contains("Berlin"), "{text}");
    }

    #[test]
    fn files_lists_only_fixed_names() {
        let dir = tempfile::tempdir().unwrap();
        let map = map_path(dir.path());
        std::fs::write(&map, "12345").unwrap();
        let outcome = RunOutcome {
            templates: vec![dir.path().join("missing.png")],
            map: MapOutcome::Written(map.clone()),
        };
        let pdf = Some(Ok(pdf_path(dir.path(), false)));
        let files = result_files(dir.path(), &outcome, &None, &pdf, &None);
        // The PDF was not written, the template is absent: only the map is listed.
        assert_eq!(files.len(), 1, "{files:?}");
        assert_eq!(files[0].kind, "map");
        assert_eq!(files[0].bytes, 5);
        assert!(!files[0].relative_path.contains('/'), "{files:?}");
    }

    #[test]
    fn describe_adds_causes_once() {
        #[derive(Debug)]
        struct Outer(std::io::Error);
        impl std::fmt::Display for Outer {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("fetch failed")
            }
        }
        impl std::error::Error for Outer {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.0)
            }
        }
        let error = Outer(std::io::Error::other("tile 33421-5821 is not in the cache"));
        assert_eq!(
            describe(&error),
            "fetch failed: tile 33421-5821 is not in the cache"
        );
    }

    #[test]
    fn control_flag_is_shared_between_clones() {
        let control = RunControl::default();
        let clone = control.clone();
        assert!(!clone.is_cancelled());
        control.cancel();
        assert!(clone.is_cancelled());
    }

    #[test]
    fn step_labels_are_distinct() {
        let keys: std::collections::HashSet<_> = [
            Step::Fetch,
            Step::Derive,
            Step::Map,
            Step::Ocad,
            Step::Pdf,
            Step::PdfCandidates,
        ]
        .map(Step::label_key)
        .into();
        assert_eq!(keys.len(), 6);
    }
}
