# 0023. Schleswig-Holstein: pinned D-TRUST anchors for one host

Date: 2026-11-08
Status: Accepted

## Context

`geodaten.schleswig-holstein.de` sends its leaf certificate without the intermediate. rustls and curl do not
fetch the missing intermediate, so the connection fails. The bDOM and the tile index exist only on that host.

## Decision

freemapper bundles the `D-TRUST BR CA 1-20-1 2020` intermediate and its root in `assets/tls/`. A second pair of
`ureq` agents uses them as its only trust anchors and serves only that exact host. Every other host keeps the
default roots. These agents follow no redirects.

## Consequences

- The intermediate is an anchor, so rustls does not check its validity, EKU or revocation. The code comment says
  so and does not claim less trust than before.
- Cost if wrong: the anchors expire in 2035 or the host changes CA, and SH bDOM and tile names then fail with a
  TLS error. A revoked intermediate would stay trusted for that host until freemapper updates the bundle.

Source: spec §4.1 (Schleswig-Holstein); plan `2026-11-08-schleswig-holstein.md` (Rulings, TLS); SH rulings (TLS,
Task 3); commits 1d20fef, 8b551da, 7f4a722.
