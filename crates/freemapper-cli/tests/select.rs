//! The area selection page served by `freemapper select`, spoken to over HTTP as the page does.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

mod isolation;
use isolation::isolated;

use serde_json::{Value, json};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/clara-zetkin-schule.geojson"
);

/// A running `freemapper select` and the URL it printed.
struct Selector {
    child: Child,
    url: String,
    agent: ureq::Agent,
}

impl Selector {
    fn start(dir: &Path, cache: &Path, extra: &[&str]) -> Self {
        Self::start_with_env(dir, cache, extra, &[])
    }

    fn start_with_env(dir: &Path, cache: &Path, extra: &[&str], env: &[(&str, &str)]) -> Self {
        let mut child = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")))
            .env("FREEMAPPER_LANG", "en")
            .env_remove("CI")
            .envs(env.iter().copied())
            .arg("select")
            .arg(dir)
            .args(["--no-browser", "--cache-dir", cache.to_str().unwrap()])
            .args(extra)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        // The URL is the first line; give up after ten seconds instead of hanging.
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut line = String::new();
            let _ = BufReader::new(stdout).read_line(&mut line);
            let _ = sender.send(line);
        });
        let line = match receiver.recv_timeout(Duration::from_secs(10)) {
            Ok(line) => line,
            Err(_) => {
                let _ = child.kill();
                panic!("freemapper select printed no URL");
            }
        };
        let url = line.trim().to_string();
        assert!(
            url.starts_with("http://127.0.0.1:") && url.ends_with('/'),
            "{url}"
        );
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();
        Self { child, url, agent }
    }

    fn get(&self, path: &str) -> (u16, String) {
        let mut response = self
            .agent
            .get(&format!("{}{path}", self.url))
            .call()
            .unwrap();
        let status = response.status().as_u16();
        (status, response.body_mut().read_to_string().unwrap())
    }

    fn post(&self, path: &str, body: Value) -> (u16, Value) {
        self.post_with_revision(path, body, None)
    }

    /// A POST as the page sends it, with the revision header when `revision` is given.
    fn post_with_revision(&self, path: &str, body: Value, revision: Option<&str>) -> (u16, Value) {
        let mut request = self
            .agent
            .post(&format!("{}{path}", self.url))
            .header("Content-Type", "application/json")
            // The page sends its own origin; the server refuses a POST without it.
            .header(
                "Origin",
                self.url.split('/').take(3).collect::<Vec<_>>().join("/"),
            );
        if let Some(revision) = revision {
            request = request.header("X-Freemapper-Revision", revision);
        }
        let mut response = request.send(body.to_string()).unwrap();
        let status = response.status().as_u16();
        let text = response.body_mut().read_to_string().unwrap();
        (
            status,
            serde_json::from_str(&text).unwrap_or(Value::String(text)),
        )
    }

    /// The project as the page reads it.
    fn project(&self) -> Value {
        let (status, body) = self.get("api/project");
        assert_eq!(status, 200, "{body}");
        serde_json::from_str(&body).unwrap()
    }

    /// A save as the page makes it: with the revision of freemapper.toml it read last.
    fn save(&self, body: Value) -> (u16, Value) {
        let revision = self.project()["revision"].as_str().map(str::to_owned);
        self.post_with_revision("api/save", body, revision.as_deref())
    }

    /// `127.0.0.1:<port>` and the token of the URL.
    fn address_and_token(&self) -> (String, String) {
        let rest = self.url.strip_prefix("http://").unwrap();
        let (address, path) = rest.split_once('/').unwrap();
        (address.to_string(), path.trim_end_matches('/').to_string())
    }

    /// Waits up to five seconds for the process to end; its exit status.
    fn wait(&mut self) -> Option<std::process::ExitStatus> {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Some(status) = self.child.try_wait().unwrap() {
                return Some(status);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        None
    }
}

impl Drop for Selector {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn school_area() -> Value {
    serde_json::from_str(&std::fs::read_to_string(FIXTURE).unwrap()).unwrap()
}

/// Sends one raw request and returns the status code.
fn raw_status(address: &str, request: &str) -> u16 {
    raw_response(address, request)
        .split(' ')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap()
}

/// Sends one raw request and returns the whole response.
fn raw_response(address: &str, request: &str) -> String {
    let mut stream = std::net::TcpStream::connect(address).unwrap();
    stream.write_all(request.as_bytes()).unwrap();
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);
    response
}

fn rectangle(lon: f64, lat: f64) -> Value {
    json!({ "type": "Polygon", "coordinates": [[
        [lon, lat], [lon + 0.003, lat], [lon + 0.003, lat + 0.002], [lon, lat + 0.002], [lon, lat]
    ]] })
}

#[test]
fn page_and_assets_are_served() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("school"), cache.path(), &[]);
    let (status, page) = selector.get("");
    assert_eq!(status, 200);
    assert!(page.contains("assets/vendor/leaflet/leaflet.js"), "{page}");
    assert!(page.contains("github.com/HenryJobst/freemapper"));
    let (status, script) = selector.get("assets/vendor/leaflet/leaflet.js");
    assert_eq!(status, 200);
    assert!(script.contains("Leaflet 1.9.4"));
    assert_eq!(selector.get("assets/missing.js").0, 404);
}

