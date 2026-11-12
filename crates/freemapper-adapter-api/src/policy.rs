//! Where a request may go: the URL rules for adapters (ADR 0082, spec 3.8).
//!
//! A manifest is untrusted input. Its URLs are `https` on port 443, name a host by its DNS name
//! and stay on the hosts the manifest declares. The rules are parsed by hand: no URL crate.

use std::collections::BTreeSet;
use std::fmt;

/// A URL or a host a policy refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyError {
    /// The stable code of the rule: `policy.url`, `policy.host`, `policy.host-not-declared`.
    pub code: &'static str,
    /// The URL or host, without query and fragment (they may carry signed parts).
    pub subject: String,
    /// What is wrong with it.
    pub reason: &'static str,
}

impl PolicyError {
    fn new(code: &'static str, subject: &str, reason: &'static str) -> Self {
        let cut = subject.find(['?', '#']).unwrap_or(subject.len());
        Self {
            code,
            subject: subject[..cut].to_string(),
            reason,
        }
    }
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.subject, self.reason)
    }
}

impl std::error::Error for PolicyError {}

/// Checks that `host` is a DNS name a request may go to and returns it in lower case: at least
/// two labels of letters, digits and hyphens, no IP literal, no `localhost`, no trailing dot, no
/// wildcard. A name outside ASCII is written in its `xn--` form.
pub fn dns_name(host: &str) -> Result<String, PolicyError> {
    let refuse = |reason| Err(PolicyError::new("policy.host", host, reason));
    if host.is_empty() || host.len() > 253 {
        return refuse("not a host name");
    }
    if !host.is_ascii() {
        return refuse("a name outside ASCII must be written in its xn-- form");
    }
    let name = host.to_ascii_lowercase();
    let labels: Vec<&str> = name.split('.').collect();
    if labels.iter().any(|label| {
        label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || !label
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    }) {
        return refuse("not a DNS name");
    }
    let last = labels.last().copied().unwrap_or_default();
    if labels.len() < 2 || last == "localhost" {
        return refuse("a name of this computer or without a domain");
    }
    // `127.0.0.1`, `0x7f.1`: an address ends in a number, a DNS name in a top-level domain.
    if last.bytes().all(|b| b.is_ascii_digit()) || last.starts_with("0x") {
        return refuse("an IP address, not a DNS name");
    }
    Ok(name)
}

/// The host of `url` in lower case, when the URL is one a request may go to: `https`, no user
/// info, port 443 or none, a DNS name as [`dns_name`] accepts it, no control character, blank or
/// backslash.
pub fn host_of(url: &str) -> Result<String, PolicyError> {
    let refuse = |reason| Err(PolicyError::new("policy.url", url, reason));
    if url
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
    {
        return refuse("unexpected character");
    }
    let rest = match url.get(..8) {
        Some(scheme) if scheme.eq_ignore_ascii_case("https://") => &url[8..],
        _ => return refuse("not https"),
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if authority.contains('@') {
        return refuse("user info");
    }
    if authority.starts_with('[') {
        return refuse("an IP address, not a DNS name");
    }
    let (host, port) = match authority.split_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (authority, None),
    };
    if port.is_some_and(|port| port != "443") {
        return refuse("a port other than 443");
    }
    dns_name(host).map_err(|e| PolicyError::new("policy.url", url, e.reason))
}

/// The hosts the requests of one adapter (or of the program itself) may go to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlPolicy {
    hosts: BTreeSet<String>,
    /// How many redirects a request may follow; each target passes [`UrlPolicy::check_url`].
    pub max_redirects: u8,
}

impl UrlPolicy {
    /// The redirects a request follows unless the policy says otherwise.
    pub const MAX_REDIRECTS: u8 = 3;

