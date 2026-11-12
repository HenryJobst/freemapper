//! Where a request may go: the URL rules for adapters (ADR 0082, spec 3.8).
//!
//! A manifest is untrusted input. Its URLs are `https` on port 443, name a host by its DNS name
//! and stay on the hosts the manifest declares. The rules are parsed by hand: no URL crate.
//!
//! A declared name can still resolve to this computer or to the local network, so the address a
//! connection goes to is checked as well ([`check_addr`]), for the first request and for every
//! redirect. The fetcher of `freemapper` does that at connect time.

use std::collections::BTreeSet;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::Path;

use chrono::NaiveDate;

use crate::error::{FetchError, HostNotNamed, Remedy};
use crate::fetch::{ByteRange, Fetcher};
use crate::note::Note;

/// A URL or a host a policy refuses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyError {
    /// The stable code of the rule: `policy.url`, `policy.host`, `policy.host-not-declared`,
    /// `policy.address`, `policy.redirect`.
    pub code: &'static str,
    /// The URL, host or address, without query and fragment (they may carry signed parts).
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

/// Every reason a refusal of this module gives ([`PolicyError::reason`]), for the programs that
/// word them in the language of the run.
pub const REASONS: [&str; 12] = [
    "not a host name",
    "a name outside ASCII must be written in its xn-- form",
    "not a DNS name",
    "a name of this computer or without a domain",
    "a name of the local network",
    "an IP address, not a DNS name",
    "unexpected character",
    "not https",
    "user info",
    "a port other than 443",
    "the host is not one of the declared hosts",
    "an address of this computer or of the local network, not of a server on the internet",
];

/// The last label of a name of a local network.
const LOCAL_ENDINGS: [&str; 11] = [
    "local",
    "internal",
    "lan",
    "home",
    "corp",
    "intranet",
    "localdomain",
    "private",
    "box",
    "test",
    "invalid",
];

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
    // The endings of names that live in a local network and not in the public DNS: the
    // special-use domains (`.local`, `.home.arpa`, `.internal`, `.test`, `.invalid`) and what
    // home routers and company networks use (`fritz.box`, `.lan`, `.corp`).
    let second_last = labels.len().checked_sub(2).map(|at| labels[at]);
    if LOCAL_ENDINGS.contains(&last) || (last == "arpa" && second_last == Some("home")) {
        return refuse("a name of the local network");
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

/// Whether a request may go to the IPv4 address: a public one. Refused are this computer, the
/// local network and everything else that is no host on the internet.
fn public_v4(addr: Ipv4Addr) -> bool {
    let [a, b, c, _] = addr.octets();
    !(a == 0                                   // "this network", unspecified
        || a == 127                            // loopback
        || a == 10                             // private
        || (a == 172 && (16..=31).contains(&b))
        || (a == 192 && b == 168)
        || (a == 169 && b == 254)              // link-local, cloud metadata
        || (a == 100 && (64..=127).contains(&b)) // shared address space of carriers
        || (a == 192 && b == 0 && c == 0)      // protocol assignments
        || (a == 192 && b == 0 && c == 2)      // documentation
        || (a == 198 && b == 51 && c == 100)
        || (a == 203 && b == 0 && c == 113)
        || (a == 198 && (18..=19).contains(&b)) // benchmarking
        || a >= 224) // multicast, reserved, broadcast
}

/// Whether a request may go to the IPv6 address. Everything is refused that is not global
/// unicast (`2000::/3`), and of that the nets for documentation and for protocols (Teredo among
/// them). An IPv4 address inside an IPv6 one is judged as that IPv4 address: mapped
/// (`::ffff:a.b.c.d`), behind the well-known NAT64 prefix (how a network without IPv4 reaches
/// an IPv4 server) and 6to4.
fn public_v6(addr: Ipv6Addr) -> bool {
    let embedded = |high: u16, low: u16| {
        let [a, b] = high.to_be_bytes();
        let [c, d] = low.to_be_bytes();
        public_v4(Ipv4Addr::new(a, b, c, d))
    };
    match addr.segments() {
        [0, 0, 0, 0, 0, 0xffff, high, low] => embedded(high, low),
        [0x64, 0xff9b, 0, 0, 0, 0, high, low] => embedded(high, low),
        [0x2002, high, low, ..] => embedded(high, low),
        // 2001::/23, the protocol assignments (Teredo, ORCHID, benchmarking), and the
        // documentation nets 2001:db8::/32 and 3fff::/20.
        [0x2001, second, ..] if second < 0x200 || second == 0xdb8 => false,
        [0x3fff, second, ..] if second & 0xf000 == 0 => false,
        [first, ..] => first & 0xe000 == 0x2000,
    }
}

/// Refuses an address a request may not go to: loopback, link-local, private, unique-local,
/// multicast, unspecified and other addresses that are no host on the internet. Checked for the
/// address a connection is opened to, so a declared name cannot lead to the local network.
pub fn check_addr(addr: IpAddr) -> Result<(), PolicyError> {
    let public = match addr {
        IpAddr::V4(addr) => public_v4(addr),
        IpAddr::V6(addr) => public_v6(addr),
    };
    if public {
        Ok(())
    } else {
        Err(PolicyError::new(
            "policy.address",
            &addr.to_string(),
            "an address of this computer or of the local network, not of a server on the internet",
        ))
    }
}

/// The absolute URL of a redirect's `location`, which may be relative to `base` (RFC 9110 allows
/// it): a whole URL, `//host/path`, `/path`, or a path relative to `base`'s folder. Dot segments
/// are left to the server; the rule for URLs judges the result.
pub fn redirect_target(base: &str, location: &str) -> String {
    let has_scheme = location.split_once("://").is_some_and(|(scheme, _)| {
        !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    });
    if has_scheme {
        return location.to_string();
    }
    let (scheme, rest) = base.split_once("://").unwrap_or(("https", base));
    if let Some(network_path) = location.strip_prefix("//") {
        return format!("{scheme}://{network_path}");
    }
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let origin = format!("{scheme}://{}", &rest[..authority_end]);
    if location.starts_with('/') {
        return format!("{origin}{location}");
    }
    let path = rest[authority_end..].split(['?', '#']).next().unwrap_or("");
    let folder = path.rfind('/').map_or("/", |end| &path[..=end]);
    format!("{origin}{folder}{location}")
}

/// The hosts the requests of one adapter (or of the program itself) may go to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlPolicy {
    hosts: BTreeSet<String>,
    /// How many redirects a request may follow; each target passes [`UrlPolicy::check_url`].
    pub max_redirects: u8,
    /// The adapter the policy belongs to, for the message about a host it does not name.
    owner: Option<Owner>,
}