/// The page gets the supported regions and their aerial photos from the registry, and the
/// content security policy lets the page load images from exactly those services.
#[test]
fn adapters_endpoint_lists_the_states_with_previews() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("school"), cache.path(), &[]);
    let (status, body) = selector.get("api/adapters");
    assert_eq!(status, 200);
    let adapters: Value = serde_json::from_str(&body).unwrap();
    let adapters = adapters.as_array().unwrap();
    let ids: Vec<&str> = adapters
        .iter()
        .map(|adapter| adapter["id"].as_str().unwrap())
        .collect();
    assert_eq!(
        ids,
        [
            "brandenburg",
            "berlin",
            "saxony",
            "saxonyanhalt",
            "hamburg",
            "hessen",
            "schleswigholstein",
            "mecklenburgvorpommern",
            "nordrheinwestfalen",
            "niedersachsen",
            "rheinlandpfalz",
            "saarland"
        ]
    );
    assert_eq!(
        adapters[2]["names"],
        json!({ "en": "Saxony", "de": "Sachsen" })
    );
    // Every region has an aerial photo over https; Brandenburg and Berlin share LGB's.
    let mut services = Vec::new();
    for adapter in adapters {
        let preview = &adapter["preview"];
        let wms = preview["wms"].as_str().unwrap();
        assert!(wms.starts_with("https://") && !wms.contains('?'), "{wms}");
        for key in ["layer", "format", "attribution", "agency"] {
            assert!(!preview[key].as_str().unwrap().is_empty(), "{key} of {wms}");
        }
        assert!(preview["params"].is_object(), "{preview}");
        if !services.contains(&wms) {
            services.push(wms);
        }
    }
    assert_eq!(adapters[0]["preview"], adapters[1]["preview"]);
    assert_eq!(services.len(), 11);
    assert_eq!(adapters[4]["preview"]["params"], json!({ "time": "2026" }));
    assert!(
        adapters[10]["preview"]["attribution"]
            .as_str()
            .unwrap()
            .contains("LVermGeoRP{year},")
    );
    // The policy names the hosts of those eleven services, in that order, and no other.
    let response = selector.agent.get(&selector.url).call().unwrap();
    let policy = response
        .headers()
        .get("content-security-policy")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(
        policy,
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
         connect-src 'self'; img-src 'self' data: https://isk.geobasis-bb.de \
         https://geodienste.sachsen.de https://www.geodatenportal.sachsen-anhalt.de \
         https://geodienste.hamburg.de https://www.gds-srv.hessen.de https://dienste.gdi-sh.de \
         https://www.geodaten-mv.de https://www.wms.nrw.de \
         https://opendata.lgln.niedersachsen.de https://geo4.service24.rlp.de \
         https://geoportal.saarland.de; \
         frame-ancestors 'none'"
    );
    for service in services {
        let host = service.split('/').nth(2).unwrap();
        assert!(policy.contains(&format!("https://{host}")), "{host}");
    }
}

#[test]
fn requests_without_token_are_404() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("school"), cache.path(), &[]);
    let (address, token) = selector.address_and_token();
    for path in ["/", "/api/project", &format!("/{}x/", &token[..31])] {
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");
        assert_eq!(raw_status(&address, &request), 404, "{path}");
    }
}

#[test]
fn foreign_host_is_403() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("school"), cache.path(), &[]);
    let (address, token) = selector.address_and_token();
    let request = format!(
        "POST /{token}/api/done HTTP/1.1\r\nHost: evil.example\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    assert_eq!(raw_status(&address, &request), 403);
    let port = address.rsplit(':').next().unwrap();
    let request = format!(
        "GET /{token}/api/project HTTP/1.1\r\nHost: localhost:{port}\r\nConnection: close\r\n\r\n"
    );
    assert_eq!(raw_status(&address, &request), 200);
}

#[test]
fn preview_reports_tiles() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("school"), cache.path(), &[]);
    let (status, preview) = selector.post("api/preview", json!({ "area": school_area() }));
    assert_eq!(status, 200, "{preview}");
    assert_eq!(preview["tiles"][0]["name"], "33421-5821");
    assert_eq!(preview["tiles"][0]["corners"].as_array().unwrap().len(), 4);
    assert_eq!(preview["too_large"], false);
    assert!(preview["estimate"]["total_megabytes"].as_f64().unwrap() > 300.0);
    // The free space of the volume that holds the cache (and here the project folder, too).
    let volume = &preview["estimate"]["disk"]["volumes"][0];
    assert!(volume["free_megabytes"].as_u64().is_some(), "{preview}");
    assert!(
        volume["needed_megabytes"].as_u64().unwrap() > 300,
        "{preview}"
    );
    assert_eq!(preview["estimate"]["disk"]["reserve_megabytes"], 1024);
    assert!(
        preview["estimate"]["items"][0]["fits"].is_boolean(),
        "{preview}"
    );
}

#[test]
fn save_creates_the_project() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    let selector = Selector::start(&project, cache.path(), &["--kind", "sprint"]);
    // Raw and in one write: the 405 comes before the server reads the body, and if the body
    // arrives later the server closes the connection without saying so. The agent would then
    // reuse a dead connection (Windows 10053, Linux "Peer disconnected").
    let (address, token) = selector.address_and_token();
    let request = format!(
        "POST /{token}/api/project HTTP/1.1\r\nHost: {address}\r\nOrigin: http://{address}\r\n\
         Content-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
    );
    assert_eq!(raw_status(&address, &request), 405);
    let (_, state) = selector.get("api/project");
    let state: Value = serde_json::from_str(&state).unwrap();
    assert_eq!(state["exists"], false);
    assert_eq!(state["kind"], "sprint");
    let (status, saved) = selector.post(
        "api/save",
        json!({ "area": school_area(), "source_id": "way/992423119", "title": "Clara-Zetkin-Schule" }),
    );
    assert_eq!(status, 200, "{saved}");
    let text = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(
        text.contains("way/992423119") && text.contains("Clara-Zetkin-Schule"),
        "{text}"
    );
    assert!(text.contains("sprint"), "{text}");
}

#[test]
fn save_changes_an_existing_area() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    let status = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")))
        .env("FREEMAPPER_LANG", "en")
        .arg("init")
        .arg(&project)
        .args(["--geojson", FIXTURE, "--title", "Schule"])
        .status()
        .unwrap();
    assert!(status.success());
    let selector = Selector::start(&project, cache.path(), &[]);
    let (_, state) = selector.get("api/project");
    let state: Value = serde_json::from_str(&state).unwrap();
    assert_eq!(state["exists"], true);
    assert_eq!(state["title"], "Schule");
    assert_eq!(state["area"]["type"], "Polygon");
    let rectangle = json!({ "type": "Polygon", "coordinates": [[
        [13.843, 52.539], [13.846, 52.539], [13.846, 52.541], [13.843, 52.541], [13.843, 52.539]
    ]] });
    let (status, saved) = selector.save(json!({ "area": rectangle }));
    assert_eq!(status, 200, "{saved}");
    let text = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(text.contains("13.846") && text.contains("Schule"), "{text}");
    assert!(!text.contains("way/992423119"), "{text}");
    assert_eq!(saved["zone_changed"], false);
    // Halle, Saxony-Anhalt: the map moves to UTM zone 32.
    let (status, saved) = selector.save(json!({ "area": rectangle_at(11.971, 51.478) }));
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["zone_changed"], true, "{saved}");
    let text = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(text.contains("crs = 25832"), "{text}");
    // Hamburg lies in the same zone, but another provider serves it: the data is dropped as well.
    let (status, saved) = selector.save(json!({ "area": rectangle_at(9.98, 53.59) }));
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["zone_changed"], false, "{saved}");
    assert_eq!(saved["provider_changed"], true, "{saved}");
}

