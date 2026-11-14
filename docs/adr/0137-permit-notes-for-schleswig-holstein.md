# 0137. Permit notes for Schleswig-Holstein

Date: 2026-11-14
Status: Accepted

Research: [`nachrecherche-2026-11-14.md`](../research/wald-und-sprint-behoerden/nachrecherche-2026-11-14.md).
Builds on [0130](0130-permit-sources-as-tables.md) (table sources).

## Context

The environment service of Schleswig-Holstein (`umweltgeodienste.schleswig-holstein.de/WFS_UWAT`, deegree WFS 2.0.0)
holds the state forest of the Schleswig-Holsteinische Landesforsten with its Revier (`shlf_gesamtflaeche`: `revier`,
`reviername` in capitals), the Naturwald (`shlf_naturwald`) and the water protection zones (`wsg_zonen`: `wsg_name`,
`wsg_zone` such as "III B", `gebstatus`). It answers GML 3.2 only, and only when the output format is given in full
and URL-encoded; `gml32` is refused. The Umweltportal names CC BY 4.0 with the credit "LfU SH" for the state forest;
the capabilities give no conditions (`Fees` NONE, no access constraints).

## Decision

- The table source of ADR 0130 reads GML services too (`wfs_permits::gml_source`, with the output format as the
  service needs it); positions come latitude first and are turned round as for the other sources.
- A new reading `StateForest` gives the owner and the office of state forest; one layer of the state forest feeds
  both the state forest ("Bundesland Schleswig-Holstein", looked after by the Revier) and the forest units.
- Names written all in capitals become names ("HÜTTEN" → "Hütten") for forest units and state forest.
- The service joins the hosts of Schleswig-Holstein's manifest; the credit is the Umweltportal's for all four layers.

## Consequences

- Forest maps in Schleswig-Holstein name the Revier where the area is state forest; private and municipal forest
  stay without an office, as in Saarland.
- The licence of the Naturwald and water layers rests on the service's "no conditions" and the portal's licence of
  the service's main dataset; should the LfU say otherwise, the two layers come out.
