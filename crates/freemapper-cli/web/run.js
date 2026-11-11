// freemapper run panel: options, start, progress, cancel, result. Runs after app.js, which
// offers `window.freemapperPage`. Only textContent is used to show server text.
"use strict";

const RUN_TEXTS = {
  de: {
    runHeading: "Karte erzeugen",
    optOcad: "OCAD-Startprojekt schreiben",
    helpOcad: "Für alle, die in OCAD weiterarbeiten.",
    optPdf: "Druckfertige PDF",
    helpPdf: "Nur die Kartenobjekte, ohne Kandidaten.",
    optPdfCandidates: "Trainings-PDF mit allen Kandidaten",
    optScaleBar: "Maßstabsleiste in die PDFs",
    helpScaleBar: "Deutsche Karten haben meist keine.",
    optSources: "Quellenangabe in den PDFs",
    sourcesAuto: "automatisch",
    sourcesFull: "vollständig",
    sourcesCreditOnly: "nur freemapper",
    warnCreditOnly:
      "Die PDFs nennen dann nur freemapper. Die Lizenzen der Daten (ODbL, dl-de/by-2-0) verlangen die Quellenangabe, wenn du die Karte weitergibst; dann nennst du die Quellen selbst. ATTRIBUTION.txt listet sie alle.",
    needPdf: "Maßstabsleiste und Quellenangabe gelten nur, wenn eine PDF entsteht.",
    optOffline: "Nur Zwischenspeicher, kein Netz",
    optForce: "Vorhandene Karte ersetzen",
    helpForce: "Die alte Datei bleibt als .omap.bak.",
    runStart: "Karte erzeugen",
    runAgain: "Nochmal erzeugen",
    runCancel: "Abbrechen",
    runCancelling: "Wird abgebrochen …",
    warningsHeading: "Warnungen",
    whereFiles: "Die Dateien liegen im Projektordner:",
    openFolder: "Ordner öffnen",
    logLabel: "Protokoll des Laufs",
    forceTitle: "Vorhandene Karte ersetzen?",
    forceText:
      "Die vorhandene Karte wird ersetzt. Deine Änderungen darin gehen nicht verloren: Die alte Datei bleibt als .omap.bak im Projektordner. Fortfahren?",
    forceCancel: "Abbrechen",
    forceConfirm: "Ersetzen",
    "step-fetch": "Daten laden",
    "step-derive": "Vorlagen berechnen",
    "step-map": "Karte schreiben",
    "step-ocad": "OCAD-Projekt schreiben",
    "step-pdf": "PDF schreiben",
    "step-pdf-candidates": "Trainings-PDF schreiben",
    stepLine: (index, total, label) => `Schritt ${index} von ${total}: ${label}`,
    starting: "Der Lauf startet …",
    "kind-map": "Karte",
    "kind-template": "Vorlage",
    "kind-ocad": "OCAD-Karte",
    "kind-ocad-candidates": "OCAD-Kandidaten",
    "kind-pdf": "PDF",
    "kind-pdf-candidates": "Trainings-PDF",
    fileLine: (path, kind, size) => `${path} (${kind}, ${size})`,
    resultDone: "Fertig. Die Dateien sind geschrieben.",
    resultFailed: (error) => `Der Lauf ist fehlgeschlagen: ${error}`,
    resultFailedKept:
      "Was schon geschrieben ist, bleibt im Projektordner; ein neuer Lauf macht dort weiter.",
    resultCancelled:
      "Abgebrochen. Was schon geladen ist, bleibt im Zwischenspeicher; ein neuer Lauf macht dort weiter.",
    resultTitleDone: "Karte erzeugt",
    resultTitleFailed: "Lauf fehlgeschlagen",
    resultTitleCancelled: "Lauf abgebrochen",
    keptMap:
      "Die vorhandene Karte blieb unverändert. Mit „Vorhandene Karte ersetzen“ entsteht sie neu, die alte bleibt als .omap.bak.",
    noFiles: "Es sind keine Dateien entstanden.",
    openedFolder: "Der Ordner ist geöffnet.",
    openFailed: (message, dir) => `${message} Ordner: ${dir}`,
    whyNoArea: "Wähle zuerst ein Gebiet.",
    whyChecking: "Die Vorschau wird berechnet …",
    whyTooLarge: "Das Gebiet ist zu groß.",
    whyOutside: "Das Gebiet liegt außerhalb der unterstützten Bundesländer.",
    whyDisk: "Es ist zu wenig Speicherplatz frei (siehe oben).",
    whyRunning: "Ein Lauf ist im Gang.",
    errScaleBarNoPdf: "Die Maßstabsleiste braucht eine PDF.",
    errSourcesNoPdf: "Die Quellenangabe braucht eine PDF.",
    saveFailed: "Das Gebiet ist nicht gespeichert; es gibt noch keinen Lauf.",
    pollLost: "Keine Antwort vom Server. freemapper versucht es weiter.",
    runUnknown: "Der Lauf ist dem Server nicht mehr bekannt.",
    failedPrefix: "Fehler: ",
  },
  en: {
    runHeading: "Create the map",
    optOcad: "Write an OCAD starter project",
    helpOcad: "For everyone who carries on in OCAD.",
    optPdf: "Printable PDF",
    helpPdf: "Map objects only, no candidates.",
    optPdfCandidates: "Training PDF with every candidate",
    optScaleBar: "Scale bar on the PDFs",
    helpScaleBar: "German maps usually have none.",
    optSources: "Source lines on the PDFs",
    sourcesAuto: "automatic",
    sourcesFull: "full",
    sourcesCreditOnly: "freemapper only",
    warnCreditOnly:
      "The PDFs then name freemapper only. The data licences (ODbL, dl-de/by-2-0) require attribution when you pass the map on; then you name the sources yourself. ATTRIBUTION.txt lists them all.",
    needPdf: "The scale bar and the source lines only apply when a PDF is made.",
    optOffline: "Cache only, no network",
    optForce: "Replace the existing map",
    helpForce: "The old file stays as .omap.bak.",
    runStart: "Create map",
    runAgain: "Create again",
    runCancel: "Cancel",
    runCancelling: "Cancelling …",
    warningsHeading: "Warnings",
    whereFiles: "The files are in the project folder:",
    openFolder: "Open folder",
    logLabel: "Log of the run",
    forceTitle: "Replace the existing map?",
    forceText:
      "The existing map is replaced. Your edits in it are not lost: the old file stays in the project folder as .omap.bak. Continue?",
    forceCancel: "Cancel",
    forceConfirm: "Replace",
    "step-fetch": "Fetching data",
    "step-derive": "Computing templates",
    "step-map": "Writing the map",
    "step-ocad": "Writing the OCAD project",
    "step-pdf": "Writing the PDF",
    "step-pdf-candidates": "Writing the training PDF",
    stepLine: (index, total, label) => `Step ${index} of ${total}: ${label}`,
    starting: "The run is starting …",
    "kind-map": "map",
    "kind-template": "template",
    "kind-ocad": "OCAD map",
    "kind-ocad-candidates": "OCAD candidates",
    "kind-pdf": "PDF",
    "kind-pdf-candidates": "training PDF",
    fileLine: (path, kind, size) => `${path} (${kind}, ${size})`,
    resultDone: "Done. The files are written.",
    resultFailed: (error) => `The run failed: ${error}`,
    resultFailedKept:
      "What is written stays in the project folder; a new run carries on from there.",
    resultCancelled:
      "Cancelled. What is downloaded already stays in the cache; a new run carries on from there.",
    resultTitleDone: "Map created",
    resultTitleFailed: "Run failed",
    resultTitleCancelled: "Run cancelled",
    keptMap:
      "The existing map was left as it is. \"Replace the existing map\" makes it anew and keeps the old one as .omap.bak.",
    noFiles: "No files were made.",
    openedFolder: "The folder is open.",
    openFailed: (message, dir) => `${message} Folder: ${dir}`,
    whyNoArea: "Choose an area first.",
    whyChecking: "The preview is being computed …",
    whyTooLarge: "The area is too large.",
    whyOutside: "The area lies outside the supported states.",
    whyDisk: "Not enough free disk space (see above).",
    whyRunning: "A run is going on.",
    errScaleBarNoPdf: "The scale bar needs a PDF.",
    errSourcesNoPdf: "The source lines need a PDF.",
    saveFailed: "The area is not saved; no run was started.",
    pollLost: "No answer from the server. freemapper keeps trying.",
    runUnknown: "The server no longer knows this run.",
    failedPrefix: "Error: ",
  },
};

