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
/** The regions freemapper supports, from `api/adapters`: `[{ id, names, preview }]`. */
let adapters = [];

/** `text` as HTML that shows it as it is: Leaflet takes layer names and credits as HTML. */
function escapeHtml(text) {
  const span = document.createElement("span");
  span.textContent = text;
  return span.innerHTML;
}

function isText(value) {
  return typeof value === "string" && value !== "";
}

/**
 * The aerial photo of a region as the page can draw it, or null: every field a text, the
 * parameters plain values, and none of them an option of Leaflet (they are query parameters of
 * the WMS request and nothing else).
 */
function usablePreview(preview) {
  if (!preview || typeof preview !== "object") return null;
  const { wms, layer, format, attribution, agency } = preview;
  if (![wms, layer, format, attribution, agency].every(isText)) return null;
  const params = {};
  const given = preview.params && typeof preview.params === "object" ? preview.params : {};
  for (const [name, value] of Object.entries(given)) {
    const plain = typeof value === "string" || typeof value === "number";
    if (plain && !(name in L.TileLayer.WMS.prototype.options)) params[name] = String(value);
  }
  return { wms, layer, format, attribution, agency, params };
}

/**
 * The regions of `api/adapters` the page can use: each with a name, and with its aerial photo
 * when that is complete. When the answer fails the list is empty and `problem` says why; the
 * page then works without aerial photos.
 */
async function loadAdapters() {
  try {
    const list = await api("api/adapters");
    const regions = (Array.isArray(list) ? list : [])
      .filter((region) => region && region.names && isText(region.names.en))
      .map((region) => ({ ...region, preview: usablePreview(region.preview) }));
    return { regions, problem: null };
  } catch (e) {
    return { regions: [], problem: e.message };
  }
}

/** The names of `regions` in the page language, joined by `separator`. */
function regionNames(regions, separator) {
  return regions
    .map((region) => (isText(region.names[language()]) ? region.names[language()] : region.names.en))
    .join(separator);
}

/**
 * The aerial photos of the regions: one WMS layer per service, named after the regions that
 * share it (Brandenburg and Berlin share the one of LGB). Regions share a layer only when
 * their previews are the same in every field.
 */
function aerialLayers() {
  const layers = new Map();
  for (const region of adapters) {
    const preview = region.preview;
    if (!preview) continue;
    const key = JSON.stringify([
      preview.wms,
      preview.layer,
      preview.format,
      preview.attribution,
      preview.agency,
      Object.entries(preview.params).sort(),
    ]);
    if (!layers.has(key)) layers.set(key, { preview, regions: [] });
    layers.get(key).regions.push(region);
  }
  const year = String(new Date().getFullYear());
  const named = {};
  for (const { preview, regions } of layers.values()) {
    const name = t("page-layer-aerial-of", {
      state: regionNames(regions, "/"),
      agency: preview.agency,
    });
    named[escapeHtml(name)] = L.tileLayer.wms(preview.wms, {
      ...preview.params,
      layers: preview.layer,
      format: preview.format,
      maxZoom: 20,
      attribution: escapeHtml(preview.attribution.replaceAll("{year}", year)),
    });
  }
  return named;
}

/** The layer control, with its names in the page language: built once the texts are loaded. */
function showLayerNames() {
  L.control.layers({ [escapeHtml(t("page-layer-osm"))]: osm, ...aerialLayers() }).addTo(map);
}
map.attributionControl.addAttribution(
  '<a href="https://github.com/HenryJobst/freemapper">freemapper</a>',
);

const tileLayer = L.layerGroup().addTo(map);
const areaLayer = L.layerGroup().addTo(map);
const frameLayer = L.layerGroup().addTo(map);
const drawLayer = L.layerGroup().addTo(map);

// --- State -----------------------------------------------------------------------------------

