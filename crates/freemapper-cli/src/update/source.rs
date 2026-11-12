//! The network of the self-update: an [`UpdateSource`] over its own HTTP agent, which follows
//! redirects by hand so that every hop passes the host rule (spec 3.8, D15).

use std::fs::File;
use std::io::{Read as _, Write as _};
use std::sync::Mutex;
use std::time::Duration;

use freemapper::fetch::{USER_AGENT, error_text, guarded_agent, redirect_target as resolve};
use freemapper_update::{ALLOWED_HOSTS, UpdateError, UpdateSource, check_url};

/// Redirects followed per request; GitHub needs one (release URL to asset host).
const MAX_REDIRECTS: u32 = 3;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// One request of a manifest or signature (each redirect hop is its own), body included.
const GET_TIMEOUT: Duration = Duration::from_secs(60);
/// The whole of a binary download: 30 MB at 200 kbit/s take about 20 minutes.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// An [`UpdateSource`] over HTTPS. Its agent follows no redirects: [`HttpUpdateSource::open`]
/// follows them itself, after each target passed [`check_url`]. The agent connects to public
/// addresses only (`freemapper::fetch::GuardedResolver`), for every hop.
pub struct HttpUpdateSource {
    agent: ureq::Agent,
    /// The URLs asked for whose answer, after any redirects, was HTTP 404.
    not_found: Mutex<Vec<String>>,
    /// Tests: the stub server that answers in place of the GitHub hosts.
    #[cfg(any(test, all(debug_assertions, feature = "test-stub")))]
    stub: Option<String>,
}

impl HttpUpdateSource {
    pub fn new() -> Self {
        Self::with_agent(guarded_agent(Self::config()))
    }

    fn config() -> ureq::config::Config {
        ureq::Agent::config_builder()
            .max_redirects(0)
            // Redirects and error answers arrive as responses, so their status can be named.
            .http_status_as_error(false)
            .timeout_connect(Some(CONNECT_TIMEOUT))
            .timeout_global(Some(GET_TIMEOUT))
            // The bytes as they are: the hashes in the manifest are those of the files.
            .accept_encoding("identity")
            .build()
    }

    fn with_agent(agent: ureq::Agent) -> Self {
        Self {
            agent,
            not_found: Mutex::new(Vec::new()),
            #[cfg(any(test, all(debug_assertions, feature = "test-stub")))]
            stub: None,
        }
    }

    /// A source whose requests to `https://github.com/...` and
    /// `https://release-assets.githubusercontent.com/...` go to `base` (`http://127.0.0.1:<port>`)
    /// with the same path. The host rule still applies to every URL before it is mapped. Exists in
    /// unit tests and in debug builds with the `test-stub` feature (the CLI tests), never in a
    /// release build. Its agent has no address check, as the stub is on this computer.
    #[cfg(any(test, all(debug_assertions, feature = "test-stub")))]
    pub fn stub(base: &str) -> Self {
        Self {
            stub: Some(base.trim_end_matches('/').to_string()),
            ..Self::with_agent(Self::config().into())
        }
    }

    /// Whether a GET of `url` through this source ended in HTTP 404: the file is not there. GitHub
    /// answers a missing file of the latest release with a redirect to the release's own URL and a
    /// 404 there, so any hop counts.
    pub fn was_not_found(&self, url: &str) -> bool {
        self.not_found
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .any(|asked| asked == url)
    }

    /// The address a request for `url` goes to: `url` itself, in tests the stub's.
    fn target(&self, url: &str) -> String {
        #[cfg(any(test, all(debug_assertions, feature = "test-stub")))]
        if let Some(base) = &self.stub {
            for host in [
                "https://github.com",
                "https://release-assets.githubusercontent.com",
            ] {
                if let Some(rest) = url.strip_prefix(host) {
                    return format!("{base}{rest}");
                }
            }
        }
        url.to_string()
    }

