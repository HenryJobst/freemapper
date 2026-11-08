# 0038. Rhineland-Palatinate: orthophoto from the WMS, dated by its creation date

Date: 2026-11-08
Status: Accepted

## Context

LVermGeo RP publishes the orthophoto (DOP20) as 2 km blocks of lossless JPEG 2000, 105 to 155 MB each, with no tile
index in the codestream. freemapper has no JPEG 2000 decoder. The WMS `rp_dop20` serves the same pixels (mean difference
3.45 of 255), accepts up to 10000 px per side and is clipped at the state border, where the image turns white. No service
has a field for the flight date. GetFeatureInfo on `rp_dop20_info` gives one `erstellung` (creation) date per block. For
all 5,265 blocks that date falls between May and September, in runs of consecutive days, and its year is the year in the
block's file name.

## Decision

The orthophoto comes from the WMS in EPSG:25832, as JPEG pieces of at most 4000 px, through the shared WMS path. One
GetFeatureInfo at the centre of the map area reads the `erstellung` date, and freemapper takes it as the flight date. This
is an inference from the dates and the leaf-on images, and freemapper names it: the project record stores the date with
`flight_label = "Aktualität (Erstellung)"`, and the log calls it the date of creation
(`orthophoto unchanged (date of creation 25.08.2025)`). The other states keep the word "flight".

## Consequences

- No decoder, and no download of 100 MB or more per block.
- The date belongs to the block at the map centre; a map across two blocks can show two flights.
- Beyond the state border the orthophoto is white, while the height tiles reach across. The beyond-the-border message
  says that the templates are empty wherever the orthophoto and height models end.
- Cost if wrong: if `erstellung` is a processing date, the shown date is later than the flight. The record names the
  field, so a user can check it.

Source: Rhineland-Palatinate reference §4.1, §4.2, §4.3, §15; plan `docs/superpowers/plans/2026-11-08-rheinland-pfalz.md`
(Rulings); commits 6610f0f and 4dc59c4 on branch `rheinland-pfalz`.