/// A project made by `init` in a fresh folder and a page on it.
fn page_on_a_project() -> (
    tempfile::TempDir,
    tempfile::TempDir,
    std::path::PathBuf,
    Selector,
) {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    let status = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")))
        .env("FREEMAPPER_LANG", "en")
        .arg("init")
        .arg(&project)
        .args(["--geojson", FIXTURE])
        .stdout(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
    let selector = Selector::start(&project, cache.path(), &[]);
    (dir, cache, project, selector)
}

#[test]
fn save_with_a_stale_revision_is_refused_and_writes_nothing() {
    let (_dir, _cache, project, selector) = page_on_a_project();
    let file = project.join("freemapper.toml");
    let seen = selector.project()["revision"].as_str().unwrap().to_owned();
    assert_eq!(seen.len(), 16, "{seen}");
    // Another window (or a run in a terminal) writes the file after this page read it.
    let mut other = std::fs::read_to_string(&file).unwrap();
    other.push_str("\n# changed elsewhere\n");
    std::fs::write(&file, &other).unwrap();
    let (status, answer) = selector.post_with_revision(
        "api/save",
        json!({ "area": rectangle(13.843, 52.539) }),
        Some(&seen),
    );
    assert_eq!(status, 409, "{answer}");
    assert!(
        answer["error"].as_str().unwrap().contains("another window"),
        "{answer}"
    );
    let current = answer["revision"].as_str().unwrap();
    assert_ne!(current, seen);
    assert_eq!(selector.project()["revision"], current);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), other);
    // A page that names no revision although the file exists has not seen it either.
    let (status, answer) = selector.post("api/save", json!({ "area": rectangle(13.843, 52.539) }));
    assert_eq!(status, 409, "{answer}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), other);
}

#[test]
fn save_with_the_current_revision_goes_through() {
    let (_dir, _cache, project, selector) = page_on_a_project();
    let seen = selector.project()["revision"].as_str().unwrap().to_owned();
    let (status, saved) = selector.post_with_revision(
        "api/save",
        json!({ "area": rectangle(13.843, 52.539) }),
        Some(&seen),
    );
    assert_eq!(status, 200, "{saved}");
    let text = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(text.contains("13.846"), "{text}");
    // The answer names the new revision, the one the page reads next.
    let new = saved["revision"].as_str().unwrap();
    assert_ne!(new, seen);
    assert_eq!(selector.project()["revision"], new);
    // The old revision is stale now.
    let (status, _) = selector.post_with_revision(
        "api/save",
        json!({ "area": rectangle(13.85, 52.539) }),
        Some(&seen),
    );
    assert_eq!(status, 409);
}

#[test]
fn first_save_needs_no_revision() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    let selector = Selector::start(&project, cache.path(), &[]);
    assert!(selector.project()["revision"].is_null());
    let (status, saved) = selector.post("api/save", json!({ "area": school_area() }));
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["created"], true);
    assert_eq!(selector.project()["revision"], saved["revision"], "{saved}");
    assert!(saved["revision"].is_string(), "{saved}");
}

fn rectangle_at(lon: f64, lat: f64) -> Value {
    rectangle(lon, lat)
}

#[test]
fn save_refuses_too_large() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    let selector = Selector::start(&project, cache.path(), &[]);
    let huge = json!({ "type": "Polygon", "coordinates": [[
        [13.7, 52.5], [13.9, 52.5], [13.9, 52.6], [13.7, 52.6], [13.7, 52.5]
    ]] });
    let (status, answer) = selector.post("api/save", json!({ "area": huge }));
    assert_eq!(status, 400);
    assert!(
        answer["error"].as_str().unwrap().contains("per side"),
        "{answer}"
    );
    assert!(!project.join("freemapper.toml").exists());
}

#[test]
fn bad_area_is_a_400() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("school"), cache.path(), &[]);
    let line = json!({ "type": "Polygon", "coordinates": [[[13.84, 52.53], [13.85, 52.54], [13.84, 52.53]]] });
    let bowtie = json!({ "type": "Polygon", "coordinates": [[
        [13.84, 52.53], [13.85, 52.54], [13.85, 52.53], [13.84, 52.54], [13.84, 52.53]
    ]] });
    for area in [line, bowtie, json!("nonsense")] {
        for path in ["api/preview", "api/save"] {
            let (status, answer) = selector.post(path, json!({ "area": area }));
            assert_eq!(status, 400, "{path} {area}: {answer}");
            assert!(answer["error"].is_string(), "{answer}");
        }
    }
}

#[test]
fn done_stops_the_server() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let mut selector = Selector::start(&dir.path().join("school"), cache.path(), &[]);
    let (status, _) = selector.post("api/done", json!({}));
    assert_eq!(status, 200);
    let exit = selector.wait().expect("select ends after done");
    assert!(exit.success());
}

#[test]
fn foreign_origin_and_bad_requests_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("school"), cache.path(), &[]);
    let (address, token) = selector.address_and_token();
    let port = address.rsplit(':').next().unwrap();
    let post = |origin: &str, content_type: &str| {
        format!(
            "POST /{token}/api/preview HTTP/1.1\r\nHost: {address}\r\n{origin}Content-Type: {content_type}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        )
    };
    let evil = post("Origin: https://evil.example\r\n", "application/json");
    assert_eq!(raw_status(&address, &evil), 403);
    let own = post(&format!("Origin: http://{address}\r\n"), "application/json");
    assert_eq!(raw_status(&address, &own), 400, "own origin, empty area");
    let form = post("", "text/plain");
    // Refused by the shared guard (strict POST) before the JSON extractor would answer 415.
    assert_eq!(raw_status(&address, &form), 403);
    let no_port = format!("GET /{token}/ HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    assert_eq!(raw_status(&address, &no_port), 403);
    let page = raw_response(
        &address,
        &format!("GET /{token}/ HTTP/1.1\r\nHost: localhost:{port}\r\nConnection: close\r\n\r\n"),
    )
    .to_lowercase();
    assert!(page.contains("referrer-policy: no-referrer"), "{page}");
}

#[test]
fn preview_uses_the_kind_chosen_on_the_page() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("school"), cache.path(), &[]);
    let width = |kind: &str| {
        let (status, preview) = selector.post(
            "api/preview",
            json!({ "area": school_area(), "kind": kind }),
        );
        assert_eq!(status, 200, "{preview}");
        preview["width_m"].as_f64().unwrap()
    };
    // Sprint buffer 50 m, schoolyard 20 m: 2 × 30 m wider.
    assert!((width("sprint") - width("schoolyard") - 60.0).abs() < 0.01);
}

