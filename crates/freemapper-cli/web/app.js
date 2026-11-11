// freemapper area selection page: draw or pick the map area, see tiles and download, save it.
// Every server call goes through the suite's api.js, every visible word comes from api/texts
// through the suite's texts.js (core ADR 0030, ADR 0096). Leaflet is a classic script before this
// module; run.js comes after it.
import { get, post } from "../suite/api.js";
import { language, loadTexts, t, tn } from "../suite/texts.js";
import { clearBanner, showBanner } from "../suite/status.js";

const $ = (id) => document.getElementById(id);

/** The text of an error answer: the server's sentence, else the status. */
function errorText(answer) {
  const data = answer.data;
  if (data && typeof data === "object" && typeof data.error === "string") return data.error;
  if (typeof data === "string" && data) return data;
  return `HTTP ${answer.status}`;
}

/**
 * A GET (without `body`) or a POST to the page's server; the answer's data, or an error with
 * `status` and `body` for callers that need more than the text (409 with a job or a revision, 422
 * with the disk plan). `revision` goes into the revision header of a POST.
 */
async function api(path, body, revision = null) {
  const answer = body === undefined ? await get(path) : await post(path, body, revision);
  if (!answer.ok) {
    const error = new Error(errorText(answer));
    error.status = answer.status;
    error.body = answer.data;
    throw error;
  }
  return answer.data;
}

// --- Map -------------------------------------------------------------------------------------