    /// The `200` answer to a GET of `url`, after at most [`MAX_REDIRECTS`] redirects, with the URL
    /// that gave it. `url` and every redirect target must pass [`check_url`]; `timeout` covers
    /// each request with its body.
    fn open(
        &self,
        url: &str,
        timeout: Duration,
    ) -> Result<(String, ureq::http::Response<ureq::Body>), UpdateError> {
        check_url(url)?;
        let mut current = url.to_string();
        let mut redirects = 0;
        loop {
            let response = self
                .agent
                .get(self.target(&current))
                .header("User-Agent", USER_AGENT)
                .config()
                .timeout_global(Some(timeout))
                .build()
                .call()
                .map_err(|e| download_error(&current, &error_text(&e)))?;
            let status = response.status().as_u16();
            if status == 200 {
                return Ok((current, response));
            }
            if status == 404 {
                self.not_found
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(url.to_string());
            }
            if !matches!(status, 301 | 302 | 303 | 307 | 308) {
                return Err(download_error(&current, &format!("HTTP {status}")));
            }
            if redirects == MAX_REDIRECTS {
                return Err(download_error(
                    url,
                    &format!("more than {MAX_REDIRECTS} redirects"),
                ));
            }
            redirects += 1;
            // Header values stay out of the messages: they come from the server.
            let location = response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| {
                    download_error(&current, &format!("redirect (HTTP {status}) to nowhere"))
                })?;
            let next = resolve(&current, location);
            check_url(&next).map_err(|_| {
                UpdateError::Url(format!(
                    "{}: redirect (HTTP {status}) to a URL outside https://{}",
                    bare_url(&current),
                    ALLOWED_HOSTS.join(", https://")
                ))
            })?;
            current = next;
        }
    }
}

impl Default for HttpUpdateSource {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateSource for HttpUpdateSource {
    fn get(&self, url: &str, max_bytes: u64) -> Result<Vec<u8>, UpdateError> {
        let (url, mut response) = self.open(url, GET_TIMEOUT)?;
        // One byte more than allowed, so an oversized answer shows.
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(max_bytes.saturating_add(1))
            .read_to_end(&mut bytes)
            .map_err(|e| download_error(&url, &e.to_string()))?;
        if bytes.len() as u64 > max_bytes {
            return Err(download_error(
                &url,
                &format!("the answer is larger than {max_bytes} bytes"),
            ));
        }
        Ok(bytes)
    }