#[test]
fn area_outside_brandenburg_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("munich");
    let selector = Selector::start(&project, cache.path(), &[]);
    let munich = rectangle(11.57, 48.13);
    let (status, preview) = selector.post("api/preview", json!({ "area": munich }));
    assert_eq!(status, 200, "{preview}");
    assert_eq!(preview["outside_coverage"], true);
    assert!(preview["tiles"].as_array().unwrap().is_empty());
    let (status, answer) = selector.post("api/save", json!({ "area": munich }));
    assert_eq!(status, 400);
    assert!(
        answer["error"].as_str().unwrap().contains("Brandenburg"),
        "{answer}"
    );
    assert!(!project.exists());
}

/// Puts the adapter `theirs` into the folder `adapters` of `project` and makes the project
/// file name it as the provider of its rasters: a project that was passed on with a source of
/// its own.
fn bring_an_adapter_along(project: &std::path::Path) {
    let folder = project.join("adapters").join("theirs");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(
        folder.join("manifest.toml"),
        "format = 1\n\n[adapter]\nid = \"theirs\"\nrevision = 2\nmin_freemapper = \"0.13.0\"\ncrs = 25832\n\
         names = { en = \"Sample\", de = \"Beispiel\" }\n\n[net]\nhosts = [\"geo.example.org\"]\n\n\
         [boundary]\nfile = \"boundary.geojson\"\ncrs = 25833\n\n[[osm.extract]]\n\
         geofabrik = \"europe/germany/hamburg\"\nmegabytes = 54\nlabel = \"Sample\"\n\n[attribution]\n\
         licence = \"dl-de/by-2-0\"\ntext = \"© Sample {year}\"\n\n[alkis]\nkind = \"wfs\"\n\
         url = \"https://geo.example.org/wfs\"\nbuildings_type = \"ave:GebaeudeBauwerk\"\n\
         land_use_type = \"ave:NutzungFlurstueck\"\n",
    )
    .unwrap();
    std::fs::write(
        folder.join("boundary.geojson"),
        r#"{"type":"Feature","properties":{},"geometry":{"type":"Polygon","coordinates":
        [[[400000,5800000],[401000,5800000],[401000,5801000],[400000,5801000],[400000,5800000]]]}}"#,
    )
    .unwrap();
    let file = project.join("freemapper.toml");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("provider = \"lgb\""), "{text}");
    std::fs::write(
        &file,
        text.replace("provider = \"lgb\"", "provider = \"theirs\""),
    )
    .unwrap();
}

/// The area page never loads the adapters of a project folder (ADR 0082): a project that
/// names an adapter it brought along is one whose adapter is not installed, for the page and
/// for a run started from it.
#[test]
fn page_never_loads_the_adapters_of_a_project() {
    let (_dir, _cache, project, first) = page_on_a_project();
    drop(first);
    bring_an_adapter_along(&project);
    let cache = tempfile::tempdir().unwrap();
    let user = tempfile::tempdir().unwrap();
    let selector = Selector::start_with_env(
        &project,
        cache.path(),
        &[],
        &[("FREEMAPPER_ADAPTER_DIR", user.path().to_str().unwrap())],
    );
    let unknown =
        "the project file names the adapter \\\"theirs\\\", which this freemapper does not have";
    let (status, body) = selector.get("api/project");
    assert_eq!(status, 400, "{body}");
    assert!(body.contains(unknown), "{body}");
    let (status, answer) = selector.post("api/run", json!({ "offline": true }));
    assert!((400..500).contains(&status), "{status}: {answer}");
    assert!(answer.to_string().contains(unknown), "{answer}");
}

/// The area page reads the hosts the person allowed and never writes the file: an entry that
/// a release made redundant stays for a command of the person to remove.
#[cfg(unix)]
#[test]
fn page_reads_the_allowed_hosts_and_never_writes_them() {
    let home = tempfile::tempdir().unwrap();
    let config = if cfg!(target_os = "macos") {
        home.path().join("Library").join("Application Support")
    } else {
        home.path().join("xdg")
    };
    let file = config.join("freemapper").join("allowed-hosts.toml");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let stored = "[allow]\nhessen = [\"gds.hessen.de\", \"cdn.example.net\"]\n";
    std::fs::write(&file, stored).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let xdg = home.path().join("xdg");
    let selector = Selector::start_with_env(
        &dir.path().join("school"),
        cache.path(),
        &[],
        &[
            ("HOME", home.path().to_str().unwrap()),
            ("XDG_CONFIG_HOME", xdg.to_str().unwrap()),
        ],
    );
    let (status, body) = selector.get("api/adapters");
    assert_eq!(status, 200, "{body}");
    drop(selector);
    assert_eq!(std::fs::read_to_string(&file).unwrap(), stored);
}

#[test]
fn broken_project_file_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    std::fs::create_dir(&project).unwrap();
    std::fs::write(project.join("freemapper.toml"), "map_kind = [broken").unwrap();
    let selector = Selector::start(&project, cache.path(), &[]);
    let mut response = selector
        .agent
        .get(&format!("{}api/project", selector.url))
        .call()
        .unwrap();
    assert_eq!(response.status().as_u16(), 400);
    let text = response.body_mut().read_to_string().unwrap();
    assert!(
        text.contains("freemapper.toml") || text.contains("TOML"),
        "{text}"
    );
}

#[test]
fn save_into_a_project_with_a_map_says_so() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    for args in [
        vec!["init", project.to_str().unwrap(), "--geojson", FIXTURE],
        vec!["export", project.to_str().unwrap()],
    ] {
        let status = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")))
            .env("FREEMAPPER_LANG", "en")
            .args(&args)
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "{args:?}");
    }
    let selector = Selector::start(&project, cache.path(), &[]);
    let (_, state) = selector.get("api/project");
    let state: Value = serde_json::from_str(&state).unwrap();
    assert_eq!(state["has_map"], true);
    assert_eq!(state["from_agent"], false);
    let (status, saved) = selector.save(json!({ "area": rectangle(13.843, 52.539) }));
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["has_map"], true);
    assert_eq!(saved["created"], false);
}

