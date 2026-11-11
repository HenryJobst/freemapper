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
};
