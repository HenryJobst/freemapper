//! The area selection page served by `freemapper select`, spoken to over HTTP as the page does.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

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
        let mut child = Command::new(env!("CARGO_BIN_EXE_freemapper"))
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
        let mut response = self
            .agent
            .post(&format!("{}{path}", self.url))
            .header("Content-Type", "application/json")
            // The page sends its own origin; the server refuses a POST without it.
            .header(
                "Origin",
                self.url.split('/').take(3).collect::<Vec<_>>().join("/"),
            )
            .send(body.to_string())
            .unwrap();
        let status = response.status().as_u16();
        let text = response.body_mut().read_to_string().unwrap();
        (
            status,
            serde_json::from_str(&text).unwrap_or(Value::String(text)),
        )
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
    let (status, state) = selector.post("api/project", json!({}));
    assert_eq!(status, 405, "{state}");
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
    let status = Command::new(env!("CARGO_BIN_EXE_freemapper"))
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
    let (status, saved) = selector.post("api/save", json!({ "area": rectangle }));
    assert_eq!(status, 200, "{saved}");
    let text = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(text.contains("13.846") && text.contains("Schule"), "{text}");
    assert!(!text.contains("way/992423119"), "{text}");
    assert_eq!(saved["zone_changed"], false);
    // Halle, Saxony-Anhalt: the map moves to UTM zone 32.
    let (status, saved) =
        selector.post("api/save", json!({ "area": rectangle_at(11.971, 51.478) }));
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["zone_changed"], true, "{saved}");
    let text = std::fs::read_to_string(project.join("freemapper.toml")).unwrap();
    assert!(text.contains("crs = 25832"), "{text}");
    // Hamburg lies in the same zone, but another provider serves it: the data is dropped as well.
    let (status, saved) = selector.post("api/save", json!({ "area": rectangle_at(9.98, 53.59) }));
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["zone_changed"], false, "{saved}");
    assert_eq!(saved["provider_changed"], true, "{saved}");
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
        let status = Command::new(env!("CARGO_BIN_EXE_freemapper"))
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
    let (status, saved) = selector.post("api/save", json!({ "area": rectangle(13.843, 52.539) }));
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
    let output = Command::new(env!("CARGO_BIN_EXE_freemapper"))
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

#[test]
fn the_page_takes_over_a_named_program_language() {
    let cache = tempfile::tempdir().unwrap();
    let lang_of = |extra: &[&str]| {
        let dir = tempfile::tempdir().unwrap();
        let selector = Selector::start(&dir.path().join("p"), cache.path(), extra);
        let (status, body) = selector.get("api/project");
        assert_eq!(status, 200, "{body}");
        serde_json::from_str::<Value>(&body).unwrap()["lang"].clone()
    };
    // The test environment names English; the switch wins over it.
    assert_eq!(lang_of(&[]), "en");
    assert_eq!(lang_of(&["--lang", "de"]), "de");
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