#[test]
fn busy_port_names_the_port() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("a"), cache.path(), &[]);
    let (address, _) = selector.address_and_token();
    let port = address.rsplit(':').next().unwrap().to_string();
    let output = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")))
        .env("FREEMAPPER_LANG", "en")
        .arg("select")
        .arg(dir.path().join("b"))
        .args([
            "--no-browser",
            "--port",
            &port,
            "--cache-dir",
            cache.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&port) && stderr.contains("--port"),
        "{stderr}"
    );
}

/// The environment of a process in which nothing names a language.
const NO_LANGUAGE: [(&str, &str); 4] = [
    ("FREEMAPPER_LANG", ""),
    ("LC_ALL", ""),
    ("LC_MESSAGES", ""),
    ("LANG", "C"),
];

/// `api/texts` of a page started with `extra` and `env`, asked with `accept_language`.
fn texts_answer(extra: &[&str], env: &[(&str, &str)], accept_language: Option<&str>) -> Value {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start_with_env(&dir.path().join("p"), cache.path(), extra, env);
    let mut request = selector.agent.get(&format!("{}api/texts", selector.url));
    if let Some(value) = accept_language {
        request = request.header("Accept-Language", value);
    }
    let mut response = request.call().unwrap();
    assert_eq!(response.status().as_u16(), 200);
    serde_json::from_str(&response.body_mut().read_to_string().unwrap()).unwrap()
}

#[test]
fn api_texts_language_comes_from_lang_then_the_browser_then_english() {
    let german = Some("de-DE,de;q=0.9,en;q=0.8");
    // --lang wins over the browser, and so does the environment's language.
    let answer = texts_answer(&["--lang", "de"], &NO_LANGUAGE, Some("en-US,en"));
    assert_eq!(answer["lang"], "de");
    assert_eq!(answer["texts"]["page-title"], "freemapper – Gebietsauswahl");
    assert_eq!(
        texts_answer(&[], &[], german)["lang"],
        "en",
        "FREEMAPPER_LANG=en"
    );
    // Nothing named: the browser's language, else English.
    let answer = texts_answer(&[], &NO_LANGUAGE, german);
    assert_eq!(answer["lang"], "de");
    assert_eq!(answer["texts"]["page-heading"], "Kartengebiet wählen");
    assert_eq!(
        texts_answer(&[], &NO_LANGUAGE, Some("fr, en;q=0.5"))["lang"],
        "en"
    );
    let answer = texts_answer(&[], &NO_LANGUAGE, Some("fr"));
    assert_eq!(answer["lang"], "en");
    assert_eq!(answer["texts"]["page-title"], "freemapper – area selection");
    assert_eq!(texts_answer(&[], &NO_LANGUAGE, None)["lang"], "en");
    // Patterns keep their placeholders; the suite's own words come along.
    let texts = &answer["texts"];
    assert_eq!(
        texts["page-size"],
        "Size with buffer: { $width } × { $height } m"
    );
    assert_eq!(texts["suite-banner-dismiss"], "Dismiss");
    assert!(
        texts
            .as_object()
            .unwrap()
            .keys()
            .all(|id| id.starts_with("page-") || id.starts_with("suite-"))
    );
    // The project answer no longer carries a language.
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start(&dir.path().join("p"), cache.path(), &[]);
    let (_, state) = selector.get("api/project");
    let state: Value = serde_json::from_str(&state).unwrap();
    assert!(state.get("lang").is_none(), "{state}");
}

/// The server's refusals come in the page's language like its texts: with nothing named, a
/// German browser gets German sentences, any other English ones.
#[test]
fn refusals_come_in_the_language_of_the_page() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let selector = Selector::start_with_env(&dir.path().join("p"), cache.path(), &[], &NO_LANGUAGE);
    let origin = selector
        .url
        .split('/')
        .take(3)
        .collect::<Vec<_>>()
        .join("/");
    let post = |path: &str, body: Value, accept: &str| {
        let mut response = selector
            .agent
            .post(&format!("{}{path}", selector.url))
            .header("Content-Type", "application/json")
            .header("Origin", &origin)
            .header("Accept-Language", accept)
            .send(body.to_string())
            .unwrap();
        let status = response.status().as_u16();
        let answer: Value =
            serde_json::from_str(&response.body_mut().read_to_string().unwrap()).unwrap();
        (
            status,
            answer["error"].as_str().unwrap_or_default().to_string(),
        )
    };
    for (accept, cancel, run) in [
        (
            "de-DE,de;q=0.9,en;q=0.8",
            "Es läuft nichts, also gibt es nichts abzubrechen.",
            "In diesem Ordner gibt es noch kein gespeichertes Projekt. Speichern Sie zuerst das Gebiet.",
        ),
        (
            "fr, en;q=0.5",
            "No run is going on, so there is nothing to cancel.",
            "There is no saved project in this folder yet. Save the area first.",
        ),
    ] {
        assert_eq!(post("api/cancel", json!({}), accept), (409, cancel.into()));
        assert_eq!(post("api/run", json!({}), accept), (404, run.into()));
    }
    let (status, german) = post("api/run", json!({ "pdf_sources": "x" }), "de");
    assert_eq!(status, 400);
    assert!(
        german.contains("pdf_sources muss auto, full oder credit-only sein"),
        "{german}"
    );
    // An area the engine refuses: its error in German, and the save with an unknown kind too.
    let line = json!({ "type": "Polygon", "coordinates": [[[13.84, 52.53], [13.85, 52.54], [13.84, 52.53]]] });
    let (status, english) = post("api/save", json!({ "area": line }), "en");
    let (_, german) = post("api/save", json!({ "area": line }), "de");
    assert_eq!(status, 400);
    assert_ne!(german, english, "{german}");
    let (status, german) = post(
        "api/save",
        json!({ "area": school_area(), "kind": "gym" }),
        "de",
    );
    assert_eq!(
        (status, german.as_str()),
        (
            400,
            "Unbekannte Kartenart „gym“. Möglich sind schoolyard, sprint und forest."
        )
    );
}