const map = L.map("map", { doubleClickZoom: true }).setView([52.45, 13.4], 8);
// Through the local server, which sends the User-Agent OpenStreetMap's tile policy asks for.
const osm = L.tileLayer("tiles/osm/{z}/{x}/{y}.png", {
  maxZoom: 19,
  attribution: '© <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
}).addTo(map);
const aerial = L.tileLayer.wms("https://isk.geobasis-bb.de/mapproxy/dop20c/service/wms", {
  layers: "bebb_dop20c",
  format: "image/png",
  maxZoom: 20,
  attribution: "© GeoBasis-DE/LGB, dl-de/by-2-0",
});
const aerialSaxony = L.tileLayer.wms("https://geodienste.sachsen.de/wms_geosn_dop-rgb/guest", {
  layers: "sn_dop_020",
  format: "image/jpeg",
  maxZoom: 20,
  attribution: "Quelle: GeoSN, dl-de/by-2-0",
});
const aerialSaxonyAnhalt = L.tileLayer.wms(
  "https://www.geodatenportal.sachsen-anhalt.de/wss/service/ST_LVermGeo_DOP_WMS_OpenData/guest",
  {
    layers: "lsa_lvermgeo_dop20_2",
    format: "image/jpeg",
    maxZoom: 20,
    attribution: "© GeoBasis-DE / LVermGeo ST, dl-de/by-2-0",
  },
);
const aerialHamburg = L.tileLayer.wms("https://geodienste.hamburg.de/wms_dop_zeitreihe_unbelaubt", {
  layers: "dop_zeitreihe_unbelaubt",
  format: "image/jpeg",
  maxZoom: 20,
  // The open 2026 vintage; 2023 is a closed-licence satellite image.
  time: "2026",
  attribution: "Freie und Hansestadt Hamburg, LGV, dl-de/by-2-0",
});
// `he_dop_rgb` is served at all scales (`he_dop20_rgb` only below 1:9000, blank when zoomed out).
const aerialHessen = L.tileLayer.wms(
  "https://www.gds-srv.hessen.de/cgi-bin/lika-services/ogc-free-images.ows",
  {
    layers: "he_dop_rgb",
    format: "image/jpeg",
    maxZoom: 20,
    attribution: "Geobasisdaten © HVBG, dl-de/zero-2-0",
  },
);
// The current DOP20 mosaic (EPSG:3857 is among the layer's CRS).
const aerialSchleswigHolstein = L.tileLayer.wms(
  "https://dienste.gdi-sh.de/WMS_SH_DOP20col_OpenGBD",
  {
    layers: "sh_dop20_rgb",
    format: "image/jpeg",
    maxZoom: 20,
    attribution: "© GeoBasis-DE/LVermGeo SH/CC BY 4.0",
  },
);
// The ADV DOP20 mosaic (EPSG:3857 is offered); the WMS stamps a watermark, fine for a preview.
const aerialMecklenburgVorpommern = L.tileLayer.wms(
  "https://www.geodaten-mv.de/dienste/adv_dop",
  {
    layers: "mv_dop",
    format: "image/jpeg",
    maxZoom: 20,
    attribution: "©GeoBasis-DE/MV/CC BY 4.0",
  },
);
// The current DOP mosaic (EPSG:3857 is among the layer's CRS).
const aerialNordrheinWestfalen = L.tileLayer.wms("https://www.wms.nrw.de/geobasis/wms_nw_dop", {
  layers: "nw_dop_rgb",
  format: "image/jpeg",
  maxZoom: 20,
  attribution: "Geobasisdaten © Geobasis NRW (Bezirksregierung Köln), dl-de/zero-2-0",
});
// The newest DOP20 (EPSG:3857 is among the service's CRS); Bremen is left white.
const aerialNiedersachsen = L.tileLayer.wms(
  "https://opendata.lgln.niedersachsen.de/doorman/noauth/dop_wms",
  {
    layers: "ni_dop20",
    format: "image/jpeg",
    maxZoom: 20,
    attribution: "© GeoBasis-DE/LGLN, CC BY 4.0",
  },
);
// The current DOP20 (EPSG:3857 is among the layer's CRS); white beyond the state border.
const aerialRheinlandPfalz = L.tileLayer.wms("https://geo4.service24.rlp.de/wms/rp_dop20.fcgi", {
  layers: "rp_dop20",
  format: "image/jpeg",
  maxZoom: 20,
  attribution: `©GeoBasis-DE / LVermGeoRP${new Date().getFullYear()}, dl-de/by-2-0, www.lvermgeo.rlp.de`,
});
/** The layer control, with its names in the page language: built once the texts are loaded. */
function showLayerNames() {
  L.control
    .layers({
      [t("page-layer-osm")]: osm,
      [t("page-layer-aerial")]: aerial,
      [t("page-layer-aerial-saxony")]: aerialSaxony,
      [t("page-layer-aerial-saxony-anhalt")]: aerialSaxonyAnhalt,
      [t("page-layer-aerial-hamburg")]: aerialHamburg,
      [t("page-layer-aerial-hessen")]: aerialHessen,
      [t("page-layer-aerial-schleswig-holstein")]: aerialSchleswigHolstein,
      [t("page-layer-aerial-mecklenburg-vorpommern")]: aerialMecklenburgVorpommern,
      [t("page-layer-aerial-nordrhein-westfalen")]: aerialNordrheinWestfalen,
      [t("page-layer-aerial-niedersachsen")]: aerialNiedersachsen,
      [t("page-layer-aerial-rheinland-pfalz")]: aerialRheinlandPfalz,
    })
    .addTo(map);
}
map.attributionControl.addAttribution(
  '<a href="https://github.com/HenryJobst/freemapper">freemapper</a>',
);

const tileLayer = L.layerGroup().addTo(map);
const areaLayer = L.layerGroup().addTo(map);
const drawLayer = L.layerGroup().addTo(map);

// --- State -----------------------------------------------------------------------------------

let project = null;
// The chosen area: GeoJSON polygon geometry and the OSM object it came from (or null).
let area = null;
let sourceId = null;
let sourceName = null;
let preview = null;
// Answers to older preview requests are dropped.
let previewRequest = 0;
let savedHere = false;
// True while a run goes on: the area, the settings and "Save" are locked.
let busy = false;
// The area and source as last saved or loaded; a different one needs a save before a run.
let savedSignature = null;

function signature() {
  return JSON.stringify({ area, sourceId });
}

/** The signature of the area that `project` holds, as `signature()` gives it once shown. */
function savedSignatureOf(project) {
  const saved = project && project.area ? geometryOf(project.area) : null;
  return JSON.stringify({ area: saved, sourceId: saved ? project.source_id : null });
}

/** Tells the run panel that the area, the preview or the project changed. */
function notifyRun() {
  if (window.freemapperRun) window.freemapperRun.refresh();
}

