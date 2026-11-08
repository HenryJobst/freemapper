# 0037. AI features inside the program are optional

Date: 2026-11-08
Status: Accepted

## Context

freemapper belongs to a suite with freemapper-print and freemapper-courseplan. The user set one rule for all of
them: a program may offer features that call a language model, but nobody may need them. freemapper calls no model
today. Its MCP server is the other way round: an outside assistant drives freemapper, and freemapper never calls
the assistant.

## Decision

Any feature that calls a model is off unless the user configures a model. Without that setting, freemapper does
every job in full: area, fetch, derivation, OOM, OCAD and the PDF. A model-backed feature only adds to a result that
the program can also produce without it. The MCP server is out of scope of this rule, because there the model sits
outside the program.

## Consequences

- No model setting, account or network access to a model provider is required to use freemapper.
- A future AI feature needs a configuration switch, a path that works without it, and tests for both paths.
- The CLI, the MCP server and the area selector stay usable offline from any model provider.
- Cost if wrong: a later feature that would be simpler with a required model needs a fallback.

Source: user rule for the suite, passed on by the freemapper-courseplan session on 2026-11-08.
