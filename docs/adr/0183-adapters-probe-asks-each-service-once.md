# ADR 0183: `adapters probe` asks each service once

Date: 2026-11-22

Status: Accepted

Refs: issue #286, ADR 0180.

## Context

A move or an outage of a geodata service showed only when a person ran into it (the Geofabrik move, issue #244). A nightly check should see it first, and the website can then say that the problem is known (issue #286).

## Decision

`freemapper adapters probe [--json]` reads the manifests of the built-in adapters, takes every `https://` address they hold and the Geofabrik extracts, and asks each address once through the fetch layer, so the host rules and the fixed `User-Agent` apply as in a run. The kind of address decides the request: files (`.pbf`, `.csv`, `.zip`, `/exports/`) get a HEAD; an OGC API `/items` gets `?limit=1`; every other address is a map service and gets `service=WMS|WCS|WFS&request=GetCapabilities`. A HEAD on the bare service address is no probe: Saxony-Anhalt, Saarland and Cologne answer it with 500 or 404, and a real request with 200.

The result per service is `ok`, `slow` (more than 10 s), `moved` (the answer leads to a host the adapter does not name) or `error`. A 4xx other than 404 counts as `ok` with a remark, because the server answers. The probe does not retry; a service counts as down only after two nights in a row (the workflow of stage 2). The exit code is 0 also when a service fails, because the result is the answer.

## Consequences

- Reachable is not correct; the golden runs check the data.
- The Polish adapter and the addresses that still sit in code join when #273 moves them into manifests.
- The probe makes about 85 requests, one per service. It belongs in the nightly workflow, never in the CI of a pull request.