const RUN_ID_PREFIX = "run-";
const $r = (id) => document.getElementById(id);
const page = () => window.freemapperPage;
const RT = () => RUN_TEXTS[page() && page().lang() === "en" ? "en" : "de"];

/** The seven options of a run from the form inside `root`; the PDF-only ones are off without a PDF. */
function optionsFromForm(root) {
  const field = (id) => root.querySelector(`#${RUN_ID_PREFIX}${id}`);
  const anyPdf = field("pdf").checked || field("pdf-candidates").checked;
  const forceShown = !root.querySelector("#run-force-row").hidden;
  return {
    force: forceShown && field("force").checked,
    offline: field("offline").checked,
    ocad: field("ocad").checked,
    pdf: field("pdf").checked,
    pdf_candidates: field("pdf-candidates").checked,
    pdf_scale_bar: anyPdf && field("pdf-scale-bar").checked,
    pdf_sources: anyPdf ? field("pdf-sources").value : "auto",
  };
}

/** Text keys of the rules the options break; the server decides in the end. */
function validateOptions(options) {
  const anyPdf = options.pdf || options.pdf_candidates;
  const errors = [];
  if (options.pdf_scale_bar && !anyPdf) errors.push("errScaleBarNoPdf");
  if (options.pdf_sources !== "auto" && !anyPdf) errors.push("errSourcesNoPdf");
  return errors;
}

