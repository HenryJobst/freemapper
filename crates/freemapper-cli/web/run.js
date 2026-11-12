// freemapper run panel: options, start, progress, cancel, result. Runs after app.js, which
// offers `window.freemapperPage`. Only textContent is used to show server text.
// Its words come from api/texts through the suite's texts.js (core ADR 0030, ADR 0096).
import { language, t } from "../suite/texts.js";

const RUN_ID_PREFIX = "run-";
const $r = (id) => document.getElementById(id);
const page = () => window.freemapperPage;

/** The text `id`, or `fallback` when the catalog has none (a step or file kind added later). */
function tOr(id, fallback, args = {}) {
  const text = t(id, args);
  return text === id ? fallback : text;
}

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

/** Text ids of the rules the options break; the server decides in the end. */
function validateOptions(options) {
  const anyPdf = options.pdf || options.pdf_candidates;
  const errors = [];
  if (options.pdf_scale_bar && !anyPdf) errors.push("page-err-scale-bar-no-pdf");
  if (options.pdf_sources !== "auto" && !anyPdf) errors.push("page-err-sources-no-pdf");
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
  const locale = language();
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toLocaleString(locale, places)} KB`;
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toLocaleString(locale, places)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toLocaleString(locale, places)} GB`;
}

/** The texts of the buttons that change with the state of the run. */
function applyRunTexts() {
  $r("run-start").textContent = t(finished ? "page-run-again" : "page-run-start");
  if (cancelling) $r("run-cancel").textContent = t("page-run-cancelling");
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
  if (running || starting) return t("page-why-running");
  if (!p.hasArea()) return t("page-why-no-area");
  if (!preview) return t("page-why-checking");
  if (preview.outside_coverage) return t("page-why-outside");
  if (preview.too_large) return t("page-why-too-large");
  const disk = preview.estimate && preview.estimate.disk;
  if (disk && disk.stops && !$r("run-offline").checked) return t("page-why-disk");
  return null;
}

