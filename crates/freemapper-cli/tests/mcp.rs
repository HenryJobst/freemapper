//! The MCP server spoken to over stdio, as an agent would.

use std::io::{BufRead, BufReader, Read as _, Write};
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

#[cfg(all(feature = "test-stub", debug_assertions))]
mod support;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/clara-zetkin-schule.geojson"
);

struct Session {
    child: Child,
    /// `None` after [`Session::close`].
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    /// Kept only by [`Session::start_with`].
    stderr: Option<ChildStderr>,
    next_id: u64,
    info: Value,
}

impl Session {
    fn start(cache: &std::path::Path) -> Self {
        Self::start_in(cache, "en")
    }

    /// A server whose program language is `lang`.
    fn start_in(cache: &std::path::Path, lang: &str) -> Self {
        let mut command = Command::new(env!("CARGO_BIN_EXE_freemapper"));
        command
            .env("FREEMAPPER_LANG", lang)
            .env_remove("FREEMAPPER_UPDATE_TEST_BASE")
            .stderr(Stdio::null());
        Self::spawn(command, cache)
    }

    /// A server started by `command` (program, environment), in English, with its stderr kept
    /// for [`Session::close`].
    fn start_with(mut command: Command, cache: &std::path::Path) -> Self {
        command.env("FREEMAPPER_LANG", "en").stderr(Stdio::piped());
        Self::spawn(command, cache)
    }

    fn spawn(mut command: Command, cache: &std::path::Path) -> Self {
        let mut child = command
            .args(["mcp", "--cache-dir", cache.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let stderr = child.stderr.take();
        let mut session = Self {
            child,
            stdin,
            stdout,
            stderr,
            next_id: 1,
            info: Value::Null,
        };
        session.info = session.request(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1" }
            }),
        );
        session.send(json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        session
    }

    fn send(&mut self, message: Value) {
        let stdin = self.stdin.as_mut().unwrap();
        writeln!(stdin, "{message}").unwrap();
        stdin.flush().unwrap();
    }

    /// Sends a request and returns its result. Every line on stdout must be a JSON-RPC message.
    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let mut line = String::new();
            assert!(
                self.stdout.read_line(&mut line).unwrap() > 0,
                "server closed"
            );
            let message: Value = serde_json::from_str(&line)
                .unwrap_or_else(|e| panic!("not JSON-RPC on stdout ({e}): {line:?}"));
            assert_eq!(message["jsonrpc"], "2.0", "{line}");
            if message["id"] == id {
                return message["result"].clone();
            }
        }
    }

    /// Calls a tool; returns (is_error, text of the first content block).
    fn call(&mut self, tool: &str, arguments: Value) -> (bool, String) {
        let (error, texts) = self.call_all(tool, arguments);
        (error, texts.into_iter().next().unwrap_or_default())
    }

    /// Calls a tool; returns (is_error, the texts of all content blocks).
    fn call_all(&mut self, tool: &str, arguments: Value) -> (bool, Vec<String>) {
        let result = self.request(
            "tools/call",
            json!({ "name": tool, "arguments": arguments }),
        );
        let texts = result["content"]
            .as_array()
            .unwrap_or_else(|| panic!("no content: {result}"))
            .iter()
            .filter_map(|block| block["text"].as_str().map(str::to_string))
            .collect();
        (result["isError"] == true, texts)
    }

    /// Ends the session the way a client does, by closing stdin, and returns what the server
    /// wrote to stderr ([`Session::start_with`] only) and whether stdout held anything more.
    fn close(&mut self) -> (String, String) {
        drop(self.stdin.take());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
        while self.child.try_wait().unwrap().is_none() {
            assert!(
                std::time::Instant::now() < deadline,
                "the server did not end with its input"
            );
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        let mut rest = String::new();
        self.stdout.read_to_string(&mut rest).unwrap();
        let mut stderr = String::new();
        if let Some(mut pipe) = self.stderr.take() {
            pipe.read_to_string(&mut stderr).unwrap();
        }
        (stderr, rest)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

#[test]
fn mcp_lists_the_tools() {
    let cache = tempfile::tempdir().unwrap();
    let mut session = Session::start(cache.path());
    assert_eq!(session.info["serverInfo"]["name"], "freemapper");
    let instructions = session.info["instructions"].as_str().unwrap();
    assert!(instructions.contains("find_place"), "{instructions}");
    assert!(
        instructions.contains("check_update tells whether a newer freemapper exists; install_update installs it, only with confirm: true after the person agreed"),
        "{instructions}"
    );
    let tools = session.request("tools/list", json!({}));
    let start = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["name"] == "start_pipeline")
        .unwrap();
    assert!(
        start["inputSchema"]["properties"]["ocad"].is_object()
            && start["inputSchema"]["properties"]["pdf"].is_object()
            && start["inputSchema"]["properties"]["pdf_candidates"].is_object()
            && start["inputSchema"]["properties"]["pdf_scale_bar"].is_object()
            && start["inputSchema"]["properties"]["pdf_sources"].is_object(),
        "{start}"
    );
    let mut names: Vec<&str> = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "check_update",
            "create_project",
            "estimate_download",
            "find_place",
            "install_update",
            "list_layers",
            "open_area_selector",
            "preview_layer",
            "project_status",
            "start_pipeline"
        ]
    );
}

