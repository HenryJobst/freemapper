# ADR 0201: Permit notes in Czechia

Date: 2026-11-27

Status: Accepted

Refs: issue #356, ADR 0175, ADR 0181, ADR 0183, ADR 0184, ADR 0189, ADR 0197.

## Context

ADR 0197 left the permit notes of Czechia open. Like Poland (ADR 0175), Czechia has open map services for the questions the notes ask, and a law of its own: the German national sources (VG250, BfN) and the German authorities do not apply. The engine already writes notes for a region without them when an adapter brings permit sources; the adapter `cesko` brought none, so `freemapper permits` refused with "not available for this region yet".

All services below were probed by hand on 27 November 2026 with the fixed research User-Agent, without a login, over https.

## Decision

- **Three adapter sources**, in `crates/adapters/cz/src/permits.rs`: the adapter has no manifest (ADR 0197), and the declared tables of ADR 0189 need a manifest and a factory in the German adapter crate, so the sources are code, as in Poland. The adapter revision is 2; `gis.nature.cz` and `geoportal.npu.cz` are the new hosts next to `ags.cuzk.gov.cz`.
- **Protected areas and objects (`aopk`).** The ArcGIS REST services of the Nature Conservation Agency (AOPK ČR) at `https://gis.nature.cz/arcgis/rest/services`: `UzemniOchrana/ChranUzemi` (layer 0 small protected areas NPR, NPP, PR, PP by the attribute `KAT`; layer 1 their protective zones; layer 3 national parks NP, protected landscape areas CHKO and the protective zone of a national park `OP`; layer 2 the zones of nature protection of NP and CHKO), `UzemniOchrana/Natura2000` (layer 1 sites for habitats EVL, layer 0 bird areas PO) and `PamatneStromy/PamatneStromy` (layer 1 memorial trees as points, layer 3 groups of trees without positions as polygons). The query is `query` with the box in WGS 84 (`inSR=4326`, `outSR=4326`), `f=geojson`, `maxAllowableOffset=0.00005` (the boundary of a protected landscape area drops from 77 KB to 28 KB), `orderByFields=OBJECTID`, paged by 1000 while the service says `exceededTransferLimit`. An error object in an answer with status 200 is an error. The services answer 2000 records at most. Eleven new `ProtectedObjectKind`s carry the Czech categories, because the German and Polish labels name German or Polish terms.
- **Administrative units (`ruian-hranice`).** RÚIAN (`ags.cuzk.gov.cz/arcgis/rest/services/RUIAN/MapServer`, layers 12 obec, 13 obec with an authorised municipal office, 14 obec s rozšířenou působností (ORP), 15 okres, 17 kraj). The fetch asks the municipalities of the map box with their polygons and the four name layers without geometry, and follows `obec.pou` to `pou.orp` to the ORP, `obec.okres` to the district and `okres.vusc` to the region. The capital has no district; its ORP carries the region. Each municipality gives four findings (region, district, ORP, municipality) with its own polygon, so the notes sum the shares as for Poland (ADR 0181). Records that ended (`platido`) or are marked wrong (`nespravny`) are left out. `AdminUnitLevel` gets `AuthorityDistrict` for the ORP, between the county and the municipality; the code of a unit prints as "RÚIAN 547786" in Czechia and "TERYT" in Poland.
- **Monuments (`npu-pamatky`).** The INSPIRE service of the protected sites of the National Heritage Institute (NPÚ, `geoportal.npu.cz/arcgis/rest/services/INSPIRE/ProtectedSites/MapServer`, layer 0) with the true polygons. The notes count them in the Monuments section by kind: `monument` is a cultural monument (kulturní památka), `settlement` a monument reservation or zone, `cultural` a site of the world heritage list. The service holds no protective zones of monuments, which the source says in its `gaps`.
- **Hints.** `permits-hint-protected-areas-cz`, `-monuments-cz`, `-forest-owner-cz` and `-roads-cz`, in all 21 languages (English, German and Czech by hand, the 18 others drafted by machine and unreviewed, as agreed in ADR 0175). The engine picks them by the id of the adapter (`cesko`); every other region without the German national sources keeps the Polish hints. The ORP of the map and the municipalities with their shares are arguments. Each hint cites its provision, and only what was read in the text of the law on `zakonyprolidi.cz` on 27 November 2026:
  - Nature Conservation Act 114/1992 Sb.: § 16 odst. 1 písm. q (events in a national park outside the places set aside are banned), § 37 odst. 3 písm. d (mass events in the protective zone of a national park need consent), § 29 písm. d (in a national nature reserve entering or driving outside marked paths is banned), § 75 odst. 1 (the nature conservation authorities).
  - Act on State Heritage Care 20/1987 Sb., § 14 odst. 1 and 2 (the owner asks the ORP office, for a national cultural monument the regional office, for a decision or a binding opinion), and § 29.
  - Act on Forests 289/1995 Sb., § 20 odst. 5: a mass sports, cultural or other social event on land meant for forest functions only with the written consent of the forest owner, and a written notice to the office of the ORP (the regional office if the event goes beyond one ORP) at the latest 30 days ahead, with the owners' consents.
  - Act on Roads 13/1997 Sb., § 25 odst. 1 and 6 písm. d (special use of a road for events that could endanger traffic needs the permit of the road administration office, with the consent of the road's owner and, where traffic is affected, of the police), § 40 odst. 4 and 5 (which office).
  The hints say that the Acts name no run or control as such where that is so, and they are a hint, not legal advice, like those of the other countries.
- **Fixtures** are real attributes of the services (Adršpach, Prague) with the geometry replaced by the bounding rectangle of the feature, as for Poland. The offline path (`--offline`) is the generic one of the engine and needs no code.

## Not built, and why

- **Forest owners and forest management plans.** The Forest Management Institute (ÚHÚL) serves its geoportal on `geoportal.uhul.cz` and `www.uhul.cz`; on 27 November 2026 the first answers with a certificate whose name does not match the host and the second with a chain that does not verify ("unable to get local issuer certificate"). A client that checks certificates, as freemapper does, cannot ask them. No other open service of forest owners was found in this probe (Lesy ČR, the state forest enterprise, was not examined further). The hint therefore says that the owner's consent is needed and asks the person to find the owners; it names no forest office. Revisit when the certificate is fixed.
- **Natural parks (přírodní parky) and the protective zones of memorial trees.** Not asked: no event rule for them was read in the Act. The source says that the natural parks are not in it.
- **Other hints of the Polish set** (land of the municipality, nature objects set by the municipality): no provision was read for them, so none is written.

## Licences

- **AOPK ČR.** The metadata record of its INSPIRE protected sites on `geoportal.gov.cz` (record `58e3584c-5dd8-488e-8bef-73280a02080a`) names "Creative Commons BY 4.0" and "no limitations to public access"; a presentation of the agency describes `data.nature.cz` as its open data catalogue under CC BY 4.0. The credit is "© AOPK ČR (CC BY 4.0)". The metadata of the exact map services (`ChranUzemi`, `Natura2000`, `PamatneStromy`) name only "© AOPK ČR" (`copyrightText`); that they carry the same licence is an inference to confirm with the agency.
- **ČÚZK (RÚIAN).** CC BY 4.0, as in ADR 0197.
- **NPÚ.** The metadata record of the INSPIRE service of the protected sites (`geoportal.npu.cz/metadataserver`, record `61af2686-4bb8-4253-8bb0-0dacc0a8cc15`) names "Creative Commons BY-SA 4.0". The credit is "© Národní památkový ústav (CC BY-SA 4.0)". ShareAlike for the notes (`permits.md`), which carry only counts and kinds from this service, is open, as for the Lower Saxony monuments. The richer ÚAP service of the same institute (`Tematicke/CP_UAP_PVO`) has no licence in its metadata and was not used.

## Consequences

- `freemapper permits` works in Czechia. The notes list the administrative units (kraj, okres, ORP, obec), the protected areas and objects, the cultural monuments and the OSM operators, with the four hints.
- The Czech hints and the Czech labels of the kinds exist in 21 languages; only English, German and Czech were written by a person who read the Acts. A Czech reader should review them before release.
- A new `AdminUnitLevel` and eleven kinds change `permits.json` only for Czech notes.
