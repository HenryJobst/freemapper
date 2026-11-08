// freemapper area selection page: draw or pick the map area, see tiles and download, save it.
"use strict";

const TEXTS = {
  de: {
    heading: "Kartengebiet wählen",
    folder: "Projektordner",
    pick: "Ort suchen",
    queryPlaceholder: "z. B. Clara-Zetkin-Schule Strausberg",
    searchButton: "Suchen",
    searching: "Suche …",
    noResults: "Nichts gefunden.",
    pointOnly: "nur ein Punkt, keine Fläche",
    draw: "Selbst zeichnen",
    rectangle: "Rechteck",
    polygon: "Polygon",
    finishPolygon: "Polygon schließen",
    clear: "Löschen",
    hintRectangle: "Klicke zwei gegenüberliegende Ecken in die Karte.",
    hintPolygon:
      "Klicke die Ecken nacheinander. Zum Schließen die erste Ecke anklicken, doppelklicken oder „Polygon schließen“ wählen. Esc bricht ab, Rücktaste nimmt die letzte Ecke zurück.",
    hintIdle: "Wähle einen Ort aus der Suche oder zeichne das Gebiet selbst.",
    summary: "Gebiet",
    sourceOsm: "Aus OpenStreetMap: ",
    sourceDrawn: "Selbst gezeichnet",
    size: (w, h) => `Größe mit Puffer: ${w} × ${h} m`,
    tiles: (n, names) => `${n} Kachel${n === 1 ? "" : "n"}: ${names}`,
    download: (mb) => `Download: etwa ${mb} MB`,
    downloadNone: "Download: alles schon im Zwischenspeicher",
    downloadDetails: "Was geladen wird",
    cached: "schon geladen",
    tooLarge: (max) =>
      `Das Gebiet ist zu groß: höchstens ${max} m Seitenlänge (mit Puffer). Bitte kleiner wählen.`,
    outside:
      "Das Gebiet liegt außerhalb der unterstützten Länder (Brandenburg, Berlin, Sachsen, Sachsen-Anhalt, Hamburg, Hessen, Schleswig-Holstein, Mecklenburg-Vorpommern, Nordrhein-Westfalen, Niedersachsen); für andere Regionen hat freemapper noch keine Datenquellen.",
    settings: "Karte",
    kind: "Kartenart",
    schoolyard: "Schulhofkarte (1:1000)",
    sprint: "Sprintkarte (1:4000)",
    scale: "Maßstab 1:",
    title: "Titel",
    settingsFixed:
      "Das Projekt gibt es schon; Kartenart, Maßstab und Titel stehen in freemapper.toml.",
    save: "Speichern",
    saving: "Speichere …",
    saved: "Gespeichert in freemapper.toml.",
    zoneChanged:
      "Gespeichert. Das Gebiet liegt jetzt in einer anderen UTM-Zone; freemapper ruft seine Daten beim nächsten Lauf neu ab.",
    hasMap:
      "Die vorhandene Karte (.omap) bleibt, wie sie ist; freemapper erzeugt neue Vorlagen und Kandidaten für das neue Gebiet. „freemapper run --force“ erzeugt auch die Karte neu und behält die alte als .omap.bak.",
    done: "Fertig",
    closed: "Fertig. Du kannst dieses Fenster schließen.",
    next: (dir) => `Weiter: freemapper run "${dir}"`,
    backToAgent: "Sag deinem KI-Assistenten, dass das Gebiet gespeichert ist.",
    nothingSaved: "Es wurde nichts gespeichert.",
    failed: "Fehler: ",
    osm: "OpenStreetMap",
    aerial: "Luftbild Brandenburg/Berlin (LGB)",
    aerialSaxony: "Luftbild Sachsen (GeoSN)",
    aerialSaxonyAnhalt: "Luftbild Sachsen-Anhalt (LVermGeo)",
    aerialHamburg: "Luftbild Hamburg (LGV)",
    aerialHessen: "Luftbild Hessen (HVBG)",
    aerialSchleswigHolstein: "Luftbild Schleswig-Holstein (LVermGeo SH)",
    aerialMecklenburgVorpommern: "Luftbild Mecklenburg-Vorpommern (LAiV M-V)",
    aerialNordrheinWestfalen: "Luftbild Nordrhein-Westfalen (Geobasis NRW)",
    aerialNiedersachsen: "Luftbild Niedersachsen (LGLN)",
  },
  en: {
    heading: "Choose the map area",
    folder: "Project folder",
    pick: "Search a place",
    queryPlaceholder: "e.g. Clara-Zetkin-Schule Strausberg",
    searchButton: "Search",
    searching: "Searching …",
    noResults: "Nothing found.",
    pointOnly: "a point only, not an area",
    draw: "Draw it yourself",
    rectangle: "Rectangle",
    polygon: "Polygon",
    finishPolygon: "Close polygon",
    clear: "Clear",
    hintRectangle: "Click two opposite corners on the map.",
    hintPolygon:
      "Click the corners one after another. To close it, click the first corner, double-click or click \"Close polygon\". Esc cancels, Backspace removes the last corner.",
    hintIdle: "Pick a place from the search or draw the area yourself.",
    summary: "Area",
    sourceOsm: "From OpenStreetMap: ",
    sourceDrawn: "Drawn by hand",
    size: (w, h) => `Size with buffer: ${w} × ${h} m`,
    tiles: (n, names) => `${n} tile${n === 1 ? "" : "s"}: ${names}`,
    download: (mb) => `Download: about ${mb} MB`,
    downloadNone: "Download: everything is cached already",
    downloadDetails: "What is downloaded",
    cached: "cached",
    tooLarge: (max) =>
      `The area is too large: at most ${max} m per side (with buffer). Please choose a smaller one.`,
    outside:
      "The area lies outside the supported states (Brandenburg, Berlin, Saxony, Saxony-Anhalt, Hamburg, Hesse, Schleswig-Holstein, Mecklenburg-Vorpommern, North Rhine-Westphalia, Lower Saxony); freemapper has no data sources for other regions yet.",
    settings: "Map",
    kind: "Map kind",
    schoolyard: "Schoolyard map (1:1000)",
    sprint: "Sprint map (1:4000)",
    scale: "Scale 1:",
    title: "Title",
    settingsFixed:
      "The project exists already; map kind, scale and title are in freemapper.toml.",
    save: "Save",
    saving: "Saving …",
    saved: "Saved to freemapper.toml.",
    zoneChanged:
      "Saved. The area now lies in another UTM zone; freemapper fetches its data again on the next run.",
    hasMap:
      "The existing map (.omap) stays as it is; freemapper makes new templates and candidates for the new area. \"freemapper run --force\" makes the map anew too and keeps the old one as .omap.bak.",
    done: "Done",
    closed: "Done. You can close this window.",
    next: (dir) => `Next: freemapper run "${dir}"`,
    backToAgent: "Tell your AI assistant that the area is saved.",
    nothingSaved: "Nothing was saved.",
    failed: "Error: ",
    osm: "OpenStreetMap",
    aerial: "Aerial photo Brandenburg/Berlin (LGB)",
    aerialSaxony: "Aerial photo Saxony (GeoSN)",
    aerialSaxonyAnhalt: "Aerial photo Saxony-Anhalt (LVermGeo)",
    aerialHamburg: "Aerial photo Hamburg (LGV)",
    aerialHessen: "Aerial photo Hesse (HVBG)",
    aerialSchleswigHolstein: "Aerial photo Schleswig-Holstein (LVermGeo SH)",
    aerialMecklenburgVorpommern: "Aerial photo Mecklenburg-Vorpommern (LAiV M-V)",
    aerialNordrheinWestfalen: "Aerial photo North Rhine-Westphalia (Geobasis NRW)",
    aerialNiedersachsen: "Aerial photo Lower Saxony (LGLN)",
  },
};