/// The adapter a policy belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Owner {
    /// The id of the adapter.
    pub adapter: String,
    /// The English name of its region.
    pub name: String,
    /// What a person can do about a host the adapter does not name.
    pub remedy: Remedy,
}

impl UrlPolicy {
    /// The redirects a request follows unless the policy says otherwise.
    pub const MAX_REDIRECTS: u8 = 3;

    /// A policy for `hosts`, each a DNS name as [`dns_name`] accepts it.
    pub fn new<'a>(hosts: impl IntoIterator<Item = &'a str>) -> Result<Self, PolicyError> {
        Ok(Self {
            hosts: hosts.into_iter().map(dns_name).collect::<Result<_, _>>()?,
            max_redirects: Self::MAX_REDIRECTS,
            owner: None,
        })
    }

    /// The policy as that of the adapter `owner`: a host it does not name is refused with
    /// what the person can do about it ([`FetchError::HostNotNamed`]).
    pub fn owned_by(mut self, owner: Owner) -> Self {
        self.owner = Some(owner);
        self
    }

    /// The error of a request this policy refused with `refused`. `from` is the address whose
    /// answer redirected to the refused one.
    pub fn refusal(&self, refused: PolicyError, from: Option<&str>) -> FetchError {
        let bare = |url: &str| url.split(['?', '#']).next().unwrap_or(url).to_string();
        if let Some(owner) = &self.owner
            && refused.code == "policy.host-not-declared"
            && let Ok(host) = host_of(&refused.subject)
        {
            return FetchError::HostNotNamed(Box::new(HostNotNamed {
                target: refused.subject,
                host,
                from: from.map(bare),
                adapter: owner.adapter.clone(),
                name: owner.name.clone(),
                remedy: owner.remedy,
            }));
        }
        match from {
            Some(from) => FetchError::NotAllowed {
                url: bare(from),
                reason: format!("a redirect to {refused}"),
            },
            None => FetchError::NotAllowed {
                url: refused.subject,
                reason: refused.reason.to_string(),
            },
        }
    }

    /// The hosts of the requests that belong to no adapter: the OSM extracts and their state
    /// files, the place search and the map tiles of the area page.
    pub const CORE_HOSTS: [&'static str; 3] = [
        "download.geofabrik.de",
        "nominatim.openstreetmap.org",
        "tile.openstreetmap.org",
    ];

    /// The policy of the requests that belong to no adapter ([`UrlPolicy::CORE_HOSTS`]).
    pub fn core() -> Self {
        Self::new(Self::CORE_HOSTS).expect("the core hosts are DNS names")
    }

    /// The policy with `hosts` added.
    pub fn with_hosts<'a>(
        mut self,
        hosts: impl IntoIterator<Item = &'a str>,
    ) -> Result<Self, PolicyError> {
        for host in hosts {
            self.hosts.insert(dns_name(host)?);
        }
        Ok(self)
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

/// A fetcher that sends a request on only when its URL passes `policy`: the requests of an
/// adapter that came from a folder. When the fetcher behind it can hold itself to the policy
/// ([`Fetcher::bound_to`]), the requests go through that one, and the redirects stay on the
/// hosts of the policy as well.
pub struct PolicyFetcher<'a> {
    inner: &'a dyn Fetcher,
    bound: Option<Box<dyn Fetcher + 'a>>,
    policy: &'a UrlPolicy,
}