// --- State -----------------------------------------------------------------------------------

let jobId = null;
let since = 0;
let logLines = [];
let timer = null;
let pollStart = 0;
let cancelling = false;
let running = false;
let finished = null;
let starting = false;

function formatSize(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  const places = { maximumFractionDigits: 1, minimumFractionDigits: 1 };
  const locale = page().lang();
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toLocaleString(locale, places)} KB`;
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toLocaleString(locale, places)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toLocaleString(locale, places)} GB`;
}

function applyRunTexts() {
  const texts = RT();
  document.querySelectorAll("#run [data-text], #force-dialog [data-text]").forEach((el) => {
    const text = texts[el.dataset.text];
    if (typeof text === "string") el.textContent = text;
  });
  $r("run-log").setAttribute("aria-label", texts.logLabel);
  $r("run-start").textContent = finished ? texts.runAgain : texts.runStart;
  if (cancelling) $r("run-cancel").textContent = texts.runCancelling;
}

function alertText(text) {
  const el = $r("run-alert");
  el.textContent = text || "";
  el.hidden = !text;
}

/** Why the start button is off, or null when a run may start. */
function blockedReason() {
  const p = page();
  const preview = p.getPreview();
  const t = RT();
  if (running || starting) return t.whyRunning;
  if (!p.hasArea()) return t.whyNoArea;
  if (!preview) return t.whyChecking;
  if (preview.outside_coverage) return t.whyOutside;
  if (preview.too_large) return t.whyTooLarge;
  const disk = preview.estimate && preview.estimate.disk;
  if (disk && disk.stops && !$r("run-offline").checked) return t.whyDisk;
  return null;
}

/** Brings the controls in line with the page: options, hints, start button. */
function refresh() {
  if (!page() || $r("run").hidden) return;
  const anyPdf = $r("run-pdf").checked || $r("run-pdf-candidates").checked;
  $r("run-pdf-scale-bar").disabled = running || !anyPdf;
  $r("run-pdf-sources").disabled = running || !anyPdf;
  $r("run-pdf-hint").hidden = anyPdf;
  $r("run-sources-warning").hidden = !(anyPdf && $r("run-pdf-sources").value === "credit-only");
  for (const id of ["ocad", "pdf", "pdf-candidates", "offline", "force"]) {
    $r(`${RUN_ID_PREFIX}${id}`).disabled = running;
  }
  const reason = blockedReason();
  const why = $r("run-why");
  // Only a reason the user can act on is worth a line; "running" shows in the progress panel.
  why.textContent = reason && !running && !starting ? reason : "";
  why.hidden = !why.textContent;
  $r("run-start").disabled = reason !== null;
}

// --- Start -----------------------------------------------------------------------------------