/** The enabled state of every control that a run locks. */
function updateControls() {
  const unusable = !area || !preview || preview.too_large || preview.outside_coverage;
  $("save").disabled = busy || unusable;
  for (const id of ["draw-rectangle", "draw-polygon", "finish-polygon", "clear", "query", "done"]) {
    $(id).disabled = busy;
  }
  $("search").querySelector("button").disabled = busy;
  for (const id of ["kind", "scale", "title"]) $(id).disabled = busy || (project && project.exists);
}

function setMessage(text, kind) {
  const el = $("message");
  el.textContent = text;
  el.className = kind || "";
}

function geometryOf(geojson) {
  if (geojson.type === "Feature") return geometryOf(geojson.geometry);
  if (geojson.type === "MultiPolygon" && geojson.coordinates.length === 1) {
    return { type: "Polygon", coordinates: geojson.coordinates[0] };
  }
  return geojson;
}

function showArea(fit) {
  areaLayer.clearLayers();
  if (!area) return;
  const shape = L.geoJSON(area, {
    style: { color: "#d4145a", weight: 3, fillOpacity: 0.08 },
    interactive: false,
  });
  shape.addTo(areaLayer);
  if (fit) map.fitBounds(shape.getBounds(), { padding: [40, 40], maxZoom: 18 });
}

async function chooseArea(geojson, id, name, fit) {
  area = geometryOf(geojson);
  sourceId = id;
  sourceName = name;
  showArea(fit);
  await updatePreview();
}

async function updatePreview() {
  if (!area) return;
  const request = ++previewRequest;
  $("save").disabled = true;
  notifyRun();
  try {
    const body = { area, source_id: sourceId };
    if (project && !project.exists) body.kind = $("kind").value;
    const answer = await api("api/preview", body);
    if (request !== previewRequest) return;
    preview = answer;
    showPreview();
    updateControls();
    setMessage("");
  } catch (e) {
    if (request !== previewRequest) return;
    preview = null;
    showPreview();
    updateControls();
    setMessage(t("page-failed", { error: e.message }), "error");
  }
  notifyRun();
}

function showPreview() {
  tileLayer.clearLayers();
  $("summary").hidden = !area;
  if (!area) return;
  $("source").textContent = sourceId
    ? t("page-source-osm", { name: sourceName || sourceId })
    : t("page-source-drawn");
  if (!preview) {
    ["size", "tiles", "download", "disk"].forEach((id) => ($(id).textContent = ""));
    $("items").replaceChildren();
    $("too-large").hidden = true;
    $("no-room").hidden = true;
    return;
  }
  $("size").textContent = t("page-size", {
    width: Math.round(preview.width_m),
    height: Math.round(preview.height_m),
  });
  const unusable = preview.too_large || preview.outside_coverage;
  $("too-large").hidden = !unusable;
  $("too-large").textContent = preview.outside_coverage
    ? t("page-outside")
    : t("page-too-large", { max: Math.round(preview.max_side_m) });
  const names = preview.tiles.map((t) => t.name);
  $("tiles").textContent = unusable
    ? ""
    : tn("page-tiles", names.length, { names: names.join(", ") });
  const mb = preview.estimate.total_megabytes;
  $("download").textContent = unusable
    ? ""
    : mb > 0 ? t("page-download", { mb: Math.ceil(mb) }) : t("page-download-none");
  $("items").replaceChildren(
    ...preview.estimate.items.map((item) => {
      const li = document.createElement("li");
      const note = item.cached
        ? ` (${t("page-cached")})`
        : item.fits === false ? ` (${t("page-no-room")})` : "";
      li.textContent = `${item.what}: ${item.megabytes} MB${note}`;
      li.classList.toggle("warning", !item.cached && item.fits === false);
      return li;
    }),
  );
  showDisk(unusable ? null : preview.estimate);
  for (const tile of preview.tiles) {
    L.polygon(tile.corners, {
      color: "#ff7800",
      weight: 2,
      dashArray: "8 6",
      fill: false,
      interactive: false,
    })
      .bindTooltip(tile.name, { permanent: true, direction: "center", className: "tile-label" })
      .addTo(tileLayer);
  }
}

