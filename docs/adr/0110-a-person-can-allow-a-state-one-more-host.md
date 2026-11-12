# 0110. A person can allow a built-in state one more host

Date: 2026-11-12
Status: Accepted

Follows [0106](0106-requests-go-to-public-addresses-and-declared-hosts.md). The user decided this on 2026-11-12,
after the review of the change that holds the built-in states to the hosts of their manifests.

## Context

Since 0106 a built-in state asks the hosts its manifest names and no other. That has a price. When an agency
moves a download to a new server, puts a redirect in front of it or links to another host from an index, maps
of that state fail until a release names the host. The person at the keyboard can see what happened, the
message names the host, and they cannot do anything about it: an adapter from a folder cannot replace a built-in
one.

Switching the rule off for such a case would give up what 0106 is for. The question was how small a way out
can be.

## Decision

- A person can allow a built-in adapter single hosts besides those of its manifest:
  `freemapper adapters allow-host <adapter> <host>`. `freemapper adapters disallow-host <adapter> <host>` takes
  one back.
- The hosts are kept in one file, `allowed-hosts.toml` in `freemapper` of the configuration folder of the system
  (macOS `~/Library/Application Support`, Linux `~/.config`, Windows `%APPDATA%`). Nothing else adds a host: no
  file in a project folder, no variable of freemapper (`FREEMAPPER_ADAPTER_DIR` and `FREEMAPPER_CONFIG_DIR` do
  not move the file), no switch of a run, no parameter of the MCP server or the area page. The MCP server and
  the area page read the file and have no way to write it.
- A host is checked like a host of a manifest, when it is entered and again when the file is read: a DNS name
  of at least two labels, no IP address, no `localhost`, no name of a local network, no wildcard, no user info,
  no port. An entry that fails is left out with a warning.
- The entry widens the list of one adapter by that one host. Everything else of 0106 holds for it: `https` on
  port 443, at most three redirects, and the check of the address a connection goes to. An allowed host that
  resolves to this computer or to the local network is still refused.
- Only a built-in adapter takes hosts this way. For an adapter of the person's own folder the host belongs in
  `[net] hosts` of its manifest, and the command says so.
- The message about a request that was not sent because the adapter does not name its host starts with the
  refused address, names the state in the language of the run and ends with the next step: for a built-in
  state the exact `allow-host` command and where to report the change, for an adapter from a folder the
  manifest. It is its own error (`FetchError::HostNotNamed`), and the policy of an adapter knows whose it is
  (`UrlPolicy::owned_by`).
- `freemapper adapters list` shows the hosts allowed for each adapter (`allowed_hosts` in its JSON). Once a
  release names a host itself, the next command of the person says that the entry is no longer needed and
  takes it out of the file.

## Why this gap is acceptable

- It is the person's own setting on their own computer, in the place where their other settings are. Someone
  who can write that file can already replace the program.
- It cannot be set from what arrives from outside: a project that was passed on, a manifest, an answer of a
  server, a request to the MCP server.
- It does not reach the local network. The address check is not part of what can be widened.
- It is narrow: one named host for one state, visible in `adapters list`, removed by one command.

## Consequences

- A service that moved is a line the person types, taken from the message, instead of a wait for a release.
  The message asks them to report it, so the release still follows.
- An assistant that runs freemapper for a person must not allow a host on its own. AGENTS.md says: ask the
  person first, and take the host from the message.
- Someone who is talked into allowing a host sends requests for that state to it. The requests carry no
  personal data (a fixed `User-Agent`, the map area as a bounding box), and what comes back is parsed as the
  answer of that service with the size limits of 0106.
- The library takes the allowed hosts as a parameter (`adapters::builtin_with_hosts`); it reads no file. The
  file is the command line's.
