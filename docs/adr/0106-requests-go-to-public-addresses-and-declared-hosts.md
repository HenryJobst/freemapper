# 0106. Requests go to public addresses, and with a policy to declared hosts only

Date: 2026-11-12
Status: Accepted

Part of stage D of [0082](0082-region-adapters-trait-and-manifests.md) (tasks D1 and D2). Builds on
[0059](0059-project-files-never-name-paths.md) (what comes from outside is checked before it is used) and keeps the
rules of the self-update ([0094](0094-self-update-from-signed-releases.md)).

Numbers 0104 and 0105 are taken by work that is not on `main` yet.

## Context

An adapter manifest names the addresses freemapper asks. People copy manifests from forums and repositories, so a
manifest is untrusted input. Two things can go wrong. A manifest can name an address inside the user's network
(`https://192.168.0.1/…`, `http://localhost:8080/…`). And a name that looks harmless can lead there all the same:
its DNS entry points to `127.0.0.1`, or the server answers with a redirect to another host.

Until now only the self-update followed redirects by hand and checked each hop. The fetcher of the pipeline and
the tile proxy of the area page asked whatever address they were given.

## Decision

- The rules for a URL are code of `freemapper-adapter-api` (`policy`), written by hand without a URL crate: `https`
  only, port 443 or none, no user info, no control character, blank or backslash. The host is a DNS name of at
  least two labels in ASCII (a name outside ASCII is written in its `xn--` form). An IP address, `localhost`, a
  trailing dot and a wildcard are refused, and so is a name that ends like a name of a local network (`.local`,
  `.home.arpa`, `.internal`, `.lan`, `.home`, `.corp`, `.intranet`, `.localdomain`, `.private`, `.box`, `.test`,
  `.invalid`).
- A `UrlPolicy` is a list of hosts. A URL passes when its host is in the list. The requests that belong to no
  adapter have a fixed list (`UrlPolicy::core()`): `download.geofabrik.de`, `nominatim.openstreetmap.org` and
  `tile.openstreetmap.org`.
- The address a connection is opened to is checked as well (`policy::check_addr`). For IPv4, refused are loopback,
  private, link-local, multicast and unspecified addresses, the shared address space of carriers and the
  documentation and benchmark nets. For IPv6 the rule is the other way round: only global unicast (`2000::/3`)
  passes, without the documentation nets and the protocol assignments (Teredo among them). An IPv4 address inside
  an IPv6 one (mapped, behind the well-known NAT64 prefix, 6to4) is judged as that IPv4 address, so a network
  without IPv4 still reaches an IPv4 server. The check sits in the resolver of the HTTP client
  (`freemapper::fetch::GuardedResolver`): the client connects to the addresses the resolver returns and to no
  other, so the address checked is the address used. It runs for every redirect hop again, and an answer with one
  refused address is refused as a whole.
- A target that needs no DNS is judged as it is written, before any connection, for every hop
  (`freemapper::fetch::TargetGuard`, the first connector of every agent): a host written as an address goes
  through `check_addr`, and a host that is no DNS name as above (`localhost`, a name of one label, a number,
  `fritz.box`, `printer.local`) is refused.
- Every agent of freemapper that talks to the internet is built by `freemapper::fetch::guarded_agent`: the two
  agents of the fetcher, those of a pinned host, the tile proxy and the self-update.
- With a policy (`HttpFetcher::with_policy`) the fetcher follows redirects by hand: at most three, and each target
  must pass the policy like the first URL. A redirect to another host, to `http` or to an IP address ends the
  request. A pinned host follows no redirect, as before.
- An adapter that came from a folder has a policy of its own: the hosts of its manifest and no other
  (`RegionAdapter::policy`). Whatever is asked for it goes through a fetcher bound to that policy
  (`Fetcher::bound_to`, `policy::PolicyFetcher`): its ALKIS, its rasters, its NAS service and its tree
  cadastres. A fetcher that cannot bind itself, a replay in a test, still has every URL checked before it is
  asked.
- A request freemapper does not send is `FetchError::NotAllowed` with the URL and the reason. It is not asked
  again. The URL is shown without query and fragment, since a signed link carries its token there.
- The self-update keeps its own, narrower rule: the hosts of `freemapper_update::ALLOWED_HOSTS`, each hop through
  `check_url` of freemapper-core, its size limits and `Accept-Encoding: identity`. It gains the address check and
  shares the function that resolves a `Location` (`policy::redirect_target`).
- The address check has one way around it: constructors that exist in test builds only (`HttpFetcher::local`,
  `HttpFetcher::stubbed`, `TileProxy::with_base`, `HttpUpdateSource::stub`), for tests that ask a server on
  `127.0.0.1`. No command-line switch and no environment variable turns the check off: a switch the user can set
  is a switch an attacker can set.

## Consequences

- A manifest cannot make freemapper ask the user's router, a service on the same computer or the metadata address
  of a cloud machine, neither by naming it nor through DNS nor through a redirect. Behind a proxy the part about
  DNS does not hold (see below).
- The fetcher of a run has no host list of its own, so the built-in states ask as before: the address check
  holds, and their redirects are followed as before. An adapter from a folder changes nothing for them. Their
  manifests declare their hosts already, and a test compares the lists with the recorded requests. Holding them
  to the lists waits for live runs of all states, because no recording shows a redirect.
- The list of an adapter from a folder is its own. It cannot ask the host of another adapter, nor the hosts of
  the OSM extracts and the place search, and a redirect cannot lead it there.
- A service of such an adapter that redirects to a host its manifest does not list fails with a message that
  names the host. The fix is one line in `[net] hosts`, which a reviewer sees.
- One request of an adapter does not pass the fetcher: the aerial photo on the map of the area page. The browser
  asks the WMS of `[preview]` itself, so the address check does not apply there, and a declared name that
  resolves into the local network is reachable from the page. Only adapters of the user's own folder reach the
  page, never those of a project, and the browser sends the map bounds and nothing else.
- A proxy from the environment (`HTTPS_PROXY`) is the user's own setting and may be on this computer. Its own
  address is the one thing the check lets through, and it is told apart by identity: the HTTP client hands the
  resolver the URI of the proxy itself. A target on the proxy's host is judged like any other, also when
  `NO_PROXY` makes the client connect to it directly.
- Behind a proxy that resolves the names itself (HTTP `CONNECT`, SOCKS5) freemapper never sees the address a
  name leads to. There the check is weaker: a target written as an address and `localhost` are still refused,
  for every hop, but a declared name can reach what the proxy can reach. With a policy the name must still be a
  declared host.
- One thing no list can close: a network whose own NAT64 prefix (inside `2000::/3`) translates to private IPv4
  addresses.
- A service on another port, without a DNS name or under one of the refused endings (`.box` is also a public
  top-level domain) cannot be an adapter in format 1.
- `Fetcher::get` of the HTTP fetcher fails over 512 MB instead of cutting the answer off.
- The resolver hook of `ureq` (`ureq::unversioned`) is outside its semver promise. The workspace pins `ureq`
  exactly; a version bump runs the tests of this decision first (`declared_host_resolving_to_loopback_is_refused`
  and its neighbours in `crates/freemapper/src/fetch.rs`).