/** `350 MB` or `1,2 GB` in the page's language. */
function megabytes(mb) {
  if (mb < 1024) return `${Math.ceil(mb)} MB`;
  const gb = (mb / 1024).toLocaleString(language(), {
    maximumFractionDigits: 1,
    minimumFractionDigits: 1,
  });
  return `${gb} GB`;
}

/** The free disk space of the cache and project volumes, and a hint when not everything fits. */
function showDisk(estimate) {
  const disk = estimate && estimate.disk;
  const volumes = (disk && disk.volumes) || [];
  const named = volumes.length > 1;
  $("disk").textContent = volumes.length
    ? t("page-disk", {
      volumes: volumes
        .map((v) => {
          const where = t(v.holds.includes("cache") ? "page-cache-volume" : "page-project-volume");
          const text = t("page-disk-volume", {
            free: megabytes(v.free_megabytes),
            needed: megabytes(v.needed_megabytes),
          });
          return named ? `${where}: ${text}` : text;
        })
        .join("; "),
    })
    : "";
  const messages = (disk && disk.messages) || [];
  const notice = $("no-room");
  notice.replaceChildren();
  if (disk && disk.error) {
    notice.textContent = t("page-disk-unknown", { error: disk.error });
  } else if (messages.length) {
    const reserve = megabytes(disk.reserve_megabytes);
    const list = document.createElement("ul");
    list.replaceChildren(
      ...messages.map((text) => {
        const li = document.createElement("li");
        li.textContent = text;
        return li;
      }),
    );
    notice.append(t(disk.stops ? "page-no-room-stop" : "page-no-room-skip", { reserve }), list);
  }
  notice.hidden = !notice.textContent;
}

// --- Search ----------------------------------------------------------------------------------

$("search").addEventListener("submit", async (event) => {
  event.preventDefault();
  const query = $("query").value.trim();
  if (!query) return;
  const list = $("results");
  list.replaceChildren(Object.assign(document.createElement("li"), { textContent: t("page-searching") }));
  try {
    const places = await api(`api/places?q=${encodeURIComponent(query)}`);
    if (places.length === 0) {
      list.replaceChildren(Object.assign(document.createElement("li"), { textContent: t("page-no-results") }));
      return;
    }
    list.replaceChildren(
      ...places.map((place) => {
        const li = document.createElement("li");
        li.textContent = place.display_name;
        if (!place.has_area) {
          li.classList.add("point");
          li.title = t("page-point-only");
          li.textContent += ` (${li.title})`;
          return li;
        }
        li.addEventListener("click", async () => {
          if (busy) return;
          setMessage(t("page-searching"));
          try {
            const found = await api(`api/place?osm_id=${encodeURIComponent(place.osm_id)}`);
            await chooseArea(found.area, found.source_id, place.name || place.display_name, true);
          } catch (e) {
            setMessage(t("page-failed", { error: e.message }), "error");
          }
        });
        return li;
      }),
    );
  } catch (e) {
    list.replaceChildren();
    setMessage(t("page-failed", { error: e.message }), "error");
  }
});

// --- Drawing ---------------------------------------------------------------------------------

let mode = null;
let points = [];

function setMode(next) {
  mode = next;
  points = [];
  drawLayer.clearLayers();
  $("draw-rectangle").classList.toggle("active", mode === "rectangle");
  $("draw-polygon").classList.toggle("active", mode === "polygon");
  $("finish-polygon").hidden = mode !== "polygon";
  $("map").classList.toggle("drawing", mode !== null);
  if (mode) map.doubleClickZoom.disable();
  else map.doubleClickZoom.enable();
  showHint();
}

/** The hint under the drawing buttons, for the current mode in the page language. */
function showHint() {
  $("hint").textContent = t(
    mode === "rectangle"
      ? "page-hint-rectangle"
      : mode === "polygon" ? "page-hint-polygon" : "page-hint-idle",
  );
}

function ring(latlngs) {
  const coordinates = latlngs.map((p) => [p.lng, p.lat]);
  coordinates.push(coordinates[0]);
  return { type: "Polygon", coordinates: [coordinates] };
}

