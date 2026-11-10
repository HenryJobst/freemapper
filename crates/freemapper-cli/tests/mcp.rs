//! The MCP server spoken to over stdio, as an agent would.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/fixtures/clara-zetkin-schule.geojson"
);

struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    info: Value,
}

impl Session {
    fn start(cache: &std::path::Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_freemapper"))
            .args(["mcp", "--cache-dir", cache.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut session = Self {
            child,
            stdin,
            stdout,
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
        writeln!(self.stdin, "{message}").unwrap();
        self.stdin.flush().unwrap();
    }

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
            let message: Value = serde_json::from_str(&line).unwrap();
            if message["id"] == id {
                return message["result"].clone();
            }
        }
    }

    /// Calls a tool; returns (is_error, text of the first content block).
    fn call(&mut self, tool: &str, arguments: Value) -> (bool, String) {
        let result = self.request(
            "tools/call",
            json!({ "name": tool, "arguments": arguments }),
        );
        let text = result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        (result["isError"] == true, text)
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
    assert!(
        session.info["instructions"]
            .as_str()
            .unwrap()
            .contains("find_place")
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
            "create_project",
            "estimate_download",
            "find_place",
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