let project = null;
// The chosen area: GeoJSON polygon geometry and the OSM object it came from (or null).
let area = null;
// The area exactly as it came in (from the project, the search or the drawing). A save sends this
// back, not `area`: the saved project's fingerprint (the preview's "is it still current") would
// otherwise depend on how this page writes the shape.
let areaInput = null;
let sourceId = null;
let sourceName = null;
let preview = null;
// Answers to older preview requests are dropped.
let previewRequest = 0;
let savedHere = false;
/** The contour interval chosen on the page in metres; `null` is the automatic choice. */
let contourInterval = null;
/** The steep slope chosen on the page in degrees; `null` is the default (about 27°). */
let steepSlope = null;
/** The face slope chosen on the page in degrees; `null` is the default (70°). */
let faceSlope = null;
// True while a run goes on: the area, the settings and "Save" are locked.
let busy = false;
// The area and source as last saved or loaded; a different one needs a save before a run.
let savedSignature = null;
// Set when another window or program wrote freemapper.toml since this page read it. Only a click
// on "Save" clears it; until then a run does not save on its own (ensureSaved).
let conflict = false;

function signature() {
  return JSON.stringify({ area, sourceId, content: contentKey(content), contour: contourInterval, steep: steepSlope, face: faceSlope });
}

/** The signature of the area that `project` holds, as `signature()` gives it once shown. */
function savedSignatureOf(project) {
  const saved = project && project.area ? geometryOf(project.area) : null;
  return JSON.stringify({
    area: saved,
    sourceId: saved ? project.source_id : null,
    content: contentKey(contentOf(project)),
    contour: project ? (project.contour_interval_m ?? null) : null,
    steep: project ? (project.steep_slope_deg ?? null) : null,
    face: project ? (project.face_slope_deg ?? null) : null,
  });
}

/** Tells the run panel that the area, the preview or the project changed. */
function notifyRun() {
  if (window.freemapperRun) window.freemapperRun.refresh();
}

/** Tells the preview that the content settings on the page changed. */
function notifyContent() {
  if (window.freemapperRun) window.freemapperRun.contentChanged();
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
  $("contour").disabled = busy;
  $("steep").disabled = busy;
  $("face").disabled = busy;
  $("paper-keep").disabled = busy || !$("paper-format").value;
  for (const button of $("paper-frames").querySelectorAll("button")) button.disabled = busy;
  for (const control of $("content-section").querySelectorAll("input, select")) {
    control.disabled = busy;
  }
}

/** "Error: …" in the page language; the bare message while the texts are missing. */
function failedText(message) {
  const text = t("page-failed", { error: message });
  return text === "page-failed" ? message : text;
}

/** What to say about an area outside every region: with the regions by name when known. */
function outsideText() {
  return adapters.length
    ? t("page-outside", { states: regionNames(adapters, ", ") })
    : t("page-outside-unknown");
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
  areaInput = geojson;
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
    setMessage(failedText(e.message), "error");
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
    ? outsideText()
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
            setMessage(failedText(e.message), "error");
          }
        });
        return li;
      }),
    );
  } catch (e) {
    list.replaceChildren();
    setMessage(failedText(e.message), "error");
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
  // While a drawing tool is on, the paper frame lets every click through to the map.
  showFrames();
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
  areaInput = null;
  sourceId = null;
  preview = null;
  areaLayer.clearLayers();
  showPreview();
  updateControls();
  setMessage("");
  notifyRun();
});

// --- Paper frames ----------------------------------------------------------------------------

/** The kept paper frames, `[{ format, orientation, lon, lat }]`, as the save takes them. */
let paperFrames = [];
// The kept frames as last loaded or saved: tells whether the person changed them since.
let loadedFrames = [];
/** The centre (`L.LatLng`) of the frame being placed; null while no format is chosen. */
let frameCentre = null;
/** The frame being placed, on the map while a format is chosen. */
let framePolygon = null;
/** The drag of that frame: the pointer and the frame centre where it began, in layer pixels. */
let frameDrag = null;
// Saves of the kept frames run one after the other: each needs the revision the one before left.
let framesSaving = Promise.resolve();

const FRAME_NAMES = {
  landscape: "page-paper-name-landscape",
  portrait: "page-paper-name-portrait",
};
const PLACED_FRAME = { color: "#7a3ec8", weight: 3, fillOpacity: 0.06, className: "paper-frame" };
const KEPT_FRAME = { color: "#9a8ab4", weight: 1.5, fill: false, interactive: false };
const EARTH_RADIUS_M = 6378137;

