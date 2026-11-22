# ADR 0180: Issues of users are labelled

Date: 2026-11-22

Status: Accepted

## Context

The maintainer opens most issues of the repository, and so do the agents that work for them. An
issue of a user stands among them and can be overlooked, although it should be looked at first.

## Decision

A workflow sets the label `von-nutzer` on every issue whose author is not the owner, a member, a
collaborator (`author_association`) or a bot (the nightly mutation tests open issues). It sets only the label and never reads the text of the issue,
so an issue cannot steer it. The label has its own colour (orange, as the banner). Two issue forms
(bug, request) ask for the version, the place and what happened; they set `bug` or `enhancement`.
The issues of the project itself are opened without a form and stay without the label. A search
`is:issue is:open label:von-nutzer` lists what is waiting.

## Consequences

- An issue of a user is marked at once, also when it was opened without a form.
- A collaborator who reports as a user is not marked; the maintainer may add the label by hand.
- The workflow needs `issues: write` and nothing else; it uses only the `gh` CLI of the runner, so
  the action pins of ADR 0163 do not change.
