# 0101. Content profiles on the area page

Date: 2026-11-12
Status: Accepted

## Context

ADR 0100 lets the user decide per group what goes straight onto the map, what stays a candidate and what is
dropped. The command line, `freemapper.toml` and MCP had it; the area page, where most people start a map
(ADR 0090), did not. ADR 0096 put the page on the suite frame, with a revision of `freemapper.toml` that every
save checks.

## Decision

- The run panel gets a collapsed `<details>` "What goes onto the map" before the start button. It is collapsed
  because the page already folds secondary lists that way (the download details) and because 14 groups would
  push the start button out of sight; most people keep `balanced`.
- It holds the three profiles as radio buttons, each with its hint, and one select per group in the library's
  order (`Group::ALL`) with four entries: "As in the profile" (no exception), Map, Candidate, Off. A group
  without an exception shows "As in the profile" and no fixed place, because what a profile does depends on the
  source of each object (ADR 0100).
- `api/project` carries `content` (the stored settings, or null) and `content_options` (the names of the
  profiles, groups and places from the library). The page names them from its own table of literal text ids; a
  test fails when the library gains a group, profile or place that the table lacks.
- Changing the profile keeps the exceptions, because the user sees them next to the profile and resets them one
  by one. `--content <profile>` on the command line starts afresh instead (ADR 0100); a command line has no
  list in front of it that shows what would be lost.
- `api/save` takes an optional `content` and stores it in the same write as the area, under the same revision
  check (ADR 0096): one write, one new revision. The library gets `set_map_area_with_content` and
  `init_project_with_content` for it; `set_map_area` and `init_project` call them with `None`. An absent
  `content` leaves the stored settings as they are. An invalid one (unknown key or value) is answered with 422
  and one sentence; the file stays unchanged. The page sends `content` only when its choice differs from the
  file, so an untouched page does not add a `[content]` table to a project that has none.
- The save signature of the page includes the choice, so a run saves first when only the content changed. The
  controls lock during a run like the other settings.
- `api/run` takes nothing new: the run reads the settings from the project file, so page, command line and MCP
  see the same state.

## Consequences

- A person on the page can choose the former behaviour (`safe`) or everything (`full`) without a terminal.
- The server and the library own the lists; the page owns the words. A new group needs one table entry and two
  catalog messages.
- Cost if wrong: a stale page overwriting a choice made on the command line is stopped by the revision check, as
  for the area.

Source: ADR 0100 (user decision 2026-11-12: adjustable on the area page).