/// `api/update` of a page whose cache holds `record` (if any). `base` is the update stub the
/// program is told about (a build with the `test-stub` feature counts as a release build then).
fn update_answer(record: Option<Value>, base: &str) -> Value {
    let cache = tempfile::tempdir().unwrap();
    let config = tempfile::tempdir().unwrap();
    if let Some(record) = record {
        std::fs::write(cache.path().join("update-check.json"), record.to_string()).unwrap();
    }
    let dir = tempfile::tempdir().unwrap();
    let selector = Selector::start_with_env(
        &dir.path().join("p"),
        cache.path(),
        &[],
        &[
            ("FREEMAPPER_UPDATE_TEST_BASE", base),
            ("FREEMAPPER_CONFIG_DIR", config.path().to_str().unwrap()),
            ("FREEMAPPER_UPDATE", "notify"),
        ],
    );
    let (status, body) = selector.get("api/update");
    assert_eq!(status, 200, "{body}");
    serde_json::from_str(&body).unwrap()
}

fn available_record(version: &str) -> Value {
    json!({
        "at": chrono::Utc::now().timestamp(),
        "by": env!("CARGO_PKG_VERSION"),
        "result": {
            "outcome": "available",
            "version": version,
            "notes": "https://github.com/HenryJobst/freemapper/releases/tag/v9.9.9",
        },
    })
}

#[cfg(feature = "test-stub")]
#[test]
fn api_update_reads_the_record_only() {
    // A listener that counts connections: the page must not reach it.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let answer = update_answer(Some(available_record("9.9.9")), &base);
    assert_eq!(
        answer,
        json!({"available": {
            "version": "9.9.9",
            "notes": "https://github.com/HenryJobst/freemapper/releases/tag/v9.9.9",
        }})
    );
    // A record that says nothing newer shows nothing.
    let current = json!({
        "at": chrono::Utc::now().timestamp(),
        "by": env!("CARGO_PKG_VERSION"),
        "result": {"outcome": "current"},
    });
    assert_eq!(
        update_answer(Some(current), &base),
        json!({"available": null})
    );
    assert!(listener.accept().is_err(), "the update stub got a request");
}

#[cfg(feature = "test-stub")]
#[test]
fn api_update_drops_notes_outside_github() {
    // A changed record file must not put a foreign link on the page.
    let mut record = available_record("9.9.9");
    record["result"]["notes"] = json!("https://example.org/notes");
    let answer = update_answer(Some(record), "http://127.0.0.1:9");
    assert_eq!(
        answer,
        json!({"available": {"version": "9.9.9", "notes": null}})
    );
}

#[cfg(feature = "test-stub")]
#[test]
fn api_update_null_without_record() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    // Without a record the daily check is due and may run against the stub, which says nothing;
    // the answer of the page is null either way.
    assert_eq!(update_answer(None, &base), json!({"available": null}));
}

#[test]
fn api_update_is_null_in_a_source_build() {
    // Without the test-stub feature nothing makes this a release build.
    if cfg!(feature = "test-stub") {
        return;
    }
    let answer = update_answer(Some(available_record("9.9.9")), "http://127.0.0.1:9");
    assert_eq!(answer, json!({"available": null}));
}

// --- Content profiles (ADR 0100, 0101) -------------------------------------------------------

/// The content settings in the project file in `dir`, read back with the library.
fn stored_content(dir: &Path) -> Option<freemapper::build::content::ContentSettings> {
    freemapper::project::ProjectFile::load(dir).unwrap().content
}

#[test]
fn project_answer_carries_the_content_settings() {
    let (_dir, _cache, project, selector) = page_on_a_project();
    let state = selector.project();
    assert!(state["content"].is_null(), "{state}");
    // The lists the page shows come from the library, in its order.
    let groups: Vec<&str> = state["content_options"]["groups"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g.as_str().unwrap())
        .collect();
    let expected: Vec<&str> = freemapper::build::content::Group::ALL
        .iter()
        .map(|g| g.name())
        .collect();
    assert_eq!(groups, expected);
    assert_eq!(
        state["content_options"]["profiles"],
        json!(["safe", "balanced", "full"])
    );
    assert_eq!(
        state["content_options"]["places"],
        json!(["map", "candidates", "off"])
    );
    let status = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")))
        .env("FREEMAPPER_LANG", "en")
        .arg("init")
        .arg(project.parent().unwrap().join("other"))
        .args([
            "--geojson",
            FIXTURE,
            "--content",
            "full",
            "--content-set",
            "walls=off",
        ])
        .stdout(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success());
    let cache = tempfile::tempdir().unwrap();
    let other = Selector::start(&project.parent().unwrap().join("other"), cache.path(), &[]);
    assert_eq!(
        other.project()["content"],
        json!({ "profile": "full", "walls": "off" })
    );
}

#[test]
fn save_stores_the_content_in_the_same_write() {
    let (_dir, _cache, project, selector) = page_on_a_project();
    let (status, saved) = selector.save(json!({
        "area": rectangle(13.843, 52.539),
        "content": { "profile": "safe", "walls": "map", "water-features": "off" },
    }));
    assert_eq!(status, 200, "{saved}");
    let content = stored_content(&project).unwrap();
    assert_eq!(content.profile.name(), "safe");
    assert_eq!(content.summary(), "safe, walls=map, water-features=off");
    // One write: the revision in the answer is that of the file now.
    let bytes = std::fs::read(project.join("freemapper.toml")).unwrap();
    assert_eq!(saved["revision"], freemapper_web::revision(&bytes));
    assert_eq!(selector.project()["revision"], saved["revision"]);
    assert_eq!(
        selector.project()["content"],
        json!({ "profile": "safe", "walls": "map", "water-features": "off" })
    );
}

#[test]
fn first_save_stores_the_content() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    let selector = Selector::start(&project, cache.path(), &[]);
    let (status, saved) = selector.post(
        "api/save",
        json!({ "area": school_area(), "content": { "profile": "full", "trees": "off" } }),
    );
    assert_eq!(status, 200, "{saved}");
    assert_eq!(
        stored_content(&project).unwrap().summary(),
        "full, trees=off"
    );
}

#[test]
fn save_without_content_keeps_the_stored_settings() {
    let (_dir, _cache, project, selector) = page_on_a_project();
    let (status, saved) = selector.save(json!({
        "area": rectangle(13.843, 52.539),
        "content": { "profile": "full", "fences": "candidates" },
    }));
    assert_eq!(status, 200, "{saved}");
    let (status, saved) = selector.save(json!({ "area": rectangle(13.85, 52.539) }));
    assert_eq!(status, 200, "{saved}");
    assert_eq!(
        stored_content(&project).unwrap().summary(),
        "full, fences=candidates"
    );
}