/** Asks before a forced rebuild; resolves true only on "Replace". Cancel has the focus. */
function confirmForce() {
  const dialog = $r("force-dialog");
  if (typeof dialog.showModal !== "function") {
    return Promise.resolve(window.confirm(RT().forceText));
  }
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => resolve(dialog.returnValue === "confirm"), {
      once: true,
    });
    dialog.returnValue = "cancel";
    dialog.showModal();
  });
}

async function startRun() {
  const p = page();
  const t = RT();
  alertText("");
  const options = optionsFromForm($r("run"));
  if (validateOptions(options).length) {
    alertText(t[validateOptions(options)[0]]);
    return;
  }
  // The page never sends force without the dialog.
  if (options.force && !(await confirmForce())) {
    $r("run-start").focus();
    return;
  }
  starting = true;
  refresh();
  try {
    if (!(await p.ensureSaved())) {
      alertText(t.saveFailed);
      return;
    }
    const answer = await p.api("api/run", options);
    attach(answer.job_id, true);
  } catch (e) {
    if (e.status === 409 && e.body && e.body.job_id) {
      // A run of this folder is going on already (another tab, or MCP): follow it.
      alertText(e.message);
      attach(e.body.job_id, false);
    } else {
      // A 422 message carries the disk plan as text; a 409 the lock text.
      alertText(e.message);
    }
  } finally {
    starting = false;
    refresh();
  }
}

// --- Progress --------------------------------------------------------------------------------

function attach(id, focusCancel) {
  jobId = id;
  since = 0;
  logLines = [];
  cancelling = false;
  running = true;
  finished = null;
  pollStart = Date.now();
  page().setBusy(true);
  $r("run-result").hidden = true;
  $r("run-warnings").hidden = true;
  $r("run-progress").hidden = false;
  $r("run-cancel").disabled = false;
  $r("run-cancel").textContent = RT().runCancel;
  $r("run-step").textContent = RT().starting;
  $r("run-log").textContent = "";
  refresh();
  if (focusCancel) $r("run-cancel").focus();
  poll();
}

function renderProgress(status) {
  const t = RT();
  const phase = status.phase;
  $r("run-step").textContent = phase
    ? t.stepLine(phase.index, phase.total, t[phase.step] || phase.step)
    : t.starting;
  logLines = logLines.concat(status.lines || []).slice(-200);
  const shown = logLines.slice(-8).join("\n");
  const log = $r("run-log");
  if (log.textContent !== shown) {
    log.textContent = shown;
    log.scrollTop = log.scrollHeight;
  }
  renderWarnings(status.warnings || []);
}

function renderWarnings(warnings) {
  const list = $r("run-warning-list");
  list.replaceChildren(
    ...warnings.map((text) => {
      const li = document.createElement("li");
      li.textContent = text;
      return li;
    }),
  );
  $r("run-warnings").hidden = warnings.length === 0;
}

function schedule() {
  const gap = Date.now() - pollStart > 5 * 60 * 1000 ? 3000 : 1000;
  timer = setTimeout(poll, gap);
}

async function poll() {
  clearTimeout(timer);
  if (!jobId) return;
  const t = RT();
  let status;
  try {
    status = await page().api(
      `api/status?job=${encodeURIComponent(jobId)}&since=${since}`,
    );
  } catch (e) {
    if (e.status === 404) {
      endRun({ state: "failed", error: t.runUnknown, files: [], warnings: [] });
      return;
    }
    // 429 (too fast) and a short outage: try again, say so for the outage.
    if (e.status !== 429) $r("run-step").textContent = t.pollLost;
    schedule();
    return;
  }
  since = status.next;
  renderProgress(status);
  if (status.state === "running") {
    schedule();
  } else {
    endRun(status);
  }
}

async function cancelRun() {
  $r("run-cancel").disabled = true;
  $r("run-cancel").textContent = RT().runCancelling;
  cancelling = true;
  try {
    await page().api("api/cancel", { job_id: jobId });
  } catch (e) {
    // Nothing runs any more (409) or the answer was lost: the next status tells.
  }
}

// --- Result ----------------------------------------------------------------------------------

