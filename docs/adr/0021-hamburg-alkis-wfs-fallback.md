# 0021. Hamburg ALKIS falls back to the classic WFS

Date: 2026-11-08
Status: Accepted

## Context

Hamburg ALKIS comes from the OGC API `alkis_vereinfacht`, read by the LGB reader. On 2026-11-07 that API
(api.hamburg.de) stopped answering for hours. The classic WFS `WFS_HH_ALKIS_vereinfacht` serves the same data as
GML, answered in 0.1 s and stayed up.

## Decision

When the OGC API fails, freemapper asks the classic WFS instead. MV ALKIS uses the same pattern, an OGC API read
by the LGB reader with a WFS fallback.

## Consequences

- One Hamburg outage no longer stops a fetch.
- Two readers must keep giving the same normalised features.
- Cost if wrong: if the two services drift apart, a map built during an outage differs from one built before it.

Source: spec §4.1 (Hamburg, Mecklenburg-Vorpommern); commits ccece60, 9998c45.
