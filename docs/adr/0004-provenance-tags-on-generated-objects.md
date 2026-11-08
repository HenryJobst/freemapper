# 0004. Provenance tags on generated objects

Date: 2026-11-06
Status: Accepted

## Context

The MVP produces a starter project only. Reconciling new data with an already edited map comes later, and it
needs to know where each generated object came from.

## Decision

Every generated map object carries OOM object tags: `freemapper:source`, `freemapper:source_id` (for example the
ALKIS object id or the OSM id) and `freemapper:currency` (the data date). The project file records parameters
and source versions.

## Consequences

- A later reconciliation step can match objects by source id instead of by geometry.
- Mappers see the tags in OOM; they do not change how the map looks.
- Cost if wrong: if reconciliation needs more than these three fields, maps made before that change lack the
  data and must be matched another way.

Source: spec §2 (Goals, Non-goals), §4.3; plan-mode notes "Wiederholte Läufe (Nutzer)".