impl<'a> PolicyFetcher<'a> {
    pub fn new(inner: &'a dyn Fetcher, policy: &'a UrlPolicy) -> Self {
        Self {
            inner,
            bound: inner.bound_to(policy),
            policy,
        }
    }

    /// The fetcher a request for `url` goes to, if the policy lets it go.
    fn through(&self, url: &str) -> Result<&dyn Fetcher, FetchError> {
        self.policy
            .check_url(url)
            .map_err(|refused| self.policy.refusal(refused, None))?;
        Ok(self.bound.as_deref().unwrap_or(self.inner))
    }
}

impl Fetcher for PolicyFetcher<'_> {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        self.through(url)?.get(url)
    }

    fn last_modified(&self, url: &str) -> Result<Option<NaiveDate>, FetchError> {
        self.through(url)?.last_modified(url)
    }

    fn get_range(&self, url: &str, range: ByteRange) -> Result<Vec<u8>, FetchError> {
        self.through(url)?.get_range(url, range)
    }

    fn download_to(
        &self,
        url: &str,
        path: &Path,
        max_bytes: u64,
        progress: &mut dyn FnMut(Note),
    ) -> Result<(), FetchError> {
        self.through(url)?
            .download_to(url, path, max_bytes, progress)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    /// A fetcher that writes down what it was asked, and one bound to a policy that writes
    /// into the same list with `bound:` before the URL.
    struct Logging<'a> {
        log: &'a RefCell<Vec<String>>,
        prefix: &'static str,
        binds: bool,
    }

    impl Fetcher for Logging<'_> {
        fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
            self.log.borrow_mut().push(format!("{}{url}", self.prefix));
            Ok(Vec::new())
        }

        fn bound_to(&self, _policy: &UrlPolicy) -> Option<Box<dyn Fetcher + '_>> {
            self.binds.then(|| {
                Box::new(Logging {
                    log: self.log,
                    prefix: "bound:",
                    binds: false,
                }) as Box<dyn Fetcher>
            })
        }
    }

    /// Every way to ask is judged before the request, and nothing refused is sent.
    #[test]
    fn a_policy_fetcher_sends_declared_urls_only() {
        let policy = UrlPolicy::new(["geo.example.org"]).unwrap();
        let log = RefCell::new(Vec::new());
        let plain = Logging {
            log: &log,
            prefix: "",
            binds: false,
        };
        let fetcher = PolicyFetcher::new(&plain, &policy);
        assert!(fetcher.get("https://geo.example.org/items?page=2").is_ok());
        let evil = "https://evil.example.net/items?token=abc";
        let Err(FetchError::NotAllowed { url, reason }) = fetcher.get(evil) else {
            panic!("the request went out");
        };
        assert_eq!(url, "https://evil.example.net/items");
        assert_eq!(reason, "the host is not one of the declared hosts");
        assert!(matches!(
            fetcher.last_modified(evil),
            Err(FetchError::NotAllowed { .. })
        ));
        assert!(matches!(
            fetcher.get_range(evil, ByteRange::Last(4)),
            Err(FetchError::NotAllowed { .. })
        ));
        assert!(matches!(
            fetcher.download_to(evil, Path::new("nowhere"), 10, &mut |_| {}),
            Err(FetchError::NotAllowed { .. })
        ));
        assert_eq!(*log.borrow(), ["https://geo.example.org/items?page=2"]);
    }

    /// The policy of an adapter refuses a host the adapter does not name with what the
    /// person can do: the refused address first, then the region, then the next step. For an
    /// adapter of the program that is the command that allows the host; for a folder of the
    /// person, the manifest. Other refusals keep their words.
    #[test]
    fn the_policy_of_an_adapter_says_what_to_do_about_a_host_it_does_not_name() {
        let owned = |remedy| {
            UrlPolicy::new(["geo.example.org"])
                .unwrap()
                .owned_by(Owner {
                    adapter: "hessen".to_string(),
                    name: "Hesse".to_string(),
                    remedy,
                })
        };
        let policy = owned(Remedy::AllowHost);
        let refused = policy
            .check_url("https://cdn.example.net/tiles/1.zip?sig=abc")
            .unwrap_err();
        assert_eq!(
            policy.refusal(refused.clone(), None).to_string(),
            "https://cdn.example.net/tiles/1.zip: freemapper does not send requests to this \
             address: the adapter of Hesse does not name the host cdn.example.net. If the \
             service has moved there, allow the host with `freemapper adapters allow-host \
             hessen cdn.example.net` and report it at \
             https://github.com/HenryJobst/freemapper/issues"
        );
        let redirected = policy.refusal(refused.clone(), Some("https://geo.example.org/a?x=1"));
        assert_eq!(
            redirected.to_string(),
            "https://cdn.example.net/tiles/1.zip: freemapper does not send requests to this \
             address: the adapter of Hesse does not name the host cdn.example.net \
             (https://geo.example.org/a redirects there). If the service has moved there, \
             allow the host with `freemapper adapters allow-host hessen cdn.example.net` and \
             report it at https://github.com/HenryJobst/freemapper/issues"
        );
        assert_eq!(
            owned(Remedy::Manifest)
                .refusal(refused.clone(), None)
                .to_string(),
            "https://cdn.example.net/tiles/1.zip: freemapper does not send requests to this \
             address: the adapter hessen does not name the host cdn.example.net. If the \
             service has moved there, add the host to `[net] hosts` of the adapter's manifest"
        );
        // A refusal of another kind, and one of a policy that is nobody's, read as before.
        let plain = policy.check_url("http://geo.example.org/a").unwrap_err();
        assert_eq!(
            policy
                .refusal(plain, Some("https://geo.example.org/b"))
                .to_string(),
            "https://geo.example.org/b: freemapper does not send requests to this address: a \
             redirect to http://geo.example.org/a: not https"
        );
        let nobody = UrlPolicy::new(["geo.example.org"]).unwrap();
        assert_eq!(
            nobody.refusal(refused, None).to_string(),
            "https://cdn.example.net/tiles/1.zip: freemapper does not send requests to this \
             address: the host is not one of the declared hosts"
        );
    }

    /// A fetcher that can hold itself to the policy gets the requests, so its redirects are
    /// judged too; the URL is still checked first.
    #[test]
    fn a_policy_fetcher_asks_through_the_bound_fetcher() {
        let policy = UrlPolicy::new(["geo.example.org"]).unwrap();
        let log = RefCell::new(Vec::new());
        let binding = Logging {
            log: &log,
            prefix: "",
            binds: true,
        };
        let fetcher = PolicyFetcher::new(&binding, &policy);
        fetcher.get("https://geo.example.org/a").unwrap();
        fetcher
            .get_range("https://geo.example.org/b", ByteRange::Last(4))
            .unwrap();
        assert!(fetcher.get("https://evil.example.net/x").is_err());
        assert_eq!(
            *log.borrow(),
            [
                "bound:https://geo.example.org/a",
                "bound:https://geo.example.org/b"
            ]
        );
    }

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
            ("https://printer.local/x", "a name of the local network"),
            ("https://nas.home.arpa/x", "a name of the local network"),
            ("https://db.internal/x", "a name of the local network"),
            ("https://router.lan/x", "a name of the local network"),
            ("https://fritz.box/x", "a name of the local network"),
            ("https://wiki.corp/x", "a name of the local network"),
            ("https://x.test/x", "a name of the local network"),
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
            assert!(REASONS.contains(&refused.reason), "{}", refused.reason);
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

    /// A declared name can still resolve to this computer or to the local network; a request
    /// goes to public addresses only (review focus 3).
    #[test]
    fn private_ranges_are_refused() {
        for refused in [
            "127.0.0.1",
            "127.255.255.254",
            "10.0.0.1",
            "172.16.0.1",
            "172.31.255.255",
            "192.168.1.1",
            "169.254.169.254",
            "100.64.0.1",
            "100.127.255.255",
            "0.0.0.0",
            "0.1.2.3",
            "224.0.0.1",
            "239.255.255.250",
            "240.0.0.1",
            "255.255.255.255",
            "192.0.0.8",
            "192.0.2.1",
            "198.18.0.1",
            "198.51.100.7",
            "203.0.113.9",
            "::1",
            "::",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
            "febf::1",
            "ff02::1",
            "2001:db8::1",
            // An IPv4 address inside an IPv6 one is judged as the IPv4 address.
            "::ffff:127.0.0.1",
            "::ffff:10.1.2.3",
            "::7f00:1",
            "64:ff9b::a00:1",
            "64:ff9b::7f00:1",
            "2002:7f00:1::1",
            "2002:c0a8:101::1",
            // IPv6 is refused unless it is global unicast: the deprecated IPv4-compatible
            // form, the local NAT64 prefix, Teredo and the other protocol assignments, the
            // second documentation net, and everything outside 2000::/3.
            "::808:808",
            "64:ff9b:1::808:808",
            "2001::1",
            "2001:0:4136:e378:8000:63bf:3fff:fdd2",
            "2001:2::1",
            "2001:10::1",
            "3fff::1",
            "3fff:0fff::1",
            "4000::1",
            "100::1",
            "e000::1",
        ] {
            let addr: IpAddr = refused.parse().unwrap();
            let error = check_addr(addr).unwrap_err();
            assert!(REASONS.contains(&error.reason), "{}", error.reason);
            assert_eq!(error.code, "policy.address", "{refused}");
            assert_eq!(error.subject, addr.to_string());
        }
        for allowed in [
            "8.8.8.8",
            "172.15.255.255",
            "172.32.0.1",
            "100.63.255.255",
            "100.128.0.1",
            "192.167.255.255",
            "193.99.144.80",
            "2a00:1450:4001:81b::200e",
            "2001:4860:4860::8888",
            "2606:4700::6810:84e5",
            // Beside the documentation net 3fff::/20.
            "3fff:1000::1",
            "3ff0::1",
            "::ffff:8.8.8.8",
            "64:ff9b::808:808",
            "2002:808:808::1",
        ] {
            assert_eq!(check_addr(allowed.parse().unwrap()), Ok(()), "{allowed}");
        }
    }

    /// Where a redirect leads: the `Location` may be a whole URL or relative to the URL asked.
    #[test]
    fn a_location_resolves_against_the_url_asked() {
        let base = "https://github.com/a/b/c?query=1#frag";
        for (location, target) in [
            ("https://example.org/x", "https://example.org/x"),
            ("http://example.org/x", "http://example.org/x"),
            ("//example.org/x", "https://example.org/x"),
            ("/x/y", "https://github.com/x/y"),
            ("d?e=2", "https://github.com/a/b/d?e=2"),
            ("", "https://github.com/a/b/"),
        ] {
            assert_eq!(redirect_target(base, location), target, "{location}");
        }
        assert_eq!(
            redirect_target("https://github.com", "x"),
            "https://github.com/x"
        );
    }

    /// The requests that belong to no adapter: OSM extracts, the place search, the map tiles.
    #[test]
    fn core_policy_allows_geofabrik_nominatim_and_osm_tiles_only() {
        let core = UrlPolicy::core();
        assert_eq!(
            core.hosts().collect::<Vec<_>>(),
            [
                "download.geofabrik.de",
                "nominatim.openstreetmap.org",
                "tile.openstreetmap.org"
            ]
        );
        assert!(
            core.check_url("https://download.geofabrik.de/europe/germany/hamburg-latest.osm.pbf")
                .is_ok()
        );
        assert_eq!(
            core.check_url("https://geodienste.hamburg.de/wms")
                .unwrap_err()
                .code,
            "policy.host-not-declared"
        );
        // An adapter adds its hosts; a host named twice is there once.
        let more = core
            .with_hosts(["geodienste.hamburg.de", "Download.Geofabrik.de"])
            .unwrap();
        assert_eq!(more.hosts().count(), 4);
        assert!(more.check_url("https://geodienste.hamburg.de/wms").is_ok());
    }
}