#[test]
fn mcp_creates_a_project_and_lists_layers() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("clara-zetkin-schule");
    let mut session = Session::start(cache.path());
    let (error, text) = session.call(
        "create_project",
        json!({ "dir": project, "geojson": FIXTURE, "title": "Clara-Zetkin-Schule" }),
    );
    assert!(!error, "{text}");
    assert!(text.contains("way/992423119"), "{text}");
    assert!(project.join("freemapper.toml").exists());

    let (error, text) = session.call("estimate_download", json!({ "dir": project }));
    assert!(!error, "{text}");
    assert!(text.contains("33421-5821"), "{text}");
    assert!(
        text.contains("\"free_megabytes\"") && text.contains("\"fits\""),
        "{text}"
    );

    let (error, text) = session.call("list_layers", json!({ "dir": project }));
    assert!(!error, "{text}");
    assert!(
        text.contains("orthophoto") && text.contains("\"ready\":false"),
        "{text}"
    );

    let (error, text) = session.call(
        "preview_layer",
        json!({ "dir": project, "layer": "orthophoto" }),
    );
    assert!(error);
    assert!(text.contains("start_pipeline"), "{text}");
}

#[test]
fn mcp_reports_tool_errors() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start(cache.path());
    let (error, text) = session.call("create_project", json!({ "dir": dir.path().join("p") }));
    assert!(error);
    assert!(
        text.contains("osm_id") && text.contains("geojson"),
        "{text}"
    );
    let (error, text) = session.call("project_status", json!({ "job_id": "job-99" }));
    assert!(error);
    assert!(text.contains("job-99"), "{text}");
    let (error, text) = session.call("start_pipeline", json!({ "dir": "projects/school" }));
    assert!(error);
    assert!(text.contains("relative"), "{text}");
    let (error, text) = session.call(
        "start_pipeline",
        json!({ "dir": dir.path().join("missing") }),
    );
    assert!(error);
    assert!(text.contains("create_project"), "{text}");
}

#[test]
fn mcp_failed_job_is_reported() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    let mut session = Session::start(cache.path());
    let (error, text) = session.call(
        "create_project",
        json!({ "dir": project, "geojson": FIXTURE }),
    );
    assert!(!error, "{text}");
    let (error, text) = session.call("start_pipeline", json!({ "dir": project, "offline": true }));
    assert!(!error, "{text}");
    let job: Value = serde_json::from_str(&text).unwrap();
    let job_id = job["job_id"].as_str().unwrap().to_string();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let status = loop {
        assert!(std::time::Instant::now() < deadline, "job still running");
        let (error, text) = session.call("project_status", json!({ "job_id": job_id }));
        assert!(!error, "{text}");
        let status: Value = serde_json::from_str(&text).unwrap();
        if status["state"] != "running" {
            break status;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    };
    assert_eq!(status["state"], "failed", "{status}");
    assert!(
        status["error"].as_str().unwrap().contains("33421-5821"),
        "{status}"
    );
    // The server keeps serving after a failed job, and the project can be started again.
    let (error, text) = session.call("list_layers", json!({ "dir": project }));
    assert!(!error, "{text}");
    let (error, text) = session.call("start_pipeline", json!({ "dir": project, "offline": true }));
    assert!(!error, "{text}");
}