/**
 * The corners `[[lat, lon] ×4]` (south-west, south-east, north-east, north-west) of a north-up
 * rectangle of `widthM` × `heightM` on the ground around `lat`/`lon`, on the page's web-mercator
 * map. That map stretches distances by 1/cos(lat), so the ground size is divided by cos(lat).
 * Only for the screen: the export computes the exact frame in the map's own CRS.
 */
function frameCorners(lat, lon, widthM, heightM) {
  const rad = Math.PI / 180;
  const x = EARTH_RADIUS_M * lon * rad;
  const y = EARTH_RADIUS_M * Math.log(Math.tan(Math.PI / 4 + (lat * rad) / 2));
  const dx = widthM / 2 / Math.cos(lat * rad);
  const dy = heightM / 2 / Math.cos(lat * rad);
  const corner = (px, py) => [
    (2 * Math.atan(Math.exp(py / EARTH_RADIUS_M)) - Math.PI / 2) / rad,
    px / EARTH_RADIUS_M / rad,
  ];
  return [corner(x - dx, y - dy), corner(x + dx, y - dy), corner(x + dx, y + dy), corner(x - dx, y + dy)];
}

/** The corners of `frame` at the scale chosen on the page; null for a format the server lacks. */
function cornersOf(frame) {
  const paper = ((project && project.paper_formats) || []).find((p) => p.name === frame.format);
  const scale = Number($("scale").value);
  if (!paper || !scale) return null;
  const [width, height] = frame.orientation === "landscape"
    ? [paper.height_mm, paper.width_mm]
    : [paper.width_mm, paper.height_mm];
  return frameCorners(frame.lat, frame.lon, (width * scale) / 1000, (height * scale) / 1000);
}

/** The frame being placed as the save takes it, or null while no format is chosen. */
function placedFrame() {
  const format = $("paper-format").value;
  if (!format || !frameCentre) return null;
  const orientation = $("paper-orientation").value;
  return { format, orientation, lon: frameCentre.lng, lat: frameCentre.lat };
}

/** The frames that `project` holds, as the page keeps them. */
function framesOf(project) {
  const stored = (project && Array.isArray(project.paper_frames)) ? project.paper_frames : [];
  return stored.map(({ format, orientation, lon, lat }) => ({ format, orientation, lon, lat }));
}

/** The frames in a fixed form, to compare two lists. */
function framesKey(frames) {
  return JSON.stringify(frames.map((f) => [f.format, f.orientation, f.lon, f.lat]));
}

/**
 * Draws the kept frames and, with a format chosen, the frame being placed: at first on the area,
 * else on the middle of the map. Nothing is drawn without either.
 */
function showFrames() {
  frameLayer.clearLayers();
  framePolygon = null;
  for (const frame of paperFrames) {
    const corners = cornersOf(frame);
    if (corners) L.polygon(corners, KEPT_FRAME).addTo(frameLayer);
  }
  if (!$("paper-format").value) return;
  const fresh = !frameCentre;
  if (fresh) frameCentre = area ? L.geoJSON(area).getBounds().getCenter() : map.getCenter();
  const corners = cornersOf(placedFrame());
  if (!corners) return;
  framePolygon = L.polygon(corners, { ...PLACED_FRAME, interactive: !mode }).addTo(frameLayer);
  if (!mode) framePolygon.getElement().addEventListener("pointerdown", startFrameDrag);
  // A new frame larger than the view would take every drag: zoom out until there is room around it.
  const bounds = framePolygon.getBounds();
  if (fresh && !map.getBounds().contains(bounds)) map.fitBounds(bounds, { padding: [40, 40] });
}

// The drag goes without a plugin: the map stops panning while the frame moves with the pointer.
function startFrameDrag(event) {
  if (mode || !event.isPrimary || event.button !== 0) return;
  event.preventDefault();
  event.stopPropagation();
  map.dragging.disable();
  frameDrag = {
    pointer: map.mouseEventToLayerPoint(event),
    centre: map.latLngToLayerPoint(frameCentre),
  };
  document.addEventListener("pointermove", moveFrame);
  document.addEventListener("pointerup", endFrameDrag);
  document.addEventListener("pointercancel", endFrameDrag);
}

