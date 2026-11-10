# 0090. One document build per command

Date: 2026-11-11
Status: Accepted

## Context

`starter_documents` builds the map and the candidates template from every stored layer of a project. Each export
function called it on its own: `export_starter_project`, `export_ocad`, `export_pdf` and `export_candidates_pdf`. One
`freemapper export --ocad --pdf --pdf-candidates` therefore built the same documents four times, and `run` with the
same switches built them once inside `run_project` and three more times after it. A review of `0f25841` found this; the
user asked for the fix on 2026-11-11.

## Decision

- `StarterDocuments` and `starter_documents` are public. The fields stay private.
- Every export function has a twin that takes the built documents: `export_starter_project_from`, `export_ocad_from`,
  `export_pdf_from`, `export_candidates_pdf_from`. The old functions keep their signatures and build the documents
  themselves, so a caller with one output changes nothing.
- The CLI's `export` builds once for all its outputs. `run` and the MCP pipeline build once for the outputs after
  `run_project`, which keeps its own build: its map and candidates are written before anything else is asked for.
- The documents belong to one folder and to the data stored at that moment. They are built again after a fetch or a
  derive, and never kept across commands.
- None of the exports writes a file the build reads (it reads `data/` and checks the image templates that `derive`
  wrote), so sharing the documents changes no output. A test writes all outputs both ways and compares the bytes.

## Consequences

- `export` with every switch builds once instead of four times, `run` twice instead of four times.
- In the MCP pipeline a failed build is reported once per requested output, with the same text as before.
- The time saved was not measured: the example projects in the repository carry no stored layers to build from.
- Cost if wrong: an export that starts to write something the build reads would make the shared documents stale
  within one command. The byte comparison test would not see that; a reviewer of such an export has to.