#[test]
fn mcp_open_area_selector_returns_one_url_per_folder() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("new-school");
    let mut session = Session::start(cache.path());
    let arguments = json!({ "dir": project, "kind": "sprint", "open_browser": false });
    let (error, text) = session.call("open_area_selector", arguments.clone());
    assert!(!error, "{text}");
    let first: Value = serde_json::from_str(&text).unwrap();
    let url = first["url"].as_str().unwrap().to_string();
    assert!(url.starts_with("http://127.0.0.1:"), "{url}");
    let (error, text) = session.call("open_area_selector", arguments);
    assert!(!error, "{text}");
    let second: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(second["url"], url.as_str());
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let mut response = agent.get(&format!("{url}api/project")).call().unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let state: Value =
        serde_json::from_str(&response.body_mut().read_to_string().unwrap()).unwrap();
    assert_eq!(state["kind"], "sprint");
    assert_eq!(state["exists"], false);
    let (error, text) = session.call("open_area_selector", json!({ "dir": "relative/path" }));
    assert!(error);
    assert!(text.contains("relative"), "{text}");
}

#[test]
fn mcp_page_and_pipeline_do_not_overlap() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("school");
    let mut session = Session::start(cache.path());
    let (error, text) = session.call(
        "create_project",
        json!({ "dir": project, "geojson": FIXTURE }),
    );
    assert!(!error, "{text}");
    // Another spelling of the same folder gives the same page.
    let spelled = dir.path().join("school").join("..").join("school");
    let (error, text) = session.call(
        "open_area_selector",
        json!({ "dir": project, "open_browser": false }),
    );
    assert!(!error, "{text}");
    let url = serde_json::from_str::<Value>(&text).unwrap()["url"]
        .as_str()
        .unwrap()
        .to_string();
    let (error, text) = session.call(
        "open_area_selector",
        json!({ "dir": spelled, "kind": "sprint", "open_browser": false }),
    );
    assert!(!error, "{text}");
    let again: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(again["url"], url.as_str());
    assert!(
        again["note"]
            .as_str()
            .unwrap_or_default()
            .contains("already open"),
        "{again}"
    );
    let (error, text) = session.call("start_pipeline", json!({ "dir": project, "offline": true }));
    assert!(error, "{text}");
    assert!(text.contains("Done"), "{text}");
    // After Done the pipeline may start, and a new call opens a new page.
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .build()
        .into();
    let done = agent
        .post(&format!("{url}api/done"))
        .header("Content-Type", "application/json")
        .header(
            "Origin",
            url.split('/').take(3).collect::<Vec<_>>().join("/"),
        )
        .send("{}")
        .unwrap();
    assert_eq!(done.status().as_u16(), 200);
    std::thread::sleep(std::time::Duration::from_millis(300));
    let (error, text) = session.call("start_pipeline", json!({ "dir": project, "offline": true }));
    assert!(!error, "{text}");
    let (error, text) = session.call(
        "open_area_selector",
        json!({ "dir": project, "open_browser": false }),
    );
    assert!(!error, "{text}");
    let reopened: Value = serde_json::from_str(&text).unwrap();
    assert_ne!(reopened["url"], url.as_str());
}