function moveFrame(event) {
  if (!frameDrag || !framePolygon) return;
  const moved = map.mouseEventToLayerPoint(event).subtract(frameDrag.pointer);
  frameCentre = map.layerPointToLatLng(frameDrag.centre.add(moved));
  const corners = cornersOf(placedFrame());
  if (corners) framePolygon.setLatLngs(corners);
}

function endFrameDrag() {
  document.removeEventListener("pointermove", moveFrame);
  document.removeEventListener("pointerup", endFrameDrag);
  document.removeEventListener("pointercancel", endFrameDrag);
  frameDrag = null;
  map.dragging.enable();
}

/** The list of the kept frames, each with its name and a delete button. */
function showFrameList() {
  $("paper-frames").replaceChildren(
    ...paperFrames.map((frame, index) => {
      const li = document.createElement("li");
      const name = document.createElement("span");
      name.textContent = t(FRAME_NAMES[frame.orientation], { format: frame.format });
      const remove = document.createElement("button");
      remove.type = "button";
      remove.textContent = t("page-paper-delete");
      remove.disabled = busy;
      remove.addEventListener("click", () => {
        paperFrames.splice(index, 1);
        framesChanged();
      });
      li.append(name, remove);
      return li;
    }),
  );
}

/** The kept frames changed: an existing project saves them at once and alone, a new one with its area. */
function framesChanged() {
  showFrames();
  showFrameList();
  if (project && project.exists) framesSaving = framesSaving.then(() => saveArea(false, true));
}

/** Fills the format list from the server's table (`paper_formats`), with "none" first. */
function fillPaperFormats() {
  const formats = (project && project.paper_formats) || [];
  $("paper-format").replaceChildren(
    new Option(t("page-paper-none"), ""),
    ...formats.map((paper) => new Option(paper.name, paper.name)),
  );
}

$("paper-format").addEventListener("change", () => {
  if (!$("paper-format").value) frameCentre = null;
  showFrames();
  updateControls();
});
$("paper-orientation").addEventListener("change", showFrames);
$("scale").addEventListener("change", showFrames);
$("paper-keep").addEventListener("click", () => {
  const frame = placedFrame();
  if (!frame || busy) return;
  paperFrames.push(frame);
  framesChanged();
});

// --- Settings, save, done --------------------------------------------------------------------

// Fills the scale list for `kind` from the server's table (`allowed_scales`, ADR 0085) and selects
// the kind's default (`default_scales`); a stored scale outside the table stays selectable.
function fillScales(kind, current) {
  const table = (project && project.allowed_scales) || {};
  const defaults = (project && project.default_scales) || {};
  const list = [...(table[kind] || (current ? [current] : []))];
  if (current && !list.includes(current)) list.push(current);
  $("scale").replaceChildren(...list.map((s) => new Option(String(s), String(s))));
  $("scale").value = String(current || defaults[kind] || list[0]);
}

// The contour intervals offered on the page (ADR 0116); a stored other one stays selectable.
const CONTOUR_INTERVALS = [1, 2, 2.5, 5];

/** Fills the contour interval list and selects `current` (`null`: automatic). */
function fillContours(current) {
  const list = [...CONTOUR_INTERVALS];
  if (current != null && !list.includes(current)) list.push(current);
  const metres = (m) => `${m.toLocaleString(language())} m`;
  $("contour").replaceChildren(
    new Option(t("page-contour-auto"), "auto"),
    ...list.map((m) => new Option(metres(m), String(m))),
  );
  $("contour").value = current == null ? "auto" : String(current);
  contourInterval = current ?? null;
}

$("contour").addEventListener("change", () => {
  contourInterval = $("contour").value === "auto" ? null : Number($("contour").value);
  notifyRun();
});

// The steep slopes offered on the page (ADR 0198); a stored other one stays selectable.
const STEEP_SLOPES = [45, 55, 65];

/** Fills the steep slope list and selects `current` (`null`: the default). */
function fillSteepSlopes(current) {
  const list = [...STEEP_SLOPES];
  if (current != null && !list.includes(current)) list.push(current);
  list.sort((a, b) => a - b);
  const degrees = (d) => `${d.toLocaleString(language())}°`;
  $("steep").replaceChildren(
    new Option(t("page-steep-slope-auto"), "auto"),
    ...list.map((d) => new Option(degrees(d), String(d))),
  );
  $("steep").value = current == null ? "auto" : String(current);
  steepSlope = current ?? null;
}