    /// A policy for `hosts`, each a DNS name as [`dns_name`] accepts it.
    pub fn new<'a>(hosts: impl IntoIterator<Item = &'a str>) -> Result<Self, PolicyError> {
        Ok(Self {
            hosts: hosts.into_iter().map(dns_name).collect::<Result<_, _>>()?,
            max_redirects: Self::MAX_REDIRECTS,
        })
    }

    /// The hosts of the policy, in lower case, sorted.
    pub fn hosts(&self) -> impl Iterator<Item = &str> {
        self.hosts.iter().map(String::as_str)
    }

    /// Whether `host` (any case) is one of the hosts.
    pub fn allows_host(&self, host: &str) -> bool {
        self.hosts.contains(&host.to_ascii_lowercase())
    }

    /// Accepts `url` when [`host_of`] accepts it and its host is one of the policy's.
    pub fn check_url(&self, url: &str) -> Result<(), PolicyError> {
        let host = host_of(url)?;
        if self.hosts.contains(&host) {
            Ok(())
        } else {
            Err(PolicyError::new(
                "policy.host-not-declared",
                url,
                "the host is not one of the declared hosts",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_are_https_on_443_with_a_dns_name() {
        for (url, host) in [
            ("https://geodienste.hamburg.de/wms", "geodienste.hamburg.de"),
            ("https://Geodienste.Hamburg.DE", "geodienste.hamburg.de"),
            ("https://api.hamburg.de:443/x?y=1#z", "api.hamburg.de"),
            ("HTTPS://xn--mnchen-3ya.example/x", "xn--mnchen-3ya.example"),
        ] {
            assert_eq!(host_of(url).as_deref(), Ok(host), "{url}");
        }
        for (url, reason) in [
            ("http://geodienste.hamburg.de/wms", "not https"),
            ("ftp://geodienste.hamburg.de/wms", "not https"),
            ("file:///etc/passwd", "not https"),
            ("https://user@api.hamburg.de/x", "user info"),
            ("https://user:pw@api.hamburg.de/x", "user info"),
            ("https://api.hamburg.de:8443/x", "a port other than 443"),
            ("https://api.hamburg.de:/x", "a port other than 443"),
            ("https://127.0.0.1/x", "an IP address, not a DNS name"),
            ("https://[::1]/x", "an IP address, not a DNS name"),
            ("https://0x7f.1/x", "an IP address, not a DNS name"),
            (
                "https://2130706433/x",
                "a name of this computer or without a domain",
            ),
            (
                "https://localhost/x",
                "a name of this computer or without a domain",
            ),
            (
                "https://app.localhost/x",
                "a name of this computer or without a domain",
            ),
            ("https://api.hamburg.de./x", "not a DNS name"),
            ("https://*.hamburg.de/x", "not a DNS name"),
            ("https://api_.hamburg.de/x", "not a DNS name"),
            ("https://-api.hamburg.de/x", "not a DNS name"),
            (
                "https://münchen.example/x",
                "a name outside ASCII must be written in its xn-- form",
            ),
            ("https://api.hamburg.de/a b", "unexpected character"),
            (
                "https://api.hamburg.de\\@evil.example/",
                "unexpected character",
            ),
            ("https://api.hamburg.de/\u{7}", "unexpected character"),
            ("https:///x", "not a host name"),
            ("", "not https"),
        ] {
            let refused = host_of(url).unwrap_err();
            assert_eq!(
                (refused.code, refused.reason),
                ("policy.url", reason),
                "{url}"
            );
        }
    }

    #[test]
    fn a_policy_keeps_requests_on_its_hosts() {
        let policy = UrlPolicy::new(["API.hamburg.de", "geodienste.hamburg.de"]).unwrap();
        assert_eq!(
            policy.hosts().collect::<Vec<_>>(),
            ["api.hamburg.de", "geodienste.hamburg.de"]
        );
        assert_eq!(policy.max_redirects, 3);
        assert!(policy.check_url("https://api.hamburg.de/datasets").is_ok());
        assert!(
            policy
                .check_url("https://API.Hamburg.de:443/datasets")
                .is_ok()
        );
        assert!(policy.allows_host("Geodienste.Hamburg.de"));
        for (url, code) in [
            (
                "https://api.hamburg.de.evil.example/x",
                "policy.host-not-declared",
            ),
            ("https://evilapi.hamburg.de/x", "policy.host-not-declared"),
            ("https://hamburg.de/x", "policy.host-not-declared"),
            ("http://api.hamburg.de/x", "policy.url"),
        ] {
            assert_eq!(policy.check_url(url).unwrap_err().code, code, "{url}");
        }
        for host in ["*.hamburg.de", "localhost", "10.0.0.1", "hamburg", ""] {
            assert_eq!(
                UrlPolicy::new([host]).unwrap_err().code,
                "policy.host",
                "{host}"
            );
        }
    }

    /// The query and the fragment of a URL may carry signed parts; a refusal never shows them.
    #[test]
    fn a_refusal_shows_no_query_and_no_fragment() {
        let refused = host_of("http://example.org/file?token=abc#frag").unwrap_err();
        assert_eq!(refused.to_string(), "http://example.org/file: not https");
    }
}
