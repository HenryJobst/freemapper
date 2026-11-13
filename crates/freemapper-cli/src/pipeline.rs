//! The pipeline behind `freemapper run`, the MCP tool `start_pipeline` and the run button of the
//! selection page: fetch, derive, write the map, then the optional OCAD and PDF outputs and the
//! permit notes. One function, so the three callers cannot drift apart (ADR 0090).

use crate::text::{Lang, TEXTS, n, text, v};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use freemapper::fetch::{Cache, FetchMode, Fetcher};
use freemapper::project::{
    MapOutcome, OcadOutcome, PdfSources, ProjectError, RunOutcome, RunStage,
    export_candidates_pdf_from, export_ocad_from, export_pdf_from, map_path, map_preview_path,
    pdf_path, run_project_steps_cancellable, starter_documents,
};
use serde::Deserialize;

/// What a run does besides fetch, derive and the OOM map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    /// Also write the permit notes after the map (ADR 0117). A failure there is a warning and
    /// never fails the run.
    pub permits: bool,
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
    pub permits: Option<bool>,
}

/// Checks a request and fills in the defaults of the command line.
pub fn parse(request: RunRequest, lang: &Lang) -> Result<RunOptions, String> {
    let pdf_sources = match request.pdf_sources.as_deref() {
        None | Some("auto") => PdfSources::Auto,
        Some("full") => PdfSources::Full,
        Some("credit-only") => PdfSources::CreditOnly,
        Some(other) => {
            return Err(TEXTS.text(
                lang,
                "cli-pdf-sources-invalid",
                &[("value", v(format!("{other:?}")))],
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
        permits: request.permits.unwrap_or(false),
        fail_fast: false,
    })
}

/// The spelling of `sources` in requests, flags and the project answer.
pub fn pdf_sources_name(sources: PdfSources) -> &'static str {
    match sources {
        PdfSources::Auto => "auto",
        PdfSources::Full => "full",
        PdfSources::CreditOnly => "credit-only",
    }
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
    Permits,
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
            Step::Permits => "step-permits",
        }
    }
}