#[test]
fn mcp_refuses_unusable_areas_and_keeps_serving() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start(cache.path());
    for (name, text) in [
        ("empty", r#"{"type":"Polygon","coordinates":[]}"#),
        (
            "bowtie",
            r#"{"type":"Polygon","coordinates":[[[13.84,52.53],[13.85,52.54],[13.85,52.53],[13.84,52.54],[13.84,52.53]]]}"#,
        ),
    ] {
        let file = dir.path().join(format!("{name}.geojson"));
        std::fs::write(&file, text).unwrap();
        let project = dir.path().join(name);
        let (error, text) =
            session.call("create_project", json!({ "dir": project, "geojson": file }));
        assert!(error, "{name}");
        assert!(text.contains("not a usable map area"), "{name}: {text}");
        assert!(!project.join("freemapper.toml").exists(), "{name}");
    }
    // The server still answers.
    let (error, _) = session.call("project_status", json!({ "job_id": "job-1" }));
    assert!(error);
}

#[test]
fn mcp_create_project_checks_the_scale_per_kind() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start(cache.path());
    let (error, text) = session.call(
        "create_project",
        json!({ "dir": dir.path().join("a"), "geojson": FIXTURE, "kind": "sprint", "scale": 3000 }),
    );
    assert!(error);
    assert!(text.contains("1:4000, 1:5000"), "{text}");
    assert!(
        text.contains("freemapper-print enlarges to 1:3000"),
        "{text}"
    );
    let (error, text) = session.call(
        "create_project",
        json!({ "dir": dir.path().join("b"), "geojson": FIXTURE, "scale": 2000 }),
    );
    assert!(error);
    assert!(text.contains("1:1000"), "{text}");
    let (error, text) = session.call(
        "create_project",
        json!({ "dir": dir.path().join("c"), "geojson": FIXTURE, "kind": "sprint", "scale": 5000 }),
    );
    assert!(!error, "{text}");
    assert!(text.contains("not in the sprint norm"), "{text}");
}
#[test]
fn mcp_pipeline_refuses_a_folder_another_run_holds() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("p");
    let mut session = Session::start(cache.path());
    let (error, text) = session.call(
        "create_project",
        json!({ "dir": project, "geojson": FIXTURE }),
    );
    assert!(!error, "{text}");
    // A terminal run holds the folder.
    let started = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let lock = project.join(".freemapper-run.lock");
    std::fs::write(
        &lock,
        format!(r#"{{"pid":1,"started":{started},"who":"cli"}}"#),
    )
    .unwrap();
    let (error, text) = session.call("start_pipeline", json!({ "dir": project, "offline": true }));
    assert!(error, "{text}");
    assert!(
        text.contains("another freemapper run is using this folder")
            || text.contains("Ein anderer freemapper-Lauf"),
        "{text}"
    );
    // Once the other run is over, the same call starts, and the job lets go of the folder.
    std::fs::remove_file(&lock).unwrap();
    let (error, text) = session.call("start_pipeline", json!({ "dir": project, "offline": true }));
    assert!(!error, "{text}");
    let job: Value = serde_json::from_str(&text).unwrap();
    let job_id = job["job_id"].as_str().unwrap().to_string();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        assert!(std::time::Instant::now() < deadline, "job still running");
        let (_, text) = session.call("project_status", json!({ "job_id": job_id }));
        let status: Value = serde_json::from_str(&text).unwrap();
        if status["state"] != "running" {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(!lock.exists(), "released when the job ended");
}

#[test]
fn mcp_errors_follow_the_program_language() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start_in(cache.path(), "de");
    let (error, text) = session.call(
        "create_project",
        json!({ "dir": dir.path().join("a"), "geojson": FIXTURE, "kind": "sprint", "scale": 3000 }),
    );
    assert!(error);
    assert!(
        text.contains("Der Maßstab 1:3000 ist für Sprintkarten nicht erlaubt"),
        "{text}"
    );
}

