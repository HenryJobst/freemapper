# ADR 0180: A service that is down for a moment is asked again; if it stays down, the message says so

Date: 2026-11-22

Status: Accepted

## Context

Issue #219: a person's run for a map in Saxony failed with `download of …/sachsen-latest.osm.pbf failed: http status: 503`.
The first message of that issue was the 404 of #244 (fixed in v0.26.0); the 503 after it came from the Geofabrik
server, which answers so when it is busy or writes new extracts. The OSM extract was the one big download that gave up
at the first 503: its state file and its PBF file went through `Fetcher::get` and `download_to` directly, while most
other requests already ask up to three times (`get_with_retries` and its siblings, 2 s and 4 s apart).

## Decision

1. **Every request to a geodata service goes through the retrying helpers of `freemapper-adapter-api::fetch`.**
   The calls that did not: the OSM state file and extract, the pages of OGC API features, the place search, the
   currency lists and archives of the German tile services, the LGB and LVermGeo point-cloud requests, the GeoSN flight
   date, and the Polish orthophoto sheets and BDOT10k archives.
2. **What is asked again:** a 5xx, now also 408 and 429 (`status_answer_error` makes them `FetchError::Http`
   instead of `Refused`), and timeouts and connection errors. **What is not:** 404, other refused requests, hosts the
   adapter does not name, a download over its limit and a full disk.
3. **The final message says what happened.** `FetchError::Http` with a failure that tends to pass
   (`is_temporary_message`) reads: the service is probably overloaded or under maintenance, freemapper asked up to three
   times, run it again in a few minutes, what is already downloaded is kept. The text is in all 21 languages
   (`error-fetch-http-temporary`); the English text equals the library's message, as for every error.

## Consequences

- A busy server for a few seconds no longer ends a run. The pauses stay at 2 s and 4 s; a maintenance of several
  minutes still ends the run, now with a message that says to try again later.
- A request that fails with a 5xx for a lasting reason is asked three times before it ends the run; the cost is
  six seconds of waiting.
- `status_answer_error` no longer returns `Refused` for 408 and 429, which no caller treated as a request to change.