async function endRun(status) {
  clearTimeout(timer);
  jobId = null;
  running = false;
  cancelling = false;
  finished = status;
  $r("run-progress").hidden = true;
  renderResult(status);
  try {
    await page().reloadProject();
  } catch (e) {
    // The result stays; the force option may be out of date until the next reload.
  }
  page().setBusy(false);
  syncForceRow(page().getProject());
  applyRunTexts();
  refresh();
  $r("run-result-heading").focus();
}

function renderResult(status) {
  const t = RT();
  const heading = $r("run-result-heading");
  const text = $r("run-result-text");
  const done = status.state === "done";
  heading.textContent =
    status.state === "done"
      ? t.resultTitleDone
      : status.state === "cancelled"
        ? t.resultTitleCancelled
        : t.resultTitleFailed;
  text.textContent = done
    ? t.resultDone
    : status.state === "cancelled"
      ? t.resultCancelled
      : t.resultFailed(status.error || "");
  text.classList.toggle("error-text", status.state === "failed");
  const kept = $r("run-kept");
  kept.textContent = done && status.kept_map ? t.keptMap : "";
  kept.hidden = !kept.textContent;
  const files = status.files || [];
  const list = $r("run-files");
  if (files.length) {
    list.replaceChildren(
      ...files.map((f) => {
        const li = document.createElement("li");
        li.textContent = t.fileLine(f.path, t[`kind-${f.kind}`] || f.kind, formatSize(f.bytes));
        return li;
      }),
    );
  } else if (status.state === "failed") {
    list.replaceChildren(
      Object.assign(document.createElement("li"), { textContent: t.resultFailedKept }),
    );
  } else {
    list.replaceChildren(
      Object.assign(document.createElement("li"), { textContent: t.noFiles }),
    );
  }
  const project = page().getProject();
  $r("run-dir").textContent = project ? project.dir : "";
  $r("run-open-message").hidden = true;
  $r("run-result").hidden = false;
  renderWarnings(status.warnings || []);
}

async function openFolder() {
  const t = RT();
  const message = $r("run-open-message");
  try {
    const answer = await page().api("api/open-folder", {});
    message.textContent = answer.opened
      ? t.openedFolder
      : t.openFailed(answer.message || "", answer.dir || "");
  } catch (e) {
    message.textContent = t.openFailed(e.message, page().getProject().dir);
  }
  message.hidden = false;
}

/** The closing text of the page after a finished run, or null. */
function closingText() {
  if (!finished || finished.state !== "done") return null;
  const files = (finished.files || []).map((f) => f.path).join(", ");
  const project = page().getProject();
  return page().lang() === "en"
    ? `Created: ${files}. Folder: ${project.dir}`
    : `Erzeugt: ${files}. Ordner: ${project.dir}`;
}

// --- Init ------------------------------------------------------------------------------------

function syncForceRow(project) {
  const row = $r("run-force-row");
  row.hidden = !(project && project.has_map);
  if (row.hidden) $r("run-force").checked = false;
}

/** Called by app.js with the project once it is loaded. */
function init(project) {
  const defaults = project.run_defaults || {};
  $r("run-ocad").checked = !!defaults.ocad;
  $r("run-pdf").checked = !!defaults.pdf;
  $r("run-pdf-candidates").checked = !!defaults.pdf_candidates;
  $r("run-pdf-scale-bar").checked = !!defaults.pdf_scale_bar;
  $r("run-pdf-sources").value = defaults.pdf_sources || "auto";
  $r("run-offline").checked = !!defaults.offline;
  // `force` is never preset: replacing a map is a decision of the moment.
  $r("run-force").checked = false;
  syncForceRow(project);
  $r("run").hidden = false;
  applyRunTexts();
  refresh();
  if (project.running && project.running.job_id) attach(project.running.job_id, false);
}

for (const id of ["ocad", "pdf", "pdf-candidates", "pdf-scale-bar", "pdf-sources", "offline", "force"]) {
  $r(`${RUN_ID_PREFIX}${id}`).addEventListener("change", refresh);
}
$r("run-start").addEventListener("click", startRun);
$r("run-cancel").addEventListener("click", cancelRun);
$r("run-open").addEventListener("click", openFolder);

window.freemapperRun = {
  init,
  refresh,
  closingText,
  optionsFromForm,
  validateOptions,
  RUN_TEXTS,
};