const T = (navigator.language || "de").toLowerCase().startsWith("de") ? TEXTS.de : TEXTS.en;
document.documentElement.lang = T === TEXTS.de ? "de" : "en";
document.querySelectorAll("[data-text]").forEach((el) => {
  const text = T[el.dataset.text];
  if (typeof text === "string") el.textContent = text;
});
document.querySelectorAll("[data-placeholder]").forEach((el) => {
  el.placeholder = T[el.dataset.placeholder];
});

const $ = (id) => document.getElementById(id);

async function api(path, body) {
  const options = body === undefined
    ? {}
    : { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) };
  const response = await fetch(path, options);
  const answer = await response.json().catch(() => ({ error: response.statusText }));
  if (!response.ok) throw new Error(answer.error || response.statusText);
  return answer;
}

// --- Map -------------------------------------------------------------------------------------

const map = L.map("map", { doubleClickZoom: true }).setView([52.45, 13.4], 8);
const osm = L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
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
L.control
  .layers({
    [T.osm]: osm,
    [T.aerial]: aerial,
    [T.aerialSaxony]: aerialSaxony,
    [T.aerialSaxonyAnhalt]: aerialSaxonyAnhalt,
    [T.aerialHamburg]: aerialHamburg,
    [T.aerialHessen]: aerialHessen,
    [T.aerialSchleswigHolstein]: aerialSchleswigHolstein,
    [T.aerialMecklenburgVorpommern]: aerialMecklenburgVorpommern,
    [T.aerialNordrheinWestfalen]: aerialNordrheinWestfalen,
    [T.aerialNiedersachsen]: aerialNiedersachsen,
  })
  .addTo(map);
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
  try {
    const body = { area, source_id: sourceId };
    if (project && !project.exists) body.kind = $("kind").value;
    const answer = await api("api/preview", body);
    if (request !== previewRequest) return;
    preview = answer;
    showPreview();
    $("save").disabled = preview.too_large || preview.outside_coverage;
    setMessage("");
  } catch (e) {
    if (request !== previewRequest) return;
    preview = null;
    showPreview();
    setMessage(T.failed + e.message, "error");
  }
}