$("steep").addEventListener("change", () => {
  steepSlope = $("steep").value === "auto" ? null : Number($("steep").value);
  notifyRun();
});

// The face slopes offered on the page besides the default of 70° (ADR 0200); a stored other
// one stays selectable.
const FACE_SLOPES = [65, 75];

/** Fills the face slope list and selects `current` (`null`: the default). */
function fillFaceSlopes(current) {
  const list = [...FACE_SLOPES];
  if (current != null && !list.includes(current)) list.push(current);
  list.sort((a, b) => a - b);
  const degrees = (d) => `${d.toLocaleString(language())}°`;
  $("face").replaceChildren(
    new Option(t("page-face-slope-auto"), "auto"),
    ...list.map((d) => new Option(degrees(d), String(d))),
  );
  $("face").value = current == null ? "auto" : String(current);
  faceSlope = current ?? null;
}

$("face").addEventListener("change", () => {
  faceSlope = $("face").value === "auto" ? null : Number($("face").value);
  notifyRun();
});

$("kind").addEventListener("change", () => {
  fillScales($("kind").value);
  showFrames();
  // The buffer depends on the map kind.
  updatePreview();
  notifyRun();
});

$("save").addEventListener("click", () => saveArea(true));

/**
 * freemapper.toml holds another state than this page saved or read: show `text` in the banner,
 * take the saved area as the saved one and wait for an explicit "Save". The content settings follow
 * the file unless the person changed them since they were loaded.
 */
function showConflict(text) {
  conflict = true;
  setMessage("");
  showBanner(text);
  savedSignature = savedSignatureOf(project);
  if (contentKey(content) === contentKey(loadedContent)) {
    // The person did not touch the section: take what the file says now, so that a "Save" does not
    // overwrite a choice that the command line or an assistant wrote.
    content = contentOf(project);
    loadedContent = contentOf(project);
    showContent();
    notifyContent();
  } else {
    // The person changed it and sees it: their choice stays; open the section so they see what
    // "Save" will write.
    $("content-section").open = true;
  }
  // The kept frames likewise: the file's, unless a change of the person's here is not saved yet.
  if (framesKey(paperFrames) === framesKey(loadedFrames)) {
    paperFrames = framesOf(project);
    loadedFrames = framesOf(project);
    showFrames();
    showFrameList();
  }
}

/**
 * Saves the area (and, for a new project, kind, scale and title); true when it worked. `explicit`
 * is a click on "Save", the only save that clears a conflict. `framesOnly` saves the kept paper
 * frames of an existing project alone: area and settings stay as the file has them.
 */