function redraw(cursor) {
  drawLayer.clearLayers();
  // Not interactive: clicks must reach the map, or a corner drawn under the cursor would
  // swallow the second click of a double-click.
  const style = { color: "#0b6bcb", weight: 2, dashArray: "6 4", interactive: false };
  if (mode === "rectangle" && points.length === 1 && cursor) {
    L.rectangle(L.latLngBounds(points[0], cursor), style).addTo(drawLayer);
  }
  if (mode === "polygon" && points.length > 0) {
    const line = cursor ? [...points, cursor] : points;
    L.polyline(line, style).addTo(drawLayer);
    points.forEach((p, i) =>
      L.circleMarker(p, { radius: i === 0 ? 6 : 4, color: "#0b6bcb", interactive: false }).addTo(
        drawLayer,
      ),
    );
  }
}

function finishPolygon() {
  if (points.length < 3) return;
  const geometry = ring(points);
  setMode(null);
  chooseArea(geometry, null, null, false);
}

map.on("click", (event) => {
  if (busy) return;
  if (mode === "rectangle") {
    points.push(event.latlng);
    if (points.length === 2) {
      const b = L.latLngBounds(points[0], points[1]);
      const geometry = ring([b.getSouthWest(), b.getSouthEast(), b.getNorthEast(), b.getNorthWest()]);
      setMode(null);
      chooseArea(geometry, null, null, false);
    } else {
      redraw(event.latlng);
    }
  } else if (mode === "polygon") {
    // Clicking near the first corner closes the polygon.
    if (points.length >= 3) {
      const first = map.latLngToContainerPoint(points[0]);
      if (first.distanceTo(event.containerPoint) < 10) {
        finishPolygon();
        return;
      }
    }
    points.push(event.latlng);
    redraw(event.latlng);
  }
});

map.on("mousemove", (event) => {
  if (mode && points.length > 0) redraw(event.latlng);
});

map.on("dblclick", () => {
  if (mode === "polygon") {
    // The double-click's two clicks added the same corner twice.
    points.pop();
    finishPolygon();
  }
});

document.addEventListener("keydown", (event) => {
  if (!mode || event.target.tagName === "INPUT") return;
  if (event.key === "Escape") setMode(null);
  if (event.key === "Backspace" && mode === "polygon") {
    points.pop();
    redraw();
    event.preventDefault();
  }
});

$("draw-rectangle").addEventListener("click", () => setMode(mode === "rectangle" ? null : "rectangle"));
$("draw-polygon").addEventListener("click", () => setMode(mode === "polygon" ? null : "polygon"));
$("finish-polygon").addEventListener("click", finishPolygon);
$("clear").addEventListener("click", () => {
  setMode(null);
  area = null;
  sourceId = null;
  preview = null;
  areaLayer.clearLayers();
  showPreview();
  updateControls();
  setMessage("");
  notifyRun();
});

// --- Settings, save, done --------------------------------------------------------------------

// Fills the scale list for `kind` from the server's table (`allowed_scales`, ADR 0085; the first
// is the default); a stored scale outside the table stays selectable.
function fillScales(kind, current) {
  const table = (project && project.allowed_scales) || {};
  const list = [...(table[kind] || (current ? [current] : []))];
  if (current && !list.includes(current)) list.push(current);
  $("scale").replaceChildren(...list.map((s) => new Option(String(s), String(s))));
  $("scale").value = String(current || list[0]);
}

$("kind").addEventListener("change", () => {
  fillScales($("kind").value);
  // The buffer depends on the map kind.
  updatePreview();
});

$("save").addEventListener("click", saveArea);

