# ADR 0186: The nightly service status lives on the branch `status`

Date: 2026-11-22

Status: Accepted

Refs: issue #286, ADR 0185.

## Context

`freemapper adapters probe` (ADR 0185) asks every service once. A single answer proves little: an agency can be in a maintenance window for a night, or block the datacentre of GitHub for an hour. The website and the maintainer need the state over several nights, kept somewhere a workflow can write and the site can read.

## Decision

The workflow `services-nightly.yml` runs at 03:17 UTC, before the daily build of the website at 04:17. It reads last night's `status.json` from the branch `status`, runs `freemapper adapters probe --status <file>` and publishes the new `status.json` on that branch, which holds nothing else (like `badges`, ADR 0177).

A service counts as down after two bad nights in a row (`error` or `moved`); `slow` is no bad night. The rule is Rust code in `freemapper::service_status`, tested with the rest, not a script. The status keeps, per address, the adapters that use it, the last answer, the bad nights, the day the outage began and whether `down` changed tonight; `down_by_adapter` is the shortcut for the website.

When a service goes down or answers again, the workflow opens or updates one issue "Geodienste: Ausfälle erkannt" with the addresses and the first day, and closes it when no service is down. A night without a change touches nothing.

## Consequences

- The branch is rewritten every night; the history of the outages is in the closed issues.
- Until the first night has run, the website finds no `status.json` and shows nothing.
- Services behind an agency that blocks datacentres for good would show as down every night. A list of exceptions is added when the first such case appears, not before.
- The Polish adapter and the addresses still in code join the probe with issue #273 (stage 4 of #286).