/** Brings the controls in line with the page: options, hints, start button. */
function refresh() {
  if (!page() || $r("run").hidden) return;
  const p = page();
  const anyPdf = $r("run-pdf").checked || $r("run-pdf-candidates").checked;
  $r("run-pdf-scale-bar").disabled = running || !anyPdf;
  $r("run-pdf-sources").disabled = running || !anyPdf;
  $r("run-pdf-hint").hidden = anyPdf;
  $r("run-sources-warning").hidden = !(anyPdf && $r("run-pdf-sources").value === "credit-only");
  for (const id of ["ocad", "pdf", "pdf-candidates", "offline", "force"]) {
    $r(`${RUN_ID_PREFIX}${id}`).disabled = running;
  }
  // The area on the page became saved or unsaved: that decides whether live images may come.
  const areaSaved = p.areaIsSaved();
  const areaChanged = previewAreaSaved !== null && areaSaved !== previewAreaSaved;
  previewAreaSaved = areaSaved;
  if (areaChanged && previewState) decidePreview(0);
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
    return Promise.resolve(window.confirm(t("page-force-text")));
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
  alertText("");
  const options = optionsFromForm($r("run"));
  if (validateOptions(options).length) {
    alertText(t(validateOptions(options)[0]));
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
      alertText(t("page-save-failed"));
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
    // A refused or failed start: the page may show a state that the save changed.
    if (!running) loadPreviewState().then(() => decidePreview(0));
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
  $r("run-cancel").textContent = t("page-run-cancel");
  $r("run-step").textContent = t("page-starting");
  $r("run-log").textContent = "";
  refresh();
  if (focusCancel) $r("run-cancel").focus();
  poll();
}

function renderProgress(status) {
  const phase = status.phase;
  $r("run-step").textContent = phase
    ? t("page-step-line", {
      index: phase.index,
      total: phase.total,
      label: tOr(`page-${phase.step}`, phase.step),
    })
    : t("page-starting");
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
  let status;
  try {
    status = await page().api(
      `api/status?job=${encodeURIComponent(jobId)}&since=${since}`,
    );
  } catch (e) {
    if (e.status === 404) {
      endRun({ state: "failed", error: t("page-run-unknown"), files: [], warnings: [] });
      return;
    }
    // 429 (too fast) and a short outage: try again, say so for the outage.
    if (e.status !== 429) $r("run-step").textContent = t("page-poll-lost");
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
  $r("run-cancel").textContent = t("page-run-cancelling");
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
  runEnded();
}

function renderResult(status) {
  const heading = $r("run-result-heading");
  const text = $r("run-result-text");
  const done = status.state === "done";
  heading.textContent = t(
    status.state === "done"
      ? "page-result-title-done"
      : status.state === "cancelled"
        ? "page-result-title-cancelled"
        : "page-result-title-failed",
  );
  text.textContent = done
    ? t("page-result-done")
    : status.state === "cancelled"
      ? t("page-result-cancelled")
      : t("page-result-failed", { error: status.error || "" });
  text.classList.toggle("error-text", status.state === "failed");
  const kept = $r("run-kept");
  kept.textContent = done && status.kept_map ? t("page-kept-map") : "";
  kept.hidden = !kept.textContent;
  const files = status.files || [];
  const list = $r("run-files");
  if (files.length) {
    list.replaceChildren(
      ...files.map((f) => {
        const li = document.createElement("li");
        li.textContent = t("page-file-line", {
          path: f.path,
          kind: tOr(`page-file-kind-${f.kind}`, f.kind),
          size: formatSize(f.bytes),
        });
        return li;
      }),
    );
  } else if (status.state === "failed") {
    list.replaceChildren(
      Object.assign(document.createElement("li"), { textContent: t("page-result-failed-kept") }),
    );
  } else {
    list.replaceChildren(
      Object.assign(document.createElement("li"), { textContent: t("page-no-files") }),
    );
  }
  const project = page().getProject();
  $r("run-dir").textContent = project ? project.dir : "";
  $r("run-open-message").hidden = true;
  $r("run-result").hidden = false;
  renderWarnings(status.warnings || []);
}

async function openFolder() {
  const message = $r("run-open-message");
  try {
    const answer = await page().api("api/open-folder", {});
    message.textContent = answer.opened
      ? t("page-opened-folder")
      : t("page-open-failed", { message: answer.message || "", dir: answer.dir || "" });
  } catch (e) {
    message.textContent = t("page-open-failed", { message: e.message, dir: page().getProject().dir });
  }
  message.hidden = false;
}

/** The closing text of the page after a finished run, or null. */
function closingText() {
  if (!finished || finished.state !== "done") return null;
  const files = (finished.files || []).map((f) => f.path).join(", ");
  const project = page().getProject();
  return t("page-made-files", { files, dir: project.dir });
}

// --- Map preview -----------------------------------------------------------------------------
// The record next to the stored image (api/map-image/state) says which settings that image shows
// and whether the saved project still fits it. The image is the stored one when the page's content
// settings are the recorded ones, otherwise a live one for the page's settings (api/map-image).

// How long the content settings must stay unchanged before a live image is asked for.
const PREVIEW_DELAY_MS = 400;
// How often a live image that another tab replaced is asked for again.
const PREVIEW_RETRIES = 2;

// The answer of api/map-image/state, or null before it is read.
let previewState = null;
// Answers and images of older requests are dropped.
let previewSeq = 0;
let previewTimer = null;
// Makes the address of the stored image change, so the browser loads it again.
let previewVersion = 0;
// The live image on show (its id), or null.
let previewLive = null;
let previewRetries = 0;
// Whether the area on the page was the saved one when last looked at; null before.
let previewAreaSaved = null;

const previewImage = () => $r("preview-image");

function previewLine(id, isError = false) {
  const line = $r("preview-state");
  line.textContent = id ? t(id) : "";
  line.classList.toggle("error-text", isError);
}

function previewShow(src) {
  const image = previewImage();
  image.alt = t("page-preview-alt");
  image.src = src;
  image.hidden = false;
  image.classList.remove("computing");
  const link = $r("preview-open");
  link.href = src;
  $r("preview-open-row").hidden = false;
}

function previewHide() {
  previewImage().hidden = true;
  previewImage().removeAttribute("src");
  $r("preview-open-row").hidden = true;
  previewLive = null;
}

function previewShowStored(lineId) {
  previewSeq += 1;
  previewLive = null;
  previewVersion += 1;
  previewShow(`api/map-image?v=${previewVersion}`);
  previewLine(lineId);
}

/** Reads api/map-image/state; the page shows no image without it. */
async function loadPreviewState() {
  try {
    previewState = await page().api("api/map-image/state");
  } catch (e) {
    previewState = null;
  }
}

/** Shows what fits the state now: the stored image, or a live one after a pause. */
function decidePreview(delay = 0) {
  clearTimeout(previewTimer);
  previewTimer = setTimeout(showPreviewNow, delay);
}

async function showPreviewNow() {
  if (!previewState || $r("run").hidden) return;
  if (!previewState.stored) {
    previewSeq += 1;
    previewHide();
    const project = page().getProject();
    previewLine(project && project.has_map ? "page-preview-later" : "page-preview-none");
    return;
  }
  // A run is starting or going on: its end brings the new image. A page opened meanwhile shows the
  // stored one until then; a computation that was asked for before leaves its line and its
  // dimming behind, so settle them.
  if (running || starting) {
    if (previewImage().hidden) {
      previewShowStored("page-preview-run");
      return;
    }
    previewImage().classList.remove("computing");
    previewLine(previewLive === null ? "page-preview-run" : previewSettledLine());
    return;
  }
  // The live images are of the saved area: a drawn area that is not saved yet gets none.
  if (!page().areaIsSaved()) {
    previewShowStored("page-preview-area-unsaved");
    return;
  }
  if (!previewState.fresh) {
    previewShowStored("page-preview-stale");
    return;
  }
  const recorded = previewState.content;
  if (recorded && page().sameContent(recorded)) {
    previewShowStored("page-preview-stored");
    return;
  }
  await requestLivePreview();
}

/** The line for the image on show when nothing is computed. */
function previewSettledLine() {
  if (previewLive !== null) {
    return page().contentIsSaved() ? "page-preview-live-saved" : "page-preview-live";
  }
  return previewState && !previewState.fresh ? "page-preview-stale" : "page-preview-stored";
}

async function requestLivePreview() {
  const mine = ++previewSeq;
  previewImage().classList.add("computing");
  previewLine("page-preview-computing");
  let answer;
  try {
    answer = await page().api("api/map-image", { content: page().contentBody() });
  } catch (e) {
    if (mine !== previewSeq) return;
    previewImage().classList.remove("computing");
    // A run started meanwhile: its end brings the image.
    if (running || starting) {
      decidePreview(0);
      return;
    }
    await loadPreviewState();
    if (mine !== previewSeq) return;
    if (previewState && !previewState.fresh) {
      previewShowStored("page-preview-stale");
    } else {
      const line = $r("preview-state");
      line.textContent = e.message;
      line.classList.add("error-text");
    }
    return;
  }
  if (mine !== previewSeq) return;
  previewLive = answer.id;
  previewShow(`api/map-image/live/${answer.id}`);
  previewImage().classList.add("computing");
  // The line changes when the image has loaded (the load handler below).
}

previewImage().addEventListener("load", () => {
  if (previewLive === null) return;
  previewImage().classList.remove("computing");
  previewRetries = 0;
  previewLine(page().contentIsSaved() ? "page-preview-live-saved" : "page-preview-live");
});

previewImage().addEventListener("error", () => {
  if (previewLive === null) return;
  // 404: another tab replaced this image with a newer one. Ask again, a few times at most; then
  // go back to what the record says.
  previewLive = null;
  if (previewRetries < PREVIEW_RETRIES) {
    previewRetries += 1;
    decidePreview(0);
  } else {
    previewRetries = 0;
    previewShowStored("page-preview-stored");
  }
});

/** The content settings on the page changed. */
function contentChanged() {
  if (!previewState) return;
  previewRetries = 0;
  // The shown image is out of date from now on; a newer answer replaces it.
  previewSeq += 1;
  decidePreview(PREVIEW_DELAY_MS);
}

/** The project was saved: the record may not fit it any more. */
async function projectSaved() {
  await loadPreviewState();
  decidePreview(0);
}

/** A run ended: a new image and a new record. */
async function runEnded() {
  await loadPreviewState();
  decidePreview(0);
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
  loadPreviewState().then(() => decidePreview(0));
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
  contentChanged,
  projectSaved,
  optionsFromForm,
  validateOptions,
};