    fn download_to(&self, url: &str, size: u64, file: &mut File) -> Result<(), UpdateError> {
        let (url, mut response) = self.open(url, DOWNLOAD_TIMEOUT)?;
        let longer = || download_error(&url, &format!("longer than {size} bytes"));
        let announced = response
            .headers()
            .get("content-length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok());
        if announced.is_some_and(|announced| announced > size) {
            return Err(longer());
        }
        let mut reader = response.body_mut().as_reader();
        let mut buffer = vec![0u8; 1 << 16];
        let mut done = 0u64;
        loop {
            let n = reader
                .read(&mut buffer)
                .map_err(|e| download_error(&url, &e.to_string()))?;
            if n == 0 {
                break;
            }
            if done + n as u64 > size {
                return Err(longer());
            }
            file.write_all(&buffer[..n])?;
            done += n as u64;
        }
        if done < size {
            return Err(download_error(
                &url,
                &format!("ended after {done} of {size} bytes"),
            ));
        }
        file.flush()?;
        Ok(())
    }
}

/// `url` without its query and fragment: the asset host's signed links carry tokens there, which
/// mean nothing to a reader.
fn bare_url(url: &str) -> &str {
    url.split(['?', '#']).next().unwrap_or(url)
}

/// [`UpdateError::Download`] for `url` with `what` went wrong.
fn download_error(url: &str, what: &str) -> UpdateError {
    UpdateError::Download(format!("{}: {what}", bare_url(url)))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashMap;
    use std::io::{BufRead as _, BufReader, Seek as _};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    const MANIFEST_URL: &str =
        "https://github.com/HenryJobst/freemapper/releases/latest/download/freemapper-update.json";
    const MANIFEST_PATH: &str =
        "/HenryJobst/freemapper/releases/latest/download/freemapper-update.json";

    /// A canned answer of the stub server.
    struct Answer {
        status: u16,
        location: Option<String>,
        body: Vec<u8>,
    }

    fn ok(body: &[u8]) -> Answer {
        Answer {
            status: 200,
            location: None,
            body: body.to_vec(),
        }
    }

    fn redirect(status: u16, location: &str) -> Answer {
        Answer {
            status,
            location: Some(location.to_string()),
            body: Vec::new(),
        }
    }

    /// A server on 127.0.0.1 that answers each path with its canned answer (404 otherwise) and
    /// records each request as `<path> <user agent>`, and its `Accept-Encoding` apart.
    struct Stub {
        base: String,
        requests: Arc<Mutex<Vec<String>>>,
        encodings: Arc<Mutex<Vec<String>>>,
    }

    impl Stub {
        fn start(answers: Vec<(&str, Answer)>) -> Self {
            let answers: HashMap<String, Answer> = answers
                .into_iter()
                .map(|(path, answer)| (path.to_string(), answer))
                .collect();
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind the stub");
            let base = format!("http://{}", listener.local_addr().expect("stub address"));
            let requests = Arc::new(Mutex::new(Vec::new()));
            let seen = Arc::clone(&requests);
            let encodings = Arc::new(Mutex::new(Vec::new()));
            let seen_encodings = Arc::clone(&encodings);
            std::thread::spawn(move || {
                for stream in listener.incoming() {
                    let Ok(mut stream) = stream else { continue };
                    let mut reader = BufReader::new(stream.try_clone().expect("clone the stream"));
                    let mut request_line = String::new();
                    if reader.read_line(&mut request_line).is_err() {
                        continue;
                    }
                    let path = request_line
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("")
                        .to_string();
                    let mut agent = String::new();
                    let mut encoding = String::new();
                    loop {
                        let mut line = String::new();
                        if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                            break;
                        }
                        if let Some((name, value)) = line.split_once(':')
                            && name.eq_ignore_ascii_case("user-agent")
                        {
                            agent = value.trim().to_string();
                        }
                        if let Some((name, value)) = line.split_once(':')
                            && name.eq_ignore_ascii_case("accept-encoding")
                        {
                            encoding = value.trim().to_string();
                        }
                    }
                    seen.lock().unwrap().push(format!("{path} {agent}"));
                    seen_encodings.lock().unwrap().push(encoding);
                    let fallback = Answer {
                        status: 404,
                        location: None,
                        body: Vec::new(),
                    };
                    let answer = answers.get(&path).unwrap_or(&fallback);
                    let mut head = format!(
                        "HTTP/1.1 {} Stub\r\nContent-Length: {}\r\nConnection: close\r\n",
                        answer.status,
                        answer.body.len()
                    );
                    if let Some(location) = &answer.location {
                        head.push_str(&format!("Location: {location}\r\n"));
                    }
                    head.push_str("\r\n");
                    let _ = stream.write_all(head.as_bytes());
                    let _ = stream.write_all(&answer.body);
                }
            });
            Self {
                base,
                requests,
                encodings,
            }
        }

        fn source(&self) -> HttpUpdateSource {
            HttpUpdateSource::stub(&self.base)
        }

        fn paths(&self) -> Vec<String> {
            self.requests
                .lock()
                .unwrap()
                .iter()
                .map(|r| r.split(' ').next().unwrap_or("").to_string())
                .collect()
        }
    }

    #[test]
    fn stub_serves_manifest_and_follows_one_redirect() {
        let stub = Stub::start(vec![
            (
                MANIFEST_PATH,
                redirect(
                    302,
                    "https://release-assets.githubusercontent.com/m/1?sig=x",
                ),
            ),
            ("/m/1?sig=x", ok(b"{\"format\":1}")),
        ]);
        let body = stub.source().get(MANIFEST_URL, 64 * 1024).unwrap();
        assert_eq!(body, b"{\"format\":1}");
        let requests = stub.requests.lock().unwrap().clone();
        assert_eq!(
            requests,
            [
                format!("{MANIFEST_PATH} {}", freemapper::fetch::USER_AGENT),
                format!("/m/1?sig=x {}", freemapper::fetch::USER_AGENT),
            ]
        );
    }

