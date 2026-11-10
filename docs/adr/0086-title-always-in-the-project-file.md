# 0086. The project file always holds a title

Date: 2026-11-10
Status: Accepted

## Context

`title` in `freemapper.toml` was optional. Without it freemapper derived the title for the PDF from the folder name
(`clara-zetkin-schule` becomes `Clara-Zetkin-Schule`). freemapper-print and freemapper-courseplan read the same file and
could end up with another text, or none.

## Decision

- `init_project` always writes `title`: the cleaned `--title`, or the title derived from the folder name by the same rule
  the PDF uses (`ProjectFile::display_title`).
- Projects created earlier keep an absent `title`; freemapper still derives it for the PDF as before. Nothing rewrites
  the file.
- Print and courseplan read `title` first and may fall back to the folder name for old projects.

## Consequences

- One title in PDF, print and courseplan for every new project.
- Cost if wrong: a renamed folder no longer renames the title of a new project. Edit `title` in the file.