/** Saves the area (and, for a new project, kind, scale and title); true when it worked. */
async function saveArea() {
  if (!area || busy) return false;
  setMessage(t("page-saving"));
  clearBanner();
  $("save").disabled = true;
  try {
    const body = { area, source_id: sourceId };
    if (!project.exists) {
      body.kind = $("kind").value;
      body.scale = Number($("scale").value) || undefined;
      body.title = $("title").value.trim() || undefined;
    }
    // The revision of freemapper.toml as this page read it; none before the file exists.
    const answer = await api("api/save", body, project.revision);
    savedHere = true;
    savedSignature = signature();
    setMessage(
      t(
        answer.zone_changed
          ? "page-zone-changed"
          : answer.provider_changed
            ? "page-provider-changed"
            : "page-saved",
      ),
      "ok",
    );
    $("has-map").hidden = !answer.has_map;
    project = await api("api/project");
    showSettings();
    return true;
  } catch (e) {
    if (e.status === 409 && e.body && typeof e.body.revision === "string") {
      // Another window or program wrote freemapper.toml since this page read it (core ADR 0030):
      // nothing was saved. The page takes the saved state and keeps the area it shows.
      setMessage("");
      showBanner(e.message);
      try {
        await reloadProject();
        savedSignature = savedSignatureOf(project);
      } catch (reload) {
        // The banner stays; the next save asks the server again.
      }
    } else {
      setMessage(t("page-failed", { error: e.message }), "error");
    }
    return false;
  } finally {
    updateControls();
    notifyRun();
  }
}

/** True when the shown area is the one in freemapper.toml. */
function isSaved() {
  return !!project && project.exists && signature() === savedSignature;
}

/** Saves first when the area changed; true when the project on disk matches the page. */
async function ensureSaved() {
  return isSaved() ? true : saveArea();
}

$("done").addEventListener("click", async () => {
  try {
    await api("api/done", {});
  } catch (e) {
    // The server may close before it answers; the page is finished either way.
  }
  const made = window.freemapperRun && window.freemapperRun.closingText();
  $("next").textContent = !savedHere && !made
    ? t("page-nothing-saved")
    : project && project.from_agent
      ? t("page-back-to-agent")
      : made || t("page-next", { dir: project ? project.dir : "" });
  $("closed").hidden = false;
});

function showSettings() {
  $("kind").value = project.kind;
  fillScales(project.kind, project.scale);
  $("title").value = project.title || "";
  updateControls();
  $("settings-fixed").hidden = !project.exists;
}

/** A run starts or ends: the area controls lock while it goes on. */
function setBusy(value) {
  busy = value;
  if (busy) setMode(null);
  updateControls();
}

/** Reads the project again: its settings, whether it has a map, the revision of freemapper.toml. */
async function reloadProject() {
  project = await api("api/project");
  $("has-map").hidden = !project.has_map;
  showSettings();
  return project;
}

// What the run panel (run.js) may use.
window.freemapperPage = {
  api,
  getProject: () => project,
  getPreview: () => preview,
  hasArea: () => !!area,
  isSaved,
  ensureSaved,
  setBusy,
  reloadProject,
};

// --- Update notice ---------------------------------------------------------------------------

/** One quiet line when the daily check found a newer version; no button, the command does it. */
async function showUpdateNotice() {
  try {
    const { available } = await api("api/update");
    if (!available) return;
    const notice = $("update-notice");
    notice.replaceChildren(t("page-update-available", { version: available.version }), " ");
    const command = document.createElement("code");
    command.textContent = "freemapper update";
    notice.append(command, ".");
    if (typeof available.notes === "string" && available.notes.startsWith("https://github.com/")) {
      const link = document.createElement("a");
      link.href = available.notes;
      link.target = "_blank";
      link.rel = "noopener";
      link.textContent = t("page-update-notes");
      notice.append(" ", link);
    }
    notice.hidden = false;
  } catch (e) {
    // The notice is a courtesy; the page works without it.
  }
}

// --- Start -----------------------------------------------------------------------------------

async function start() {
  try {
    // First the texts: everything below writes them.
    await loadTexts();
    setMode(null);
    showLayerNames();
    project = await api("api/project");
    $("folder").textContent = project.dir;
    showUpdateNotice();
    showSettings();
    $("has-map").hidden = !project.has_map;
    if (project.area) {
      await chooseArea(project.area, project.source_id, null, true);
      savedSignature = signature();
    }
    if (window.freemapperRun) window.freemapperRun.init(project);
  } catch (e) {
    setMessage(t("page-failed", { error: e.message }), "error");
  }
}

// Module scripts run before DOMContentLoaded, run.js after this one: by then both are loaded.
document.addEventListener("DOMContentLoaded", start);
