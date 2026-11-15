# freemapper.de: a website for the suite

Date: 2026-11-15
Status: Draft for the user's review

## Intent

GitHub shows a README in its own fonts and layout, and the user asked whether that can change. It cannot; a site of its
own can. The user wants a website for freemapper and its sister programs on GitHub Pages, set in Inter
(<https://rsms.me/inter/>), under the domain freemapper.de, which the user registers.

The site is for the people who would use the suite and do not read GitHub: club mappers, course setters, school
teachers, event organisers. It answers three questions on the first screen: what the suite makes, whether it works for
my place, and how I get it. GitHub stays the place for code, issues and releases; the site links there.

## Goals

- One start page for the suite: freemapper (base maps and training maps), freemapper-print (print layouts) and
  freemapper-courseplan (course setting), with freemapper-core named as their shared library.
- A page per program with what it does, its state, and its download for each system, always the newest release.
- The examples of freemapper as a gallery: the training maps, the check prints and the PDFs, per state.
- The manuals and the changes readable on the site, in the version that matches the download.
- German and English, German first (the users are in Germany).
- Fast, readable, no tracking, nothing loaded from other hosts.

## Non-goals

- No web version of the programs. The site describes and links; maps are made on the user's computer.
- No accounts, no forms, no comments, no newsletter, no analytics.
- No second copy of the documentation that drifts: the site renders the files of the repositories, it does not keep
  its own.

## Anti-goals

- Loading Inter (or anything else) from a third-party server. A request to a font CDN hands the visitor's IP address
  to that host; German courts have fined sites for doing that with Google Fonts. Everything is served from
  freemapper.de.
- A site that says more than the programs do. Supported states, map kinds and outputs come from the same files as
  the READMEs.

## Decisions (mine; the user can overrule them)

### Where it lives

- A new public repository `HenryJobst/freemapper-site`, licence AGPL-3.0-or-later like the suite (ADR 0152) for the
  code, the texts under the same licence as the repositories' docs. Not a folder in freemapper: the site belongs to
  the suite, and its build pulls from all three programs.
- GitHub Pages built by GitHub Actions (`actions/upload-pages-artifact` and `actions/deploy-pages`, pinned by commit
  as in the other workflows), custom domain freemapper.de, `www.freemapper.de` redirecting to it, HTTPS enforced.

### How it is built

- **Zola** (Rust, one binary, current release v0.23.6, pinned by version and checksum in the workflow). It renders
  Markdown, has languages built in, needs no Node toolchain and fits the suite's Rust. Hugo would do the same; Jekyll
  (GitHub's default) is slower to keep up to date and has weaker language support; Astro would bring Node.
- The build checks out freemapper, freemapper-print and freemapper-courseplan at their **latest release tags**, so the
  manual on the site is the manual of the program one downloads. It copies:
  - `docs/manual/{de,en}.md` of each program, split at its chapter headings into pages;
  - `CHANGELOG.md` and `CHANGELOG.en.md`;
  - from freemapper `examples/*/preview*.webp`, the training and check-print PDFs and the examples overview, with the
    prose reworded for the site where the README speaks of GitHub;
  - the release data (tag, date, archive names and sizes, checksums) from the GitHub API, for the download buttons.
- Relative links between these files are rewritten to the site's pages; links to code stay links to GitHub.
- The workflow runs on a push to the site repository, once a day, and by hand. A later step lets the release workflow
  of each program trigger it right after a release (`repository_dispatch`, needs a fine-grained token as a secret,
  which only the user can create).
- Checks in CI: Zola's own link check, `lychee` for outside links (weekly, not blocking a deploy), an HTML validator,
  and a size budget (the start page under 500 KB with its images).

### How it looks

- **Inter** as variable font (`InterVariable.woff2`, `InterVariable-Italic.woff2` from Inter v4.1, SIL OFL 1.1),
  served from the site with `font-display: swap` and its licence file next to it. Tabular figures (`tnum`) in
  tables, Inter's `cv11` single-storey a off. Code in the system monospace font.
- The colours are the suite's tokens from freemapper-core's `suite.css` (core ADR 0030): `--bg --fg --line --accent`
  and the rest, the accent `#0b6bcb` light and `#6cb0ff` dark, with `prefers-color-scheme`. The site copies the
  token block from the pinned core tag, so the website and the programs' pages look alike.
- Plain HTML and CSS. JavaScript only for the language switch remembering a choice (and the page works without it).
- The map images are the hero: the training map of Viktoriapark on the start page, the example gallery as a grid of
  thumbnails that open the full image.
- Accessible: real headings, alt texts, contrast of the tokens checked (WCAG AA), visible focus, no motion under
  `prefers-reduced-motion`.

### Languages

- German at `/`, English at `/en/`, each page linked to its other language (`hreflang`). The browser's language does
  not redirect: a link someone shares opens in the language it was shared in.
- Texts of the site's own pages are written in both languages, in the plain style of the READMEs, and checked with
  the unslop and humanizer skills before they go online. German says "Bundesland".

### Pages

| Page | Content | Source |
|---|---|---|
| Start | What the suite makes, the Viktoriapark training map, the three programs as cards, "Does it work for my place?" (the states), download button, a line for AI assistants (AGENTS.md) | site + READMEs |
| freemapper | What it makes, map kinds, outputs, states table, permit notes, install per system, update | README, AGENTS.md install table, releases |
| freemapper-print | What it makes, state, install | its README, releases |
| freemapper-courseplan | What it makes, state, install | its README, releases |
| Beispiele / Examples | The 30 examples, by state and map kind, training map first, check print and PDF beside it | `examples/` |
| Handbuch / Manual | One section per program, a page per chapter, a table of contents | `docs/manual/*.md` |
| Änderungen / Changes | Per program | `CHANGELOG*.md` |
| Für KI-Assistenten | How to let an assistant install and run freemapper; links the AGENTS.md files | AGENTS.md |
| Lizenz und Daten | AGPL, the data licences and credit lines, the credit line rule | LICENSE, ATTRIBUTION examples |
| Impressum, Datenschutz | See "Left to the user" | user |

Every page carries the suite's credit line in its footer: freemapper and github.com/HenryJobst/freemapper.

### The domain

When freemapper.de is registered, its DNS gets:

- `A` records for `freemapper.de` to `185.199.108.153`, `185.199.109.153`, `185.199.110.153`, `185.199.111.153`;
- `AAAA` records to `2606:50c0:8000::153`, `2606:50c0:8001::153`, `2606:50c0:8002::153`, `2606:50c0:8003::153`;
- `CNAME` for `www` to `henryjobst.github.io`;
- the `TXT` record GitHub asks for when the domain is verified under the account's Pages settings (this keeps
  anyone else from claiming the domain on GitHub).

Verifying the domain and entering the DNS records needs the user's accounts; the site repository then names the domain
in its Pages settings.

## Left to the user

- **Impressum.** A German `.de` site run by a person usually names who is responsible (§ 5 DDG), with a postal
  address and a way to reach them. These are personal data; the site gets them only from the user, as text the user
  writes or approves. Whether this non-commercial project needs one is a legal question I cannot answer; I would add
  it.
- **Datenschutzerklärung.** GitHub Pages logs the visitors' IP addresses as host; the site sets no cookies and loads
  nothing from other hosts. The statement names GitHub as host and says that. The user approves the text.
- **Contact.** Whether the site names an e-mail address or only the GitHub issues.
- **Domain and DNS**, as above, and the token for the release trigger if it is wanted.

## Left to the builder

- The exact layout of the cards and the gallery, the order of the manual chapters on pages, the wording of the site's
  own texts within the rules above.
- How the chapter split handles a manual without chapter numbers (freemapper-print's and courseplan's are shorter).

## Order of work

1. Site repository with Zola, Inter, the suite tokens, start page in both languages, legal pages as placeholders,
   deploy to `henryjobst.github.io/freemapper-site/` for review. (No domain needed yet.)
2. Domain: DNS, verification, custom domain, HTTPS.
3. Program pages with the download buttons from the release data.
4. Example gallery.
5. Manuals and changes from the release tags.
6. The READMEs and AGENTS.md of the three programs link to freemapper.de; the release workflows trigger a rebuild.

Steps 1 to 5 each go in as a pull request in the site repository; step 6 is one pull request per program.