/// What a run reports while it works.
#[derive(Debug, Clone, Copy)]
pub enum Event<'a> {
    /// `step` starts; `index` counts from 1 among `total` selected steps.
    Phase { step: Step, index: u8, total: u8 },
    /// A progress line of the engine, in the language of the run.
    Line(&'a str),
    /// A warning of the engine, in the language of the run. The run report lists it again.
    Warning(&'a str),
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
    /// `permits.md`.
    Permits(&'a Path),
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
#[allow(dead_code)] // `warnings`, `files` and `kept_map` are read by the page routes of stage 4.
pub struct RunReport {
    /// The outcome in words for an agent (English).
    pub summary: String,
    /// The engine lines that start with `warning:`.
    pub warnings: Vec<String>,
    pub files: Vec<ResultFile>,
    /// An existing map was left alone.
    pub kept_map: bool,
    /// The text of `permits.md`, for a job of the MCP tool `permits_for_area` alone.
    pub permits_md: Option<String>,
}

/// Lets a caller stop a run from another thread. The run looks at the flag before each step,
/// at each part of the fetch and the derive, before each request and before each output.
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

/// A run that stopped because it was cancelled. Everything written before is whole (ADR 0059,
/// 0060), so the next run continues from there; `kept` lists what is in the folder now.
#[derive(Debug)]
pub struct Cancelled {
    pub kept: Vec<ResultFile>,
}

impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use crate::errors::Localize;
        f.write_str(&self.localize(&crate::text::lang()))
    }
}

impl std::error::Error for Cancelled {}

/// The map, its preview, the templates, the OCAD files and the PDFs that exist in `dir` under
/// their fixed names.
fn kept_files(dir: &Path) -> Vec<ResultFile> {
    let mut paths: Vec<(&'static str, PathBuf)> =
        vec![("map", map_path(dir)), ("preview", map_preview_path(dir))];
    for (kind, folder) in [("template", "templates"), ("ocad", "ocad")] {
        let mut found: Vec<PathBuf> = std::fs::read_dir(dir.join(folder))
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| !p.to_string_lossy().contains(".part"))
            .collect();
        found.sort();
        paths.extend(found.into_iter().map(|p| (kind, p)));
    }
    paths.push(("pdf", pdf_path(dir, false)));
    paths.push(("pdf-candidates", pdf_path(dir, true)));
    paths
        .into_iter()
        .filter_map(|(kind, path)| file_entry(dir, kind, &path))
        .collect()
}

/// `path` as a result file, when it is a file.
fn file_entry(dir: &Path, kind: &'static str, path: &Path) -> Option<ResultFile> {
    let bytes = std::fs::metadata(path).ok().filter(|m| m.is_file())?.len();
    let relative = path.strip_prefix(dir).unwrap_or(path);
    Some(ResultFile {
        kind,
        relative_path: relative.to_string_lossy().replace('\\', "/"),
        bytes,
    })
}

/// An error with its causes, e.g. "fetch failed: tile 33421-5821 is not in the cache".
pub fn describe(error: &(dyn std::error::Error + 'static)) -> String {
    crate::errors::message(error, &crate::text::lang())
}

/// A progress line or a warning of the engine in the language of the run, its errors included.
pub fn line(note: &freemapper::note::Note) -> String {
    note.text(&crate::text::lang(), &crate::errors::message)
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
    if options.permits {
        steps.push(Step::Permits);
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
    control: &RunControl,
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
    let warn = |text: String| {
        (progress.borrow_mut())(Event::Warning(&text));
        warnings.borrow_mut().push(text);
    };
    let on_note = |note: freemapper::note::Note| {
        let text = line(&note);
        if note.is_warning() {
            warn(text);
        } else {
            (progress.borrow_mut())(Event::Line(&text));
        }
    };
    let outcome = run_project_steps_cancellable(
        dir,
        fetcher,
        cache,
        mode,
        options.force,
        &mut |note| on_note(note),
        &mut |stage| {
            let step = match stage {
                RunStage::Fetch => Step::Fetch,
                RunStage::Derive => Step::Derive,
                RunStage::Map => Step::Map,
            };
            (progress.borrow_mut())(phase(step));
        },
        &|| control.is_cancelled(),
    )
    .map_err(|e| match e {
        ProjectError::Cancelled => anyhow::Error::new(Cancelled {
            kept: kept_files(dir),
        }),
        e => e.into(),
    })?;
    (progress.borrow_mut())(Event::Written(Written::Map(&outcome)));
    let mut summary = map_summary(&outcome);
    let (mut ocad, mut pdf, mut pdf_candidates) = (None, None, None);
    // The map is written; a stop from here on keeps it and what the outputs finished.
    let stop = |ocad: &Option<Result<OcadOutcome, String>>,
                pdf: &Option<Result<PathBuf, String>>,
                pdf_candidates: &Option<Result<PathBuf, String>>| {
        control.is_cancelled().then(|| {
            anyhow::Error::new(Cancelled {
                kept: result_files(dir, &outcome, ocad, pdf, pdf_candidates, None),
            })
        })
    };
    if options.ocad || options.pdf || options.pdf_candidates {
        if let Some(cancelled) = stop(&ocad, &pdf, &pdf_candidates) {
            return Err(cancelled);
        }
        // Built once for the outputs after the run (ADR 0091); a failure is reported by each
        // output that needed them.
        let documents = settle(options.fail_fast, starter_documents(dir))?;
        let fail_fast = options.fail_fast;
        // The OOM project is written already: an output failure is reported, not fatal.
        if options.ocad {
            if let Some(cancelled) = stop(&ocad, &pdf, &pdf_candidates) {
                return Err(cancelled);
            }
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
                Err(e) => summary.push_str(&failed("cli-summary-ocad-failed", e)),
            }
            ocad = Some(result);
        }
        if (options.pdf || options.pdf_candidates)
            && options.pdf_sources.leaves_out_required_credits()
        {
            summary.push('\n');
            summary.push_str(&text("cli-summary-pdf-without-credits", &[]));
        }
        if options.pdf {
            if let Some(cancelled) = stop(&ocad, &pdf, &pdf_candidates) {
                return Err(cancelled);
            }
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
                    summary.push('\n');
                    summary.push_str(&text(
                        "cli-summary-wrote-pdf",
                        &[("path", v(path.display()))],
                    ));
                }
                Err(e) => summary.push_str(&failed("cli-summary-pdf-failed", e)),
            }
            pdf = Some(result);
        }
        if options.pdf_candidates {
            if let Some(cancelled) = stop(&ocad, &pdf, &pdf_candidates) {
                return Err(cancelled);
            }
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
                    summary.push('\n');
                    summary.push_str(&text(
                        "cli-summary-wrote-pdf-candidates",
                        &[("path", v(path.display()))],
                    ));
                }
                Err(e) => summary.push_str(&failed("cli-summary-pdf-candidates-failed", e)),
            }
            pdf_candidates = Some(result);
        }
    }
    // The permit notes come last: they ask other services, and the map is whole without them. A
    // failure is a warning, also on the command line (ruling of ADR 0117).
    let mut permits = None;
    if options.permits {
        if let Some(cancelled) = stop(&ocad, &pdf, &pdf_candidates) {
            return Err(cancelled);
        }
        (progress.borrow_mut())(phase(Step::Permits));
        match crate::permits::write_permits(
            dir,
            options.offline,
            cache,
            fetcher,
            &crate::text::lang(),
            &mut |note| on_note(note),
        ) {
            Ok(written) => {
                for warning in &written.warnings {
                    warn(warning.clone());
                }
                (progress.borrow_mut())(Event::Written(Written::Permits(&written.markdown)));
                summary.push('\n');
                summary.push_str(&text(
                    "cli-summary-wrote-permits",
                    &[("path", v(written.markdown.display()))],
                ));
                permits = Some(written);
            }
            Err(e) => {
                let warning = crate::text::warning(&text(
                    "cli-permits-failed",
                    &[("error", v(describe(e.as_ref())))],
                ));
                summary.push('\n');
                summary.push_str(&warning);
                warn(warning);
            }
        }
    }
    let kept_map = matches!(outcome.map, MapOutcome::Kept { .. });
    let files = result_files(
        dir,
        &outcome,
        &ocad,
        &pdf,
        &pdf_candidates,
        permits.as_ref(),
    );
    Ok(RunReport {
        summary,
        warnings: warnings.into_inner(),
        files,
        kept_map,
        permits_md: None,
    })
}