#[test]
fn save_with_invalid_content_is_refused_and_writes_nothing() {
    let (_dir, _cache, project, selector) = page_on_a_project();
    let file = project.join("freemapper.toml");
    let before = std::fs::read_to_string(&file).unwrap();
    for content in [
        json!({ "profile": "balanced", "gardens": "map" }),
        json!({ "profile": "balanced", "walls": "everywhere" }),
        json!({ "profile": "extreme" }),
        json!("balanced"),
    ] {
        let (status, answer) = selector.save(json!({
            "area": rectangle(13.843, 52.539),
            "content": content,
        }));
        assert_eq!(status, 422, "{content}: {answer}");
        // `refused` answers with the sentence as plain text.
        assert!(answer.as_str().is_some_and(|e| !e.is_empty()), "{answer}");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), before, "{content}");
    }
}

#[test]
fn save_with_content_and_a_stale_revision_is_refused() {
    let (_dir, _cache, project, selector) = page_on_a_project();
    let file = project.join("freemapper.toml");
    let seen = selector.project()["revision"].as_str().unwrap().to_owned();
    let mut other = std::fs::read_to_string(&file).unwrap();
    other.push_str("\n# changed elsewhere\n");
    std::fs::write(&file, &other).unwrap();
    let (status, answer) = selector.post_with_revision(
        "api/save",
        json!({ "area": rectangle(13.843, 52.539), "content": { "profile": "full" } }),
        Some(&seen),
    );
    assert_eq!(status, 409, "{answer}");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), other);
}

#[test]
fn a_save_with_content_never_shows_a_half_written_file() {
    // Two writes (the area, then the content) would leave a file with the new area and the old
    // content for a moment. A reader that polls the file all the time sees every state that the
    // file has had; with one write it sees the old file and the new one, nothing between. A
    // polling reader can miss a state, so this test can pass by luck with two writes, but it
    // never fails with one (the writes are renames, a reader sees whole files).
    let (_dir, _cache, project, selector) = page_on_a_project();
    let file = project.join("freemapper.toml");
    let before = std::fs::read(&file).unwrap();
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let seen = {
        let (stop, file) = (stop.clone(), file.clone());
        std::thread::spawn(move || {
            let mut states: Vec<Vec<u8>> = Vec::new();
            while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                if let Ok(bytes) = std::fs::read(&file)
                    && states.last() != Some(&bytes)
                {
                    states.push(bytes);
                }
            }
            states
        })
    };
    std::thread::sleep(Duration::from_millis(50));
    let (status, saved) = selector.save(json!({
        "area": rectangle(13.843, 52.539),
        "content": { "profile": "full", "walls": "off" },
    }));
    assert_eq!(status, 200, "{saved}");
    std::thread::sleep(Duration::from_millis(50));
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let states = seen.join().unwrap();
    let after = std::fs::read(&file).unwrap();
    assert_ne!(before, after);
    for state in &states {
        assert!(
            *state == before || *state == after,
            "the file had a state between the old and the new one:\n{}",
            String::from_utf8_lossy(state)
        );
    }
    assert!(states.contains(&after));
}

#[test]
fn content_written_elsewhere_shows_in_the_next_project_answer() {
    // The page takes this answer after a conflict, so an untouched content section follows the file.
    let (_dir, _cache, project, selector) = page_on_a_project();
    let seen = selector.project()["revision"].as_str().unwrap().to_owned();
    assert!(selector.project()["content"].is_null());
    freemapper::project::set_content(
        &project,
        Some(freemapper::build::content::ContentSettings::with_profile(
            freemapper::build::content::Profile::Full,
        )),
    )
    .unwrap();
    let state = selector.project();
    assert_eq!(state["content"], json!({ "profile": "full" }));
    assert_ne!(state["revision"].as_str().unwrap(), seen);
    // A save with the old revision is refused and the foreign choice stays in the file.
    let (status, _) = selector.post_with_revision(
        "api/save",
        json!({ "area": rectangle(13.843, 52.539), "content": { "profile": "balanced" } }),
        Some(&seen),
    );
    assert_eq!(status, 409);
    assert_eq!(stored_content(&project).unwrap().profile.name(), "full");
}

/// Status, content type, cache control and body of a GET.
fn get_bytes(selector: &Selector, path: &str) -> (u16, String, String, Vec<u8>) {
    let mut response = selector
        .agent
        .get(&format!("{}{path}", selector.url))
        .call()
        .unwrap();
    let header = |name: &str| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string()
    };
    let (content_type, cache_control) = (header("content-type"), header("cache-control"));
    let status = response.status().as_u16();
    let body = response.body_mut().read_to_vec().unwrap();
    (status, content_type, cache_control, body)
}

/// A POST as the page sends it, in the browser language `accept`; the error sentence of a JSON
/// answer, or the whole text of another one.
fn post_in(selector: &Selector, path: &str, body: Value, accept: &str) -> (u16, Value) {
    let origin = selector
        .url
        .split('/')
        .take(3)
        .collect::<Vec<_>>()
        .join("/");
    let mut response = selector
        .agent
        .post(&format!("{}{path}", selector.url))
        .header("Content-Type", "application/json")
        .header("Origin", &origin)
        .header("Accept-Language", accept)
        .send(body.to_string())
        .unwrap();
    let status = response.status().as_u16();
    let text = response.body_mut().read_to_string().unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(Value::String(text)),
    )
}