    #[test]
    fn requests_ask_for_the_bytes_as_they_are() {
        let stub = Stub::start(vec![("/m.json", ok(b"{}")), ("/asset.gz", ok(&[1; 10]))]);
        let source = stub.source();
        source.get("https://github.com/m.json", 1024).unwrap();
        let mut file = tempfile::tempfile().unwrap();
        source
            .download_to("https://github.com/asset.gz", 10, &mut file)
            .unwrap();
        assert_eq!(*stub.encodings.lock().unwrap(), ["identity", "identity"]);
    }

    #[test]
    fn messages_leave_out_query_and_fragment() {
        let stub = Stub::start(vec![(
            "/asset.gz",
            redirect(
                302,
                "https://release-assets.githubusercontent.com/blob?sig=secret&se=1#part",
            ),
        )]);
        let error = stub
            .source()
            .get("https://github.com/asset.gz", 1024)
            .unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("https://release-assets.githubusercontent.com/blob: HTTP 404"),
            "{message}"
        );
        assert!(
            !message.contains("sig=") && !message.contains('#'),
            "{message}"
        );
        assert_eq!(
            bare_url("https://github.com/a?b=c#d"),
            "https://github.com/a"
        );
        assert_eq!(bare_url("https://github.com/a#d?x"), "https://github.com/a");
    }

    #[test]
    fn redirect_to_foreign_host_is_refused() {
        for location in [
            "https://evil.example/m.json",
            "http://github.com/m.json",
            "https://github.com:8443/m.json",
            "//evil.example/m.json",
        ] {
            let stub = Stub::start(vec![(MANIFEST_PATH, redirect(302, location))]);
            let error = stub.source().get(MANIFEST_URL, 64 * 1024).unwrap_err();
            assert!(matches!(error, UpdateError::Url(_)), "{location}: {error}");
            assert!(
                !error.to_string().contains(location),
                "no header values in the message: {error}"
            );
            assert_eq!(stub.paths(), [MANIFEST_PATH]);
        }
    }

    /// The agent of the source connects to public addresses only: asked for a server on this
    /// computer, it refuses before a connection is opened. The source of the tests has no such
    /// check, or it could not reach its stub.
    #[test]
    fn the_agent_of_the_source_refuses_this_computer() {
        let stub = Stub::start(vec![(MANIFEST_PATH, ok(b"{}"))]);
        let url = format!("{}{MANIFEST_PATH}", stub.base);
        let error = HttpUpdateSource::new().agent.get(&url).call().unwrap_err();
        let text = error_text(&error);
        assert!(text.contains("this computer"), "{text}");
        assert!(!text.starts_with("other"), "{text}");
        assert!(stub.paths().is_empty());
        assert_eq!(stub.source().agent.get(&url).call().unwrap().status(), 200);
    }

    #[test]
    fn a_url_outside_the_host_rule_is_not_requested() {
        let stub = Stub::start(vec![]);
        let error = stub
            .source()
            .get("https://example.com/freemapper-update.json", 64 * 1024)
            .unwrap_err();
        assert!(matches!(error, UpdateError::Url(_)), "{error}");
        assert!(stub.paths().is_empty());
    }

    #[test]
    fn fourth_redirect_is_refused() {
        // Three hops, relative locations of each kind: allowed.
        let stub = Stub::start(vec![
            ("/a/r0", redirect(301, "/a/r1")),
            ("/a/r1", redirect(307, "r2")),
            ("/a/r2", redirect(308, "//github.com/a/r3")),
            ("/a/r3", ok(b"done")),
            ("/b/s0", redirect(302, "/b/s1")),
            ("/b/s1", redirect(303, "/b/s2")),
            ("/b/s2", redirect(302, "/b/s3")),
            ("/b/s3", redirect(302, "/b/s4")),
            ("/b/s4", ok(b"too far")),
        ]);
        let source = stub.source();
        assert_eq!(
            source.get("https://github.com/a/r0", 1024).unwrap(),
            b"done"
        );
        let error = source.get("https://github.com/b/s0", 1024).unwrap_err();
        assert!(
            matches!(&error, UpdateError::Download(m) if m.contains("redirect")),
            "{error}"
        );
        assert!(!stub.paths().contains(&"/b/s4".to_string()));
    }