/// The permit notes alone, as the MCP tool `permits_for_area` runs them as a job: one step, the
/// files and the text of `permits.md` in the report. The caller holds the folder lock.
pub fn permits_job(
    dir: &Path,
    offline: bool,
    cache: &Cache,
    fetcher: &dyn Fetcher,
    control: &RunControl,
    progress: &mut dyn FnMut(Event),
) -> Result<RunReport, anyhow::Error> {
    if control.is_cancelled() {
        return Err(anyhow::Error::new(Cancelled { kept: Vec::new() }));
    }
    progress(Event::Phase {
        step: Step::Permits,
        index: 1,
        total: 1,
    });
    let mut warnings = Vec::new();
    let written = {
        let mut on_note = |note: freemapper::note::Note| {
            let text = line(&note);
            if note.is_warning() {
                progress(Event::Warning(&text));
                warnings.push(text);
            } else {
                progress(Event::Line(&text));
            }
        };
        crate::permits::write_permits(
            dir,
            offline,
            cache,
            fetcher,
            &crate::text::lang(),
            &mut on_note,
        )?
    };
    for warning in &written.warnings {
        progress(Event::Warning(warning));
        warnings.push(warning.clone());
    }
    progress(Event::Written(Written::Permits(&written.markdown)));
    let markdown = std::fs::read_to_string(&written.markdown)?;
    let files = [&written.markdown, &written.json]
        .into_iter()
        .filter_map(|path| file_entry(dir, "permits", path))
        .collect();
    Ok(RunReport {
        summary: text(
            "cli-summary-wrote-permits",
            &[("path", v(written.markdown.display()))],
        ),
        warnings,
        files,
        kept_map: false,
        permits_md: Some(markdown),
    })
}