/// `RIFF`, the size, `WEBP`.
fn is_webp(bytes: &[u8]) -> bool {
    bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

/// The map image routes: the stored preview after an export, live previews by the content
/// settings, and the refusals in the page's language. No network: `init` and `export` without
/// fetched data give an (almost empty) map and its preview.
#[test]
fn map_image_routes_serve_the_stored_and_live_previews() {
    let dir = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    let freemapper = |args: &[&str]| {
        let status = isolated(Command::new(env!("CARGO_BIN_EXE_freemapper")))
            .env("FREEMAPPER_LANG", "en")
            .args(args)
            .stdout(Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "{args:?}");
    };
    let path = project.to_str().unwrap();
    freemapper(&["init", path, "--geojson", FIXTURE]);
    let selector = Selector::start_with_env(&project, cache.path(), &[], &NO_LANGUAGE);
    let state = |selector: &Selector| -> Value {
        let (status, body) = selector.get("api/map-image/state");
        assert_eq!(status, 200, "{body}");
        serde_json::from_str(&body).unwrap()
    };

    // No map yet: no stored image, and no live one either.
    let (status, ..) = get_bytes(&selector, "api/map-image");
    assert_eq!(status, 404);
    assert_eq!(
        state(&selector),
        json!({ "stored": false, "fresh": false, "content": null })
    );
    for (accept, sentence) in [
        ("de-DE,de;q=0.9", "Erzeugen Sie zuerst die Karte."),
        ("en", "Create the map first."),
    ] {
        let (status, answer) = post_in(&selector, "api/map-image", json!({}), accept);
        assert_eq!(status, 409, "{answer}");
        assert_eq!(answer["error"], sentence, "{answer}");
    }

    // After the export the stored preview is served, and it fits the saved project.
    freemapper(&["export", path]);
    let (status, content_type, cache_control, stored) = get_bytes(&selector, "api/map-image");
    assert_eq!(status, 200);
    assert_eq!(content_type, "image/webp");
    assert_eq!(cache_control, "no-store");
    assert!(is_webp(&stored));
    assert_eq!(
        state(&selector),
        json!({ "stored": true, "fresh": true, "content": { "profile": "balanced" } })
    );

    // A live preview with other settings, kept in memory until a newer one replaces it.
    let (status, first) = post_in(
        &selector,
        "api/map-image",
        json!({ "content": { "profile": "full" } }),
        "en",
    );
    assert_eq!(status, 200, "{first}");
    assert_eq!(first["content"], json!({ "profile": "full" }));
    assert!(first["width"].as_u64().unwrap() > 0, "{first}");
    assert!(first["height"].as_u64().unwrap() > 0, "{first}");
    let first_id = first["id"].as_u64().unwrap();
    let (status, content_type, cache_control, live) =
        get_bytes(&selector, &format!("api/map-image/live/{first_id}"));
    assert_eq!(status, 200);
    assert_eq!(content_type, "image/webp");
    assert_eq!(cache_control, "no-store");
    assert!(is_webp(&live));
    // Without settings the project's own are used; the older image is gone.
    let (status, second) = post_in(&selector, "api/map-image", json!({}), "en");
    assert_eq!(status, 200, "{second}");
    assert_eq!(second["content"], json!({ "profile": "balanced" }));
    let second_id = second["id"].as_u64().unwrap();
    assert!(second_id > first_id, "{first} {second}");
    let (status, ..) = get_bytes(&selector, &format!("api/map-image/live/{first_id}"));
    assert_eq!(status, 404);
    let (status, _, _, live) = get_bytes(&selector, &format!("api/map-image/live/{second_id}"));
    assert_eq!(status, 200);
    assert!(is_webp(&live));
    for path in ["api/map-image/live/0", "api/map-image/live/x"] {
        let (status, body) = selector.get(path);
        assert_eq!(status, 404, "{path}: {body}");
        assert_eq!(
            serde_json::from_str::<Value>(&body).unwrap()["error"],
            "This preview is no longer available.",
            "{path}"
        );
    }

    // Settings that are no valid choice: 422 with one sentence, nothing computed.
    for content in [json!({ "profile": "nope" }), json!({ "colour": "map" })] {
        let (status, answer) = post_in(
            &selector,
            "api/map-image",
            json!({ "content": content }),
            "de",
        );
        assert_eq!(status, 422, "{answer}");
        assert_eq!(answer, "Die Inhaltseinstellungen sind ungültig.");
    }
    // A body that is no JSON object: 400 in the page's language, nothing computed either.
    for body in ["[]", "\"x\"", "1", "{"] {
        let mut response = selector
            .agent
            .post(&format!("{}api/map-image", selector.url))
            .header("Content-Type", "application/json")
            .header(
                "Origin",
                selector
                    .url
                    .split('/')
                    .take(3)
                    .collect::<Vec<_>>()
                    .join("/"),
            )
            .header("Accept-Language", "de")
            .send(body)
            .unwrap();
        assert_eq!(response.status().as_u16(), 400, "{body}");
        let answer: Value =
            serde_json::from_str(&response.body_mut().read_to_string().unwrap()).unwrap();
        assert!(
            answer["error"]
                .as_str()
                .unwrap()
                .starts_with("Die Anfrage für die Vorschau ist ungültig."),
            "{body}: {answer}"
        );
    }
    // The newest live image is still the one before.
    let (status, ..) = get_bytes(&selector, &format!("api/map-image/live/{second_id}"));
    assert_eq!(status, 200);

    // A save of the same area with other content settings, as the page makes it, keeps the
    // stored data fitting: the preview stays fresh and still names the settings it shows.
    let page = selector.project();
    let (status, saved) = selector.save(json!({
        "area": page["area"],
        "source_id": page["source_id"],
        "content": { "profile": "full" },
    }));
    assert_eq!(status, 200, "{saved}");
    assert_eq!(
        state(&selector),
        json!({ "stored": true, "fresh": true, "content": { "profile": "balanced" } })
    );
    let (status, answer) = post_in(&selector, "api/map-image", json!({}), "en");
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["content"], json!({ "profile": "full" }));
    // The refused requests took no number.
    assert_eq!(answer["id"].as_u64().unwrap(), second_id + 1, "{answer}");

    // Another area saved: the stored data is not that area's any more, so a new run is needed.
    let (status, saved) = selector.save(json!({ "area": rectangle(13.843, 52.539) }));
    assert_eq!(status, 200, "{saved}");
    let (status, answer) = post_in(&selector, "api/map-image", json!({}), "de-DE,de");
    assert_eq!(status, 409, "{answer}");
    assert_eq!(
        answer["error"],
        "Das gespeicherte Projekt passt nicht mehr zur erzeugten Karte, etwa weil sich Gebiet, \
         Kartenart oder Maßstab geändert haben. Starten Sie zuerst einen Lauf, der die Karte \
         ersetzt („Vorhandene Karte ersetzen“ auf der Seite oder freemapper run --force).",
        "{answer}"
    );
    // The stored image stays, marked as not fresh.
    assert_eq!(
        state(&selector),
        json!({ "stored": true, "fresh": false, "content": { "profile": "balanced" } })
    );
    let (status, _, _, again) = get_bytes(&selector, "api/map-image");
    assert_eq!(status, 200);
    assert_eq!(again, stored);
}