    #[test]
    fn answer_over_cap_is_refused() {
        let stub = Stub::start(vec![("/big", ok(&[b'x'; 100]))]);
        let source = stub.source();
        assert_eq!(
            source.get("https://github.com/big", 100).unwrap().len(),
            100
        );
        let error = source.get("https://github.com/big", 99).unwrap_err();
        assert!(
            matches!(&error, UpdateError::Download(m) if m.contains("https://github.com/big")),
            "{error}"
        );
    }

    #[test]
    fn an_error_status_names_url_and_status() {
        let stub = Stub::start(vec![("/gone", redirect(302, "/missing"))]);
        let error = stub
            .source()
            .get("https://github.com/gone", 1024)
            .unwrap_err();
        let UpdateError::Download(message) = error else {
            panic!("{error}");
        };
        assert!(message.contains("https://github.com/missing"), "{message}");
        assert!(message.contains("404"), "{message}");
    }

    #[test]
    fn a_404_after_a_redirect_is_not_found_too() {
        let stub = Stub::start(vec![
            ("/gone", redirect(302, "/missing")),
            (
                "/broken",
                Answer {
                    status: 500,
                    location: None,
                    body: Vec::new(),
                },
            ),
            ("/here", ok(b"x")),
        ]);
        let source = stub.source();
        for url in ["https://github.com/missing", "https://github.com/gone"] {
            source.get(url, 1024).unwrap_err();
            assert!(source.was_not_found(url), "{url}");
        }
        source.get("https://github.com/broken", 1024).unwrap_err();
        source.get("https://github.com/here", 1024).unwrap();
        for url in [
            "https://github.com/broken",
            "https://github.com/here",
            "https://github.com/never-asked",
        ] {
            assert!(!source.was_not_found(url), "{url}");
        }
    }

    #[test]
    fn download_writes_exactly_the_size() {
        let body: Vec<u8> = (0..=255).cycle().take(200_000).collect();
        let stub = Stub::start(vec![
            (
                "/asset.gz",
                redirect(302, "https://release-assets.githubusercontent.com/blob"),
            ),
            ("/blob", ok(&body)),
        ]);
        let mut file = tempfile::tempfile().unwrap();
        stub.source()
            .download_to("https://github.com/asset.gz", 200_000, &mut file)
            .unwrap();
        let mut written = Vec::new();
        file.rewind().unwrap();
        file.read_to_end(&mut written).unwrap();
        assert_eq!(written, body);
    }

    #[test]
    fn download_refuses_more_or_fewer_bytes() {
        let stub = Stub::start(vec![("/asset.gz", ok(&[7; 1000]))]);
        let source = stub.source();
        for (size, reason) in [
            (999, "longer than 999 bytes"),
            (1001, "ended after 1000 of 1001"),
        ] {
            let mut file = tempfile::tempfile().unwrap();
            let error = source
                .download_to("https://github.com/asset.gz", size, &mut file)
                .unwrap_err();
            assert!(
                matches!(&error, UpdateError::Download(m)
                    if m.contains("https://github.com/asset.gz") && m.contains(reason)),
                "{size}: {error}"
            );
        }
    }

    #[test]
    fn locations_resolve_against_the_current_url() {
        let base = "https://github.com/o/r/releases/latest/download/x.json?a=1#f";
        for (location, expected) in [
            (
                "https://release-assets.githubusercontent.com/p",
                "https://release-assets.githubusercontent.com/p",
            ),
            (
                "//objects.githubusercontent.com/q",
                "https://objects.githubusercontent.com/q",
            ),
            ("/o/r/y.json", "https://github.com/o/r/y.json"),
            (
                "y.json",
                "https://github.com/o/r/releases/latest/download/y.json",
            ),
        ] {
            assert_eq!(resolve(base, location), expected, "{location}");
        }
        assert_eq!(resolve("https://github.com", "z"), "https://github.com/z");
    }
}