async function saveArea(explicit = false, framesOnly = false) {
  if (busy || (!framesOnly && !area)) return false;
  setMessage(t("page-saving"));
  clearBanner();
  $("save").disabled = true;
  try {
    const body = framesOnly ? {} : { area: areaInput || area, source_id: sourceId };
    if (!framesOnly) {
      // Only a choice that differs from the file; without one the stored settings stay as they are.
      if (contentKey(content) !== contentKey(contentOf(project))) body.content = contentBody(content);
      if (contourInterval !== (project.contour_interval_m ?? null)) body.contour_interval_m = contourInterval;
      if (steepSlope !== (project.steep_slope_deg ?? null)) body.steep_slope_deg = steepSlope;
      if (faceSlope !== (project.face_slope_deg ?? null)) body.face_slope_deg = faceSlope;
      if (!project.exists) {
        body.kind = $("kind").value;
        body.scale = Number($("scale").value) || undefined;
        body.title = $("title").value.trim() || undefined;
      }
    }
    // The kept frames go with a save when they differ from the file (a new project: with its area).
    if (framesOnly || framesKey(paperFrames) !== framesKey(framesOf(project))) {
      body.paper_frames = paperFrames.map((frame) => ({ ...frame }));
    }
    // The revision of freemapper.toml as this page read it; none before the file exists.
    const answer = await api("api/save", body, project.revision);
    savedHere = true;
    loadedFrames = body.paper_frames || framesOf(project);
    if (!framesOnly) {
      savedSignature = signature();
      loadedContent = contentOf({ content_options: project.content_options, content: contentBody(content) });
      project.contour_interval_m = contourInterval;
      project.steep_slope_deg = steepSlope;
      project.face_slope_deg = faceSlope;
    }
    setMessage(
      t(
        framesOnly
          ? "page-paper-saved"
          : answer.zone_changed
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
    if (answer.revision !== project.revision) {
      // Another window or program wrote the file between this save and the reload.
      showConflict(t("page-save-stale"));
      return false;
    }
    if (explicit) conflict = false;
    return true;
  } catch (e) {
    if (e.status === 409 && e.body && typeof e.body.revision === "string") {
      // Another window or program wrote freemapper.toml since this page read it (core ADR 0030):
      // nothing was saved. The page takes the saved state and keeps the area it shows.
      try {
        await reloadProject();
      } catch (reload) {
        // The banner stays; the next save asks the server again.
      }
      showConflict(e.message);
    } else {
      setMessage(failedText(e.message), "error");
    }
    return false;
  } finally {
    updateControls();
    notifyRun();
    if (window.freemapperRun) window.freemapperRun.projectSaved();
  }
}

/** True when the shown area and content settings are the ones in freemapper.toml. */
function isSaved() {
  return !!project && project.exists && signature() === savedSignature;
}

/** True when the shown area is the one in freemapper.toml, whatever the content settings. */
function areaIsSaved() {
  if (!project || !project.exists || savedSignature === null) return false;
  const saved = JSON.parse(savedSignature);
  return JSON.stringify({ area, sourceId }) === JSON.stringify({ area: saved.area, sourceId: saved.sourceId });
}

/**
 * Saves first when the area or the content settings changed; true when the project on disk matches the page. After a
 * conflict it saves nothing and answers false: the person decides with "Save".
 */
async function ensureSaved() {
  if (conflict) return false;
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
  fillContours(project.contour_interval_m ?? null);
  fillSteepSlopes(project.steep_slope_deg ?? null);
  fillFaceSlopes(project.face_slope_deg ?? null);
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
  areaIsSaved,
  ensureSaved,
  setBusy,
  reloadProject,
  // For the map preview: the content settings as the server takes them, and whether two of them
  // (this page's choice, a body that the server sent) are the same choice.
  contentBody: () => contentBody(content),
  sameContent: (body) => contentKey(content) === contentKey(contentOf({
    content_options: project.content_options,
    content: body,
  })),
  contentIsSaved: () => contentKey(content) === contentKey(contentOf(project)),
};

// --- Content profiles (ADR 0100, 0101) --------------------------------------------------------

// The texts by literal id, so that the tests find every one of them in this script. A group, a
// profile or a place that the server lists and this table lacks shows under its own name; the
// Rust test `content_names_cover_the_library` fails first.
const CONTENT_NAMES = {
  profiles: {
    safe: ["page-content-profile-safe", "page-content-profile-safe-hint"],
    balanced: ["page-content-profile-balanced", "page-content-profile-balanced-hint"],
    full: ["page-content-profile-full", "page-content-profile-full-hint"],
  },
  groups: {
    walls: "page-content-group-walls",
    fences: "page-content-group-fences",
    hedges: "page-content-group-hedges",
    slopes: "page-content-group-slopes",
    stairs: "page-content-group-stairs",
    trees: "page-content-group-trees",
    "water-features": "page-content-group-water-features",
    monuments: "page-content-group-monuments",
    furniture: "page-content-group-furniture",
    passages: "page-content-group-passages",
    contours: "page-content-group-contours",
    landforms: "page-content-group-landforms",
    vegetation: "page-content-group-vegetation",
    "out-of-bounds": "page-content-group-out-of-bounds",
    paths: "page-content-group-paths",
    streets: "page-content-group-streets",
  },
  places: {
    map: "page-content-place-map",
    candidates: "page-content-place-candidates",
    off: "page-content-place-off",
  },
  profileDefault: "page-content-place-profile",
};

// The choice on the page: a profile and the groups that differ from it ({ group: place }).
let content = { profile: "balanced", groups: {} };
// The choice as last loaded from the file or saved: tells whether the person changed it since.
let loadedContent = { profile: "balanced", groups: {} };

/** The choice that `project` holds; `balanced` without exceptions when its file has none. */
function contentOf(project) {
  const stored = project && project.content;
  const groups = {};
  if (stored) {
    for (const name of project.content_options.groups) if (stored[name]) groups[name] = stored[name];
  }
  return { profile: (stored && stored.profile) || "balanced", groups };
}

/** The choice in a fixed order, to compare two of them. */
function contentKey(choice) {
  return JSON.stringify([choice.profile, Object.entries(choice.groups).sort()]);
}

/** The choice as `api/save` takes it: flat, with the groups by their names in freemapper.toml. */
function contentBody(choice) {
  return { profile: choice.profile, ...choice.groups };
}

/** Builds the controls once from the lists of the server, in the library's order. */
function buildContent(options) {
  const profiles = $("content-profile");
  for (const name of options.profiles) {
    const [nameId, hintId] = CONTENT_NAMES.profiles[name] || [null, null];
    const label = document.createElement("label");
    label.className = "option";
    const radio = document.createElement("input");
    radio.type = "radio";
    radio.name = "content-profile";
    radio.value = name;
    radio.id = `content-profile-${name}`;
    radio.addEventListener("change", () => {
      content.profile = name;
      notifyRun();
      notifyContent();
    });
    const text = document.createElement("span");
    text.textContent = nameId ? t(nameId) : name;
    label.append(radio, text);
    const hint = document.createElement("p");
    hint.className = "hint option-help";
    hint.id = `content-profile-${name}-hint`;
    hint.textContent = hintId ? t(hintId) : "";
    radio.setAttribute("aria-describedby", hint.id);
    profiles.append(label, hint);
  }
  const groups = $("content-groups");
  for (const group of options.groups) {
    const label = document.createElement("label");
    label.className = "group-row";
    const text = document.createElement("span");
    text.textContent = CONTENT_NAMES.groups[group] ? t(CONTENT_NAMES.groups[group]) : group;
    const select = document.createElement("select");
    select.id = `content-group-${group}`;
    select.append(new Option(t(CONTENT_NAMES.profileDefault), ""));
    for (const place of options.places) {
      select.append(new Option(CONTENT_NAMES.places[place] ? t(CONTENT_NAMES.places[place]) : place, place));
    }
    select.addEventListener("change", () => {
      if (select.value) content.groups[group] = select.value;
      else delete content.groups[group];
      notifyRun();
      notifyContent();
    });
    label.append(text, select);
    groups.append(label);
  }
}

/** Shows `content` in the controls. */
function showContent() {
  for (const radio of $("content-profile").querySelectorAll("input")) {
    radio.checked = radio.value === content.profile;
  }
  for (const select of $("content-groups").querySelectorAll("select")) {
    select.value = content.groups[select.id.slice("content-group-".length)] || "";
  }
}

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
    // The page works without the list of the regions: the map keeps OpenStreetMap.
    const listed = await loadAdapters();
    adapters = listed.regions;
    showLayerNames();
    project = await api("api/project");
    $("folder").textContent = project.dir;
    showUpdateNotice();
    showSettings();
    fillPaperFormats();
    paperFrames = framesOf(project);
    loadedFrames = framesOf(project);
    showFrameList();
    buildContent(project.content_options);
    content = contentOf(project);
    loadedContent = contentOf(project);
    showContent();
    $("has-map").hidden = !project.has_map;
    if (project.area) {
      await chooseArea(project.area, project.source_id, null, true);
      savedSignature = signature();
    }
    // After the area: a kept frame is drawn where it lies, and the map is already there.
    showFrames();
    if (window.freemapperRun) window.freemapperRun.init(project);
    if (listed.problem) {
      setMessage(t("page-adapters-failed", { detail: listed.problem }), "error");
    }
  } catch (e) {
    setMessage(failedText(e.message), "error");
  }
}

// Module scripts run before DOMContentLoaded, run.js after this one: by then both are loaded.
document.addEventListener("DOMContentLoaded", start);