/// The instructions, the tool descriptions and the results come in the program's language; the
/// tool and parameter names stay as they are (ADR 0092, stage 4).
#[test]
fn mcp_describes_its_tools_in_the_program_language() {
    let cache = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut session = Session::start_in(cache.path(), "de");
    let instructions = session.info["instructions"].as_str().unwrap().to_string();
    assert!(
        instructions.starts_with("freemapper erzeugt Startprojekte"),
        "{instructions}"
    );
    let tools = session.request("tools/list", json!({}));
    let tool = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "create_project")
        .unwrap()
        .clone();
    let description = tool["description"].as_str().unwrap();
    assert!(
        description.starts_with("Legt einen Projektordner"),
        "{description}"
    );
    let osm_id = tool["inputSchema"]["properties"]["osm_id"]["description"]
        .as_str()
        .unwrap();
    assert!(
        osm_id.starts_with("OSM-Objekt des Kartengebiets"),
        "{osm_id}"
    );
    let (error, text) = session.call(
        "create_project",
        json!({ "dir": dir.path().join("a"), "geojson": FIXTURE }),
    );
    assert!(!error, "{text}");
    assert!(
        text.contains("angelegt (Schulhof, 1:1000). Als Nächstes: estimate_download"),
        "{text}"
    );
    let (error, text) = session.call("project_status", json!({ "job_id": "job-9" }));
    assert!(error);
    assert!(text.starts_with("Keinen Job \"job-9\" gefunden"), "{text}");
}

/// A build from source neither checks nor installs; both tools say how to update instead, as a
/// normal result (spec 3.6).
#[test]
fn mcp_update_tools_in_a_source_build_say_how_to_update() {
    let cache = tempfile::tempdir().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_freemapper"));
    command.env_remove("FREEMAPPER_UPDATE_TEST_BASE");
    let mut session = Session::start_with(command, cache.path());
    let source_build = "This freemapper was built from source. Update it with git pull and cargo build --release --locked.";
    for (tool, arguments) in [
        ("check_update", json!({})),
        ("install_update", json!({ "confirm": true })),
        ("install_update", json!({})),
    ] {
        let (error, texts) = session.call_all(tool, arguments);
        assert!(!error, "{tool}: {texts:?}");
        assert_eq!(texts, [source_build], "{tool}");
    }
    let (stderr, rest) = session.close();
    assert_eq!((stderr.as_str(), rest.as_str()), ("", ""));
    assert!(!cache.path().join("update-check.json").exists());
}

/// The update tools and the daily check of the server against the stub of `tests/support`
/// (spec 3.7). Only a build with the `test-stub` feature reaches them; the module name puts them
/// under CI's `update` filter.
#[cfg(all(feature = "test-stub", debug_assertions))]
mod update {
    use std::path::Path;
    use std::process::Command;
    use std::time::{Duration, Instant};

    use serde_json::json;

    use super::Session;
    use super::support::{MANIFEST_PATH, UpdateStub, copied_program};

    /// `program` as a server talking to `stub`, without `CI` and the update switches of the
    /// environment, with its settings in `dir`.
    fn checking(program: &Path, stub: &UpdateStub, dir: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .env_remove("CI")
            .env_remove("FREEMAPPER_UPDATE")
            .env("FREEMAPPER_CONFIG_DIR", dir.join("config"))
            .env("FREEMAPPER_UPDATE_TEST_BASE", &stub.base);
        command
    }