function showPreview() {
  tileLayer.clearLayers();
  $("summary").hidden = !area;
  if (!area) return;
  $("source").textContent = sourceId ? T.sourceOsm + (sourceName || sourceId) : T.sourceDrawn;
  if (!preview) {
    ["size", "tiles", "download"].forEach((id) => ($(id).textContent = ""));
    $("items").replaceChildren();
    $("too-large").hidden = true;
    return;
  }
  $("size").textContent = T.size(Math.round(preview.width_m), Math.round(preview.height_m));
  const unusable = preview.too_large || preview.outside_coverage;
  $("too-large").hidden = !unusable;
  $("too-large").textContent = preview.outside_coverage
    ? T.outside
    : T.tooLarge(Math.round(preview.max_side_m));
  const names = preview.tiles.map((t) => t.name);
  $("tiles").textContent = unusable ? "" : T.tiles(names.length, names.join(", "));
  const mb = preview.estimate.total_megabytes;
  $("download").textContent = unusable
    ? ""
    : mb > 0 ? T.download(Math.ceil(mb)) : T.downloadNone;
  $("items").replaceChildren(
    ...preview.estimate.items.map((item) => {
      const li = document.createElement("li");
      li.textContent = `${item.what}: ${item.megabytes} MB${item.cached ? ` (${T.cached})` : ""}`;
      return li;
    }),
  );
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

// --- Search ----------------------------------------------------------------------------------

$("search").addEventListener("submit", async (event) => {
  event.preventDefault();
  const query = $("query").value.trim();
  if (!query) return;
  const list = $("results");
  list.replaceChildren(Object.assign(document.createElement("li"), { textContent: T.searching }));
  try {
    const places = await api(`api/places?q=${encodeURIComponent(query)}`);
    if (places.length === 0) {
      list.replaceChildren(Object.assign(document.createElement("li"), { textContent: T.noResults }));
      return;
    }
    list.replaceChildren(
      ...places.map((place) => {
        const li = document.createElement("li");
        li.textContent = place.display_name;
        if (!place.has_area) {
          li.classList.add("point");
          li.title = T.pointOnly;
          li.textContent += ` (${T.pointOnly})`;
          return li;
        }
        li.addEventListener("click", async () => {
          setMessage(T.searching);
          try {
            const found = await api(`api/place?osm_id=${encodeURIComponent(place.osm_id)}`);
            await chooseArea(found.area, found.source_id, place.name || place.display_name, true);
          } catch (e) {
            setMessage(T.failed + e.message, "error");
          }
        });
        return li;
      }),
    );
  } catch (e) {
    list.replaceChildren();
    setMessage(T.failed + e.message, "error");
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
  $("hint").textContent =
    mode === "rectangle" ? T.hintRectangle : mode === "polygon" ? T.hintPolygon : T.hintIdle;
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
  $("save").disabled = true;
  setMessage("");
});

// --- Settings, save, done --------------------------------------------------------------------

$("kind").addEventListener("change", () => {
  $("scale").value = $("kind").value === "sprint" ? 4000 : 1000;
  // The buffer depends on the map kind.
  updatePreview();
});

$("save").addEventListener("click", async () => {
  if (!area) return;
  setMessage(T.saving);
  $("save").disabled = true;
  try {
    const body = { area, source_id: sourceId };
    if (!project.exists) {
      body.kind = $("kind").value;
      body.scale = Number($("scale").value) || undefined;
      body.title = $("title").value.trim() || undefined;
    }
    const answer = await api("api/save", body);
    savedHere = true;
    setMessage(answer.zone_changed ? T.zoneChanged : T.saved, "ok");
    $("has-map").hidden = !answer.has_map;
    project = await api("api/project");
    showSettings();
  } catch (e) {
    setMessage(T.failed + e.message, "error");
  } finally {
    $("save").disabled = !area || !preview || preview.too_large || preview.outside_coverage;
  }
});

$("done").addEventListener("click", async () => {
  try {
    await api("api/done", {});
  } catch (e) {
    // The server may close before it answers; the page is finished either way.
  }
  $("next").textContent = !savedHere
    ? T.nothingSaved
    : project && project.from_agent
      ? T.backToAgent
      : T.next(project ? project.dir : "");
  $("closed").hidden = false;
});

function showSettings() {
  $("kind").value = project.kind;
  $("scale").value = project.scale;
  $("title").value = project.title || "";
  for (const id of ["kind", "scale", "title"]) $(id).disabled = project.exists;
  $("settings-fixed").hidden = !project.exists;
}

// --- Start -----------------------------------------------------------------------------------

(async () => {
  setMode(null);
  try {
    project = await api("api/project");
    $("folder").textContent = project.dir;
    showSettings();
    $("has-map").hidden = !project.has_map;
    if (project.area) {
      await chooseArea(project.area, project.source_id, null, true);
    }
  } catch (e) {
    setMessage(T.failed + e.message, "error");
  }
})();