/// The files of a run that exist now, under their fixed names.
fn result_files(
    dir: &Path,
    outcome: &RunOutcome,
    ocad: &Option<Result<OcadOutcome, String>>,
    pdf: &Option<Result<PathBuf, String>>,
    pdf_candidates: &Option<Result<PathBuf, String>>,
    permits: Option<&crate::permits::PermitsWritten>,
) -> Vec<ResultFile> {
    let mut candidates: Vec<(&'static str, PathBuf)> = vec![("map", map_path(dir))];
    // Only a preview written with the map: one left next to a kept map belongs to an older run.
    if let Some(preview) = &outcome.preview {
        candidates.push(("preview", preview.clone()));
    }
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
    if let Some(written) = permits {
        candidates.push(("permits", written.markdown.clone()));
        candidates.push(("permits", written.json.clone()));
    }
    candidates
        .into_iter()
        .filter_map(|(kind, path)| file_entry(dir, kind, &path))
        .collect()
}

/// What the map step did, in words for the user.
fn map_summary(outcome: &RunOutcome) -> String {
    let mut lines = Vec::new();
    match &outcome.map {
        MapOutcome::Written(map) => {
            lines.push(text("cli-summary-wrote-map", &[("path", v(map.display()))]));
            if let Some(preview) = &outcome.preview {
                lines.push(text("cli-wrote", &[("path", v(preview.display()))]));
            }
        }
        MapOutcome::Kept {
            map,
            references_images,
            missing_buildings,
            missing_canopies,
        } => {
            lines.push(text("cli-summary-kept-map", &[("path", v(map.display()))]));
            if !references_images {
                lines.push(text("cli-summary-map-lacks-templates", &[]));
            }
            if *missing_buildings {
                lines.push(text("cli-summary-map-lacks-buildings", &[]));
            }
            if *missing_canopies {
                lines.push(text("cli-summary-map-lacks-canopies", &[]));
            }
        }
    }
    for template in &outcome.templates {
        lines.push(text("cli-wrote", &[("path", v(template.display()))]));
    }
    lines.join("\n")
}

/// The summary line of a failed export step.
fn failed(id: &str, error: &str) -> String {
    format!("\n{}", text(id, &[("error", v(error))]))
}

/// What the OCAD export did, in words for the user.
fn ocad_summary(outcome: &OcadOutcome) -> String {
    let map = ("path", v(outcome.map.display()));
    let mut lines = vec![if outcome.kept {
        text("cli-summary-kept-ocad", &[map])
    } else {
        text(
            "cli-summary-wrote-ocad",
            &[map, ("count", n(outcome.objects))],
        )
    }];
    lines.extend(outcome.warnings.iter().map(line));
    if outcome.skipped > 0 || outcome.dropped_parts > 0 {
        lines.push(text(
            "cli-summary-ocad-left-out",
            &[
                ("skipped", n(outcome.skipped)),
                ("dropped", n(outcome.dropped_parts)),
            ],
        ));
    }
    lines.push(text(
        "cli-wrote",
        &[("path", v(outcome.candidates.display()))],
    ));
    lines.push(text(
        "cli-summary-ocad-symbols",
        &[("kind", v(outcome.map_kind.name()))],
    ));
    lines.iter().map(|line| format!("\n{line}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use freemapper::project::MapKind;

    fn all_off() -> RunOptions {
        parse(RunRequest::default(), &Lang::english()).unwrap()
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
        let request = || RunRequest {
            pdf_sources: Some("x".into()),
            ..RunRequest::default()
        };
        assert_eq!(
            parse(request(), &Lang::english()).unwrap_err(),
            "pdf_sources must be auto, full or credit-only, not \"x\""
        );
        let german = crate::text::TEXTS.parse_lang_arg("de").unwrap();
        assert_eq!(
            parse(request(), &german).unwrap_err(),
            "pdf_sources muss auto, full oder credit-only sein, nicht \"x\""
        );
        for (name, expected) in [
            ("full", PdfSources::Full),
            ("credit-only", PdfSources::CreditOnly),
        ] {
            let request = RunRequest {
                pdf_sources: Some(name.into()),
                ..RunRequest::default()
            };
            assert_eq!(
                parse(request, &Lang::english()).unwrap().pdf_sources,
                expected
            );
        }
    }

    #[test]
    fn parse_reads_permits() {
        assert!(!all_off().permits);
        let request = RunRequest {
            permits: Some(true),
            ..RunRequest::default()
        };
        assert!(parse(request, &Lang::english()).unwrap().permits);
        assert!(serde_json::from_str::<RunRequest>(r#"{"permits":true}"#).is_ok());
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
        let mut outcome = RunOutcome {
            templates: vec![PathBuf::from("/p/orthophoto.png")],
            map: MapOutcome::Written(PathBuf::from("/p/p.omap")),
            preview: None,
        };
        assert_eq!(
            map_summary(&outcome),
            "Wrote the map /p/p.omap\nWrote /p/orthophoto.png"
        );
        outcome.preview = Some(PathBuf::from("/p/p-preview.webp"));
        assert_eq!(
            map_summary(&outcome),
            "Wrote the map /p/p.omap\nWrote /p/p-preview.webp\nWrote /p/orthophoto.png"
        );
        let ocad = OcadOutcome {
            map: PathBuf::from("/p/ocad/p.ocd"),
            kept: false,
            candidates: PathBuf::from("/p/ocad/candidates.omap"),
            objects: 7,
            skipped: 1,
            dropped_parts: 2,
            warnings: vec![
                freemapper::note::Note::new("warning-ocad-no-background")
                    .with("error", "no hillshade"),
            ],
            map_kind: MapKind::Sprint,
        };
        assert_eq!(
            ocad_summary(&ocad),
            "\nWrote the OCAD map /p/ocad/p.ocd (7 objects)\nwarning: no hillshade; the OCAD map gets no background from it\n\
             Left out of the OCAD map: 1 objects and 2 degenerate parts\n\
             Wrote /p/ocad/candidates.omap\nIn OCAD, choose Symbol > Load Colors and Symbols \
             From… and pick the official symbol set ISSprOM 2019; it replaces the placeholder \
             symbols by number."
        );
    }

    #[test]
    fn ocad_summary_names_the_symbol_set_of_the_map_kind() {
        let summary = |map_kind| {
            ocad_summary(&OcadOutcome {
                map: PathBuf::from("/p/ocad/p.ocd"),
                kept: false,
                candidates: PathBuf::from("/p/ocad/p-candidates.ocd"),
                objects: 1,
                skipped: 0,
                dropped_parts: 0,
                warnings: Vec::new(),
                map_kind,
            })
        };
        let forest = summary(MapKind::Forest);
        assert!(
            forest.contains("the official symbol set ISOM 2017-2;"),
            "{forest}"
        );
        assert!(!forest.contains("ISSprOM"), "{forest}");
        let sprint = summary(MapKind::Sprint);
        assert!(
            sprint.contains("the official symbol set ISSprOM 2019;"),
            "{sprint}"
        );
        let school = summary(MapKind::Schoolyard);
        assert!(
            school.contains("the official symbol set, the IOF school set 2019;"),
            "{school}"
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
            preview: None,
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
        let mut outcome = RunOutcome {
            templates: vec![dir.path().join("missing.png")],
            map: MapOutcome::Written(map.clone()),
            preview: None,
        };
        let pdf = Some(Ok(pdf_path(dir.path(), false)));
        let files = result_files(dir.path(), &outcome, &None, &pdf, &None, None);
        // The PDF was not written, the template is absent: only the map is listed.
        assert_eq!(files.len(), 1, "{files:?}");
        assert_eq!(files[0].kind, "map");
        assert_eq!(files[0].bytes, 5);
        assert!(!files[0].relative_path.contains('/'), "{files:?}");

        // A preview in the folder that the run did not write (a kept map, a failed preview) is
        // not listed; the one written with the map is, right after it.
        let preview = map_preview_path(dir.path());
        std::fs::write(&preview, "123").unwrap();
        let files = result_files(dir.path(), &outcome, &None, &pdf, &None, None);
        assert_eq!(files.len(), 1, "{files:?}");
        outcome.preview = Some(preview.clone());
        let files = result_files(dir.path(), &outcome, &None, &pdf, &None, None);
        assert_eq!(files.len(), 2, "{files:?}");
        assert_eq!(files[0].kind, "map");
        assert_eq!(files[1].kind, "preview");
        assert_eq!(files[1].bytes, 3);
        assert_eq!(
            files[1].relative_path,
            preview.file_name().unwrap().to_string_lossy()
        );
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

    /// Counts the requests and fails them; `on_first` runs at the first one.
    struct Counting<'a> {
        calls: std::cell::Cell<usize>,
        on_first: &'a dyn Fn(),
    }

    impl Fetcher for Counting<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, freemapper::fetch::FetchError> {
            if self.calls.replace(self.calls.get() + 1) == 0 {
                (self.on_first)();
            }
            Err(freemapper::fetch::FetchError::Http {
                url: url.to_string(),
                message: "503 Service Unavailable".into(),
            })
        }
    }

    fn project(dir: &Path) {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/clara-zetkin-schule.geojson"
        ))
        .unwrap();
        let area = freemapper::area::MapArea::from_geojson(&text).unwrap();
        let date = chrono::NaiveDate::from_ymd_opt(2026, 11, 11).unwrap();
        freemapper::project::init_project(
            dir,
            &area,
            freemapper::project::MapKind::Schoolyard,
            None,
            date,
            None,
        )
        .unwrap();
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    #[test]
    fn run_cancelled_before_the_start_asks_for_nothing() {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path());
        let before = names(dir.path());
        let cache = Cache::new(dir.path().join("cache"));
        let control = RunControl::default();
        control.cancel();
        let fetcher = Counting {
            calls: Default::default(),
            on_first: &|| {},
        };
        let error = run(
            dir.path(),
            &all_off(),
            &cache,
            &fetcher,
            &control,
            &mut |_| {},
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "cancelled");
        assert_eq!(fetcher.calls.get(), 0);
        assert_eq!(names(dir.path()), before);
    }

    #[test]
    fn run_cancelled_during_the_fetch_says_cancelled_not_the_download_error() {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path());
        let cache = Cache::new(dir.path().join("cache"));
        let control = RunControl::default();
        let fetcher = Counting {
            calls: Default::default(),
            on_first: &|| control.cancel(),
        };
        let error = run(
            dir.path(),
            &all_off(),
            &cache,
            &fetcher,
            &control,
            &mut |_| {},
        )
        .unwrap_err();
        assert!(error.to_string().starts_with("cancelled"), "{error}");
        assert_eq!(fetcher.calls.get(), 1, "no request after the cancel");
    }

    #[test]
    fn a_cancelled_run_names_what_it_kept() {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path());
        let map = map_path(dir.path());
        std::fs::write(&map, "edited in OOM").unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        let control = RunControl::default();
        control.cancel();
        let fetcher = Counting {
            calls: Default::default(),
            on_first: &|| {},
        };
        let error = run(
            dir.path(),
            &all_off(),
            &cache,
            &fetcher,
            &control,
            &mut |_| {},
        )
        .unwrap_err()
        .to_string();
        let name = map.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(error, format!("cancelled; kept: {name}"));
        assert_eq!(std::fs::read_to_string(&map).unwrap(), "edited in OOM");
        // A preview in the folder is named too.
        let preview = map_preview_path(dir.path());
        std::fs::write(&preview, "image").unwrap();
        let error = run(
            dir.path(),
            &all_off(),
            &cache,
            &fetcher,
            &control,
            &mut |_| {},
        )
        .unwrap_err()
        .to_string();
        let preview_name = preview.file_name().unwrap().to_string_lossy().into_owned();
        assert_eq!(error, format!("cancelled; kept: {name}, {preview_name}"));
    }

    /// Records every URL. Answers the currency lists of the LGB from the cache of the fixture,
    /// ALKIS and the national services with an empty collection, and refuses everything else.
    #[derive(Default)]
    struct Recording {
        cache: PathBuf,
        urls: RefCell<Vec<String>>,
    }

    impl Fetcher for Recording {
        fn get(&self, url: &str) -> Result<Vec<u8>, freemapper::fetch::FetchError> {
            self.urls.borrow_mut().push(url.to_string());
            let product = if url.ends_with("bb_dop_aktualitaet.csv") {
                Some("dop20rgb")
            } else if url.ends_with("bb_dgm_aktualitaet.csv") {
                Some("dgm1")
            } else {
                None
            };
            if url.starts_with("https://ogc-api.geobasis-bb.de/")
                || NATIONAL_HOSTS
                    .iter()
                    .any(|host| url.starts_with(&format!("https://{host}/")))
            {
                return Ok(br#"{"type":"FeatureCollection","features":[]}"#.to_vec());
            }
            match product {
                Some(product) => Ok(std::fs::read(
                    self.cache.join("lgb").join(product).join("currency.csv"),
                )
                .unwrap()),
                // Refused, not retried: the test does not wait for the backoff.
                None => Err(freemapper::fetch::FetchError::Refused {
                    url: url.to_string(),
                    status: 403,
                    detail: None,
                }),
            }
        }
    }

    fn copy_tree(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap().flatten() {
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&entry.path(), &target);
            } else {
                std::fs::copy(entry.path(), target).unwrap();
            }
        }
    }

    /// `UrlPolicy::NATIONAL_HOSTS`, the hosts of the BfN and the BKG.
    const NATIONAL_HOSTS: [&str; 2] = ["geodienste.bfn.de", "sgx.geodatenzentrum.de"];

    fn asks_a_national_host(urls: &[String]) -> bool {
        urls.iter()
            .any(|url| NATIONAL_HOSTS.iter().any(|host| url.contains(host)))
    }

    /// An online run asks the BfN and the BKG only for the permit notes: without `permits` no
    /// request goes to them (ADR 0117, opt-in). MCP `start_pipeline` and the page run through
    /// this function with `RunRequest::permits` false by default.
    #[test]
    fn an_online_run_without_permits_asks_no_national_host() {
        let dir = tempfile::tempdir().unwrap();
        let fixture = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/offline-school"
        ));
        let (project, cache_dir) = (dir.path().join("school"), dir.path().join("cache"));
        copy_tree(&fixture.join("project"), &project);
        copy_tree(&fixture.join("cache"), &cache_dir);
        let cache = Cache::new(cache_dir.clone());
        let online = RunOptions {
            offline: false,
            ..all_off()
        };
        let fetcher = Recording {
            cache: cache_dir.clone(),
            ..Recording::default()
        };
        let report = run(
            &project,
            &online,
            &cache,
            &fetcher,
            &RunControl::default(),
            &mut |_| {},
        );
        assert!(report.is_ok(), "the map run gets to its end: {report:?}");
        assert!(
            !asks_a_national_host(&fetcher.urls.borrow()),
            "{:?}",
            fetcher.urls.borrow()
        );

        // The same run with the notes does ask them, so the run above did get that far.
        let with_permits = RunOptions {
            permits: true,
            force: true,
            ..online
        };
        let fetcher = Recording {
            cache: cache_dir.clone(),
            ..Recording::default()
        };
        run(
            &project,
            &with_permits,
            &cache,
            &fetcher,
            &RunControl::default(),
            &mut |_| {},
        )
        .unwrap();
        assert!(
            asks_a_national_host(&fetcher.urls.borrow()),
            "{:?}",
            fetcher.urls.borrow()
        );
        let parsed = parse(RunRequest::default(), &Lang::english()).unwrap();
        assert!(!parsed.permits, "MCP and the page default to no notes");
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
            Step::Permits,
        ]
        .map(Step::label_key)
        .into();
        assert_eq!(keys.len(), 7);
    }
}