    fn program() -> &'static Path {
        Path::new(env!("CARGO_BIN_EXE_freemapper"))
    }

    fn note() -> String {
        format!(
            "Note for the person: freemapper 99.0.0 is available (this is {}). Ask them before calling install_update.",
            env!("CARGO_PKG_VERSION")
        )
    }

    /// Waits until the daily check wrote its record into `cache`.
    fn wait_for_record(cache: &Path) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !cache.join("update-check.json").exists() {
            assert!(Instant::now() < deadline, "no check record");
            std::thread::sleep(Duration::from_millis(20));
        }
        // The note is set just after the record is written.
        std::thread::sleep(Duration::from_millis(300));
    }

    /// A tool that answers at once without the network: its result, as (is_error, texts).
    fn cheap_call(session: &mut Session) -> (bool, Vec<String>) {
        session.call_all("project_status", json!({ "job_id": "job-1" }))
    }

    #[test]
    fn check_update_reports_available_with_stub() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        let stub = UpdateStub::with_fixture();
        // In mode `off` the server does not check by itself, so only the tool asks.
        let mut command = checking(program(), &stub, dir.path());
        command.env("FREEMAPPER_UPDATE", "off");
        let mut session = Session::start_with(command, &cache);
        let (error, texts) = session.call_all("check_update", json!({}));
        assert!(!error, "{texts:?}");
        let current = env!("CARGO_PKG_VERSION");
        assert_eq!(
            texts,
            [format!(
                "freemapper 99.0.0 is available (you have {current}), released 2026-11-20.\n\
                 Release notes: https://github.com/HenryJobst/freemapper/releases/tag/v99.0.0\n\
                 Download: 11.8 MB"
            )]
        );
        assert_eq!(
            stub.paths(),
            [
                MANIFEST_PATH.to_string(),
                format!("{MANIFEST_PATH}.minisig")
            ]
        );
        // The tool checks; it neither records nor installs.
        assert!(!cache.join("update-check.json").exists());
        assert!(!stub.asked_for_an_asset());
    }

    #[test]
    fn install_update_needs_confirm_true() {
        let dir = tempfile::tempdir().unwrap();
        let stub = UpdateStub::with_fixture();
        let mut command = checking(program(), &stub, dir.path());
        command.env("FREEMAPPER_UPDATE", "off");
        let mut session = Session::start_with(command, &dir.path().join("cache"));
        for arguments in [json!({}), json!({ "confirm": false })] {
            let (error, texts) = session.call_all("install_update", arguments.clone());
            assert!(error, "{arguments}: {texts:?}");
            assert_eq!(
                texts,
                [
                    "nothing was installed; ask the person first and call install_update with confirm: true once they agree"
                ],
                "{arguments}"
            );
        }
        assert!(stub.paths().is_empty(), "{:?}", stub.paths());
    }

    #[test]
    fn first_tool_result_carries_the_note_once() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        let stub = UpdateStub::with_fixture();
        let mut session = Session::start_with(checking(program(), &stub, dir.path()), &cache);
        wait_for_record(&cache);
        let (error, texts) = cheap_call(&mut session);
        assert!(error, "{texts:?}");
        // The tool's own text first, then the note.
        assert_eq!(texts.len(), 2, "{texts:?}");
        assert!(texts[0].contains("job-1"), "{texts:?}");
        assert_eq!(texts[1], note());
        for _ in 0..3 {
            let (_, texts) = cheap_call(&mut session);
            assert_eq!(texts.len(), 1, "{texts:?}");
        }
        let (stderr, rest) = session.close();
        assert_eq!((stderr.as_str(), rest.as_str()), ("", ""));
        assert_eq!(stub.paths().len(), 2);

        // A server started later the same day does not ask again; the record tells it.
        let mut session = Session::start_with(checking(program(), &stub, dir.path()), &cache);
        let (_, texts) = cheap_call(&mut session);
        assert_eq!(texts.get(1), Some(&note()), "{texts:?}");
        let (_, texts) = cheap_call(&mut session);
        assert_eq!(texts.len(), 1, "{texts:?}");
        assert_eq!(stub.paths().len(), 2);

        // check_update gives the news itself; no note follows it.
        std::fs::remove_file(cache.join("update-check.json")).unwrap();
        let mut session = Session::start_with(checking(program(), &stub, dir.path()), &cache);
        let (_, texts) = session.call_all("check_update", json!({}));
        assert_eq!(texts.len(), 1, "{texts:?}");
        wait_for_record(&cache);
        let (_, texts) = cheap_call(&mut session);
        assert_eq!(texts.len(), 1, "{texts:?}");
    }

    /// A check that is not done when the first result goes out does not hold it up; its note
    /// goes on a later result.
    #[test]
    fn a_late_check_puts_the_note_on_a_later_result() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        // Two requests of 700 ms each, well inside the 3 s of the check.
        let stub = UpdateStub::with_fixture_after(Duration::from_millis(700));
        let mut session = Session::start_with(checking(program(), &stub, dir.path()), &cache);
        let started = Instant::now();
        let (_, texts) = cheap_call(&mut session);
        assert!(started.elapsed() < Duration::from_millis(500));
        assert_eq!(texts.len(), 1, "{texts:?}");
        wait_for_record(&cache);
        let (_, texts) = cheap_call(&mut session);
        assert_eq!(texts.get(1), Some(&note()), "{texts:?}");
        let (stderr, rest) = session.close();
        assert_eq!((stderr.as_str(), rest.as_str()), ("", ""));
    }

    /// D14: mode `install` does not apply to the server; it tells and waits for install_update.
    #[test]
    fn mcp_never_installs_in_install_mode() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("bin");
        std::fs::create_dir(&folder).unwrap();
        let program = copied_program(&folder);
        let cache = dir.path().join("cache");
        let stub = UpdateStub::with_fixture();
        let mut command = checking(&program, &stub, dir.path());
        command.env("FREEMAPPER_UPDATE", "install");
        let mut session = Session::start_with(command, &cache);
        wait_for_record(&cache);
        let (_, texts) = cheap_call(&mut session);
        assert_eq!(texts.get(1), Some(&note()), "{texts:?}");
        std::thread::sleep(Duration::from_millis(500));
        let (stderr, rest) = session.close();
        assert_eq!((stderr.as_str(), rest.as_str()), ("", ""));
        assert!(!stub.asked_for_an_asset(), "{:?}", stub.paths());
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 1);
    }

    /// The server's stdout carries only the protocol, and its stderr stays empty, also while it
    /// checks and knows an update: the usual session of an agent, line by line.
    #[test]
    fn nothing_reaches_stdout_outside_the_protocol() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("cache");
        let project = dir.path().join("p");
        let stub = UpdateStub::with_fixture();
        let mut session = Session::start_with(checking(program(), &stub, dir.path()), &cache);
        let tools = session.request("tools/list", json!({}));
        assert_eq!(tools["tools"].as_array().unwrap().len(), 10);
        let (error, texts) = session.call_all(
            "create_project",
            json!({ "dir": project, "geojson": super::FIXTURE }),
        );
        assert!(!error, "{texts:?}");
        wait_for_record(&cache);
        for (tool, arguments) in [
            ("estimate_download", json!({ "dir": project })),
            ("list_layers", json!({ "dir": project })),
            ("check_update", json!({})),
            ("install_update", json!({})),
        ] {
            session.call_all(tool, arguments);
        }
        let (stderr, rest) = session.close();
        assert_eq!(stderr, "");
        assert_eq!(rest, "");
    }

    /// The whole install through the tool against the stub (see `installable_release`): the
    /// result says the server must be restarted. Without the `minisign` CLI the test passes.
    #[test]
    #[cfg(unix)]
    fn install_update_with_confirm_installs_and_asks_for_a_restart() {
        let dir = tempfile::tempdir().unwrap();
        let folder = dir.path().join("bin");
        let work = dir.path().join("work");
        std::fs::create_dir(&folder).unwrap();
        std::fs::create_dir(&work).unwrap();
        let program = copied_program(&folder);
        let Some(stub) = super::support::installable_release(&work) else {
            return;
        };
        let mut command = checking(&program, &stub, dir.path());
        command.env("FREEMAPPER_UPDATE", "off");
        let mut session = Session::start_with(command, &dir.path().join("cache"));
        let (error, texts) = session.call_all("install_update", json!({ "confirm": true }));
        assert!(!error, "{texts:?}");
        let current = env!("CARGO_PKG_VERSION");
        assert_eq!(
            texts,
            [format!(
                "freemapper updated from {current} to 99.0.0. Release notes: https://github.com/HenryJobst/freemapper/releases/tag/v99.0.0\n\
                 A running MCP server keeps the old version until the AI assistant restarts it."
            )]
        );
        let (stderr, rest) = session.close();
        assert_eq!((stderr.as_str(), rest.as_str()), ("", ""));
        let version = Command::new(&program).arg("--version").output().unwrap();
        assert_eq!(
            String::from_utf8(version.stdout).unwrap().trim(),
            "freemapper 99.0.0"
        );
    }
}
