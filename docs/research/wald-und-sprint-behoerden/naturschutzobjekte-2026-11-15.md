# Naturdenkmale, geschützte Landschaftsbestandteile and §30 biotopes per state (research, 2026-11-15)

Research for the permit notes, done by research agents with curl and the fixed freemapper User-Agent; read-only, small
boxes.
I checked 15 states for three categories: Naturdenkmale (a), geschützte Landschaftsbestandteile, GLB (b), and gesetzlich geschützte Biotope (c). Berlin was checked for (c) only. A live state-wide service with a clear open licence exists in Brandenburg, Hamburg, Saarland, Rheinland-Pfalz, Bayern (a, b), Niedersachsen (a, b), Sachsen (c), NRW (c) and Berlin (c). The rest is incomplete, needs a question to the authority, or does not exist.

Every request was a read-only GET with the required User-Agent, no personal data, at most 50 features, and no bulk download (the largest was about 200 KB). Scratch files are under `/tmp/claude-502/natur/`. Nothing in the repository was changed. I tested BB, BE, HH and SL myself. Three helper agents covered SN/ST/MV/SH, NRW/NI/HB/HE and RP/BW/BY under the same rules. GDI-DE (`gdk.gdi-de.org`) did not answer plain GETs from here (curl 000 / ECONNRESET), so licences come from capabilities, metaver CSW and the state catalogues. Where a record could not be read, I say so.

## Brandenburg
- **(a) and (b): only via the Forst service.** WFS 2.0 `https://www.brandenburg-forst.de/geoserver/IWFK/ows`, output `application/json`.
  - Types: `IWFK:lkrs_nd_pu` (Point, Naturdenkmal) and `IWFK:lkrs_glb_fl` (Polygon, GLB).
  - Attributes: `landkreis`, `typ`, `name`, `art` (species), `merkmale`, `ort`, `lage`, `rvo_rgl` (ordinance), `quelle`.
  - Test, Potsdam box 52.30,13.0–52.45,13.2: HTTP 200, 15 ND (sample `name:'Schwarzpappel', art:'Schwarzpappel (Populus nigra)', rvo_rgl:'VO ND P 21.11.2005'`) and 1 GLB (`'Eichen an der Schleuse Klm'`, `rvo_rgl:'Beschluss 2002/0881 24.10.02'`). Strausberg box: 0 and 0.
  - `quelle` is 2017-12-31 throughout, so the data are old, and coverage looks incomplete (unverified).
  - Licence: capabilities Fees/AccessConstraints "NONE". Metaver record 52B5CF01-2B76-4F07-8F46-87D6E76B7E9F ("Waldfunktionen des Landes Brandenburg (INSPIRE)"): "Datenlizenz Deutschland Namensnennung 2.0", "Quellenvermerk: Landesbetrieb Forst Brandenburg".
  - No personal data seen. `lage` holds site addresses.
- **(c) LfU biotope register.** WFS 2.0 `https://inspire.brandenburg.de/services/bbk_wfs`, output `application/geo+json`.
  - Types: `app:bbk_fl` (Polygon), `app:bbk_li` (LineString), `app:bbk_pu` (Point).
  - Attributes: `pk_ident`, `biotyp`, `biotyp_t` (kind), `bbgnat_t` (legal status, e.g. "geschützter Biotop nach § 18"), `ffhlrt_t`, `datum_e`.
  - Test, Strausberg box 52.56,13.85–52.59,13.90: HTTP 200, 26 polygons, 10 lines, 0 points. Sample: `biotyp_t:'perennierende Kleingewässer (Sölle, Kolke, Pfuhle etc., < 1 ha), naturnah, beschattet', bbgnat_t:'geschützter Biotop nach § 18'`.
  - Licence, capabilities AccessConstraints: "Es gelten die Bedingungen der Datenlizenz Deutschland – Namensnennung – Version 2.0: https://www.govdata.de/dl-de/by-2-0. Als Bezeichnung des Bereitstellers ist „© Landesamt für Umwelt Brandenburg“ anzugeben." Fees: "Nutzung erfolgt derzeit kostenfrei unter Beachtung des Urheberrechts." Metaver record A061BB02-70AC-4422-BB58-4A49F585D7F2 confirms dl-de/by-2-0.
  - The LfU's `schutzg_wfs` has no ND or GLB layer.
  - No personal data.

## Berlin (only c)
- WFS 2.0 `https://gdi.berlin.de/services/wfs/ua_biotoptypen_2024`, output `application/json`.
  - Types: `ua_biotoptypen_2024:bc_gesetzschutz_f` (MultiPolygon), `bb_gesetzschutz_l` (MultiLineString), `ba_gesetzschutz_p` (MultiPoint).
  - Attributes: `biotop_id`, `bt_code`, `bt_name` (kind), `schutz_ges`, `kart_jahr`.
  - `schutz_ges` codes, from the Datenformatbeschreibung PDF: "0 = kein Schutz, 1 = sicher geschützt, 2 = Verdachtsfläche". The status follows the Umweltatlas mapping, so this is not a legal register.
  - Test, Grunewald box 52.43,13.20–52.46,13.25: HTTP 200, 50 polygons (capped), 1 line, 6 points. Sample: `bt_name:'Kiefern-Vorwald trockener Standorte', schutz_ges:2`.
  - Licence, capabilities Fees: "Für die Nutzung der Daten ist die Datenlizenz Deutschland - Zero - Version 2.0 anzuwenden." No personal data.

## Hamburg
- **(a)** WFS 2.0 `https://geodienste.hamburg.de/HH_WFS_Schutzgebiete`, type `de.hh.up:nr_f_naturdenkmal`, output `application/geo+json`.
  - Polygons, 12 state-wide (HTTP 200).
  - Attributes: `schutzstatus` ND, `gebietsname`, `rechtsgrundlage`, `verordnung`, `url_rechtsgrundlage`, `link_info`, `feature_text`.
  - Sample: `gebietsname:"ND Garten de L'Aigles", rechtsgrundlage:'HmbGVBl. 1994, S. 109'`.
  - Licence, Fees: "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und Hansestadt Hamburg, Behörde für Umwelt und Energie". AccessConstraints: "Es gelten keine Zugriffsbeschränkungen".
  - `feature_text` can name long-dead historical persons. No current personal data.
- **(b)** No GLB service found. The Baumschutzverordnung 2023 makes trees of 80 cm or more in circumference GLB in law, with no geometry. Old ordinances on "Landschaftsteile" are found as texts only (lexaris).
- **(c)** WFS 2.0 `https://geodienste.hamburg.de/HH_WFS_Biotopkataster`.
  - Types: `de.hh.up:flaechenhafte_biotope_geschuetzt`, `linienhafte_biotope_geschuetzt`, `punkthafte_biotope_geschuetzt`, plus matching `*_nicht_geschuetzt` layers.
  - Attributes: `hauptbiotoptyp`, `paragraf` (e.g. "§ 30 (2) 3.4 Trockenrasen", "§ 14 (2) 2.2 Knicks"), `biotop_erfasst_am`, `biotopbogen` (PDF link).
  - Test, Blankenese box 53.55,9.78–53.58,9.86: HTTP 200, 50 polygons (capped), 23 lines, 2 points.
  - Licence: Fees "Datenlizenz Deutschland Namensnennung 2.0, Quellenvermerk: Freie und Hansestadt Hamburg, Behörde für Umwelt, Klima, Energie und Agrarwirtschaft". Metaver record D7B5CCBB-1F03-4482-AB59-26F2F7972547 says the same.
  - No personal data.

## Saarland
All three come as WFS 2.0 with `outputFormat=GEOJSON`. Capabilities: Fees "Es gelten keine Bedingungen", AccessConstraints "Es gelten keine Zugriffsbeschränkungen".
- **(a)** `https://geoportal.saarland.de/arcgis/services/Internet/Schutzgebiete_INS/MapServer/WFSServer` (also in `Internet/Naturschutz`), type `Naturdenkmal_Flaechen`.
  - MultiPolygon. Trees are small circles of about 80 m².
  - Attributes: `KENNUNG`, `NAME`, `ART` (e.g. "Einzelbaum", "Felswand"), `BM_ART` (species), `RECHTSGR` (ordinance PDF), `AUSWEISUNG`, `METADATEN`.
  - Test, Saarbrücken box 49.20,6.95–49.26,7.05: HTTP 200, 30 features. Sample: `NAME:'1 Buche, 104', ART:'Einzelbaum', BM_ART:'Fagus sylvatica', RECHTSGR:'…/ND_Satzung_Saarbruecken_2008.pdf'`.
  - Licence, metaver record 30c42c7c-19fc-45bb-90cc-8d066019f736: "Creative Commons Namensnennung – 4.0 International (CC BY 4.0)", "Quellenvermerk: © GDI-SL (Jahr)".
- **(b)** `…/Internet/Naturschutz/MapServer/WFSServer`, type `Naturschutz:Geschuetzte_Landschaftsbestand` (MultiPolygon).
  - Same attributes as (a), plus `SCHUTZZWECK` and `BEMERKUNG`.
  - Same box: 2 features. One carries `BEMERKUNG` "Satzung wurde vom OVG-Saarlouis … für unwirksam erklärt", so filter on `BEMERKUNG`.
  - That remark mentions "Eigentümerin", a private owner, though not by name.
  - Licence: same CC BY 4.0 series (record 6490142d-1a0e-45a3-a799-3df062ddb024).
- **(c)** `…/Internet/Biotopkataster/MapServer/WFSServer`, types `Biotopkataster:Geschuetzte_Biotope_f` (MultiPolygon) and `Geschuetzte_Biotope_l` (MultiLineString). There is also `Lebensraumtypen_fl`.
  - Attributes: `KENNUNG`, `BEZEICHNUNG`, `REPORT` (an HTML link that holds the biotope types). There is no kind field in the WFS.
  - Same box: 50 polygons (capped), 2 lines.
  - Licence: CC BY 4.0, "© GDI-SL (Jahr)", per the GDI-DE record 0b645b03-7d75-3e4c-aaba-69a41b504c75 as a search snippet quotes it. I could not open the record myself.
  - The mapping is selective, not complete.

## Sachsen
- **(a)** Statewide, areas only: LfULG `https://luis.sachsen.de/arcgis/rest/services/natur/schutzgebiete/MapServer/2` "Flaechennaturdenkmaeler" (Polygon). Also served as WFS 2.0 at `…/arcgis/services/natur/schutzgebiete/MapServer/WFSServer`.
  - Attributes: `NAME`, `SG_NR`, `STATUS`, `RVO_DATUM`, `RVO_1`/`RVO_2` (ordinance and the body that issued it).
  - Test, Dresden: HTTP 200, 16 features ("Bienertwiese").
  - Licence: Fees "Datenlizenz Deutschland 2.0 - Variante mit Namensnennung", source "Sächsisches Landesamt für Umwelt, Landwirtschaft und Geologie" (GDI-DE 9490f0e1-…).
  - Point (tree) Naturdenkmale are kept by the districts. Example: Landkreis Mittelsachsen WFS `https://webgis.landkreis-mittelsachsen.de/wfs/schutzgebiete` (`misa:nd_festgesetzt` and others). Test Freiberg: 2 features. Its licence is unclear: "Nutzungsbedingungen: Bedingungen unbekannt".
- **(b)** Not available state-wide. Only district records exist, e.g. Plauen.
- **(c)** `https://luis.sachsen.de/arcgis/rest/services/natur/gesetz_gesch_biotope/MapServer` (also WFS 2.0).
  - Layers 0, 1, 2: point, line and polygon. Layer 2 has 70,713 objects.
  - Attributes: `BTTP26_BEZ` (kind), `BTP26_LID`, `BTP26_DAT_`. No name, act or authority.
  - Test, Leipzig Auwald: 50 features (capped).
  - Licence: the same dl-de/by-2-0 (GDI-DE f1fa40a0-…). No personal data.

## Sachsen-Anhalt
- **(a) and (b)** LAU WFS 2.0 `https://geodatenportal.sachsen-anhalt.de/ows_lau_wfs_naturschutz`, GEOJSON.
  - Types: `ps:Flächenhafte_Naturdenkmale` (MultiPolygon), `ps:Flächennaturdenkmale` (Point) and `ps:Geschützte_Landschaftsbestandteile` (MultiPolygon).
  - Attributes: `CODE_ST`, `Gebietsname`, `STATUS`, `Erstausweisung`, `Metadaten`.
  - Test, Halle: 7 NDF, 4 FND, 4 GLB (e.g. "Amtsgarten").
  - Licence: capabilities AccessConstraints "Alle Rechte beim Landesamt fuer Umweltschutz Sachsen-Anhalt." GDI-DE 30BC0527-…: "Datenlizenz Deutschland Namensnennung 2.0", "Quellenvermerk: Landesamt für Umweltschutz Sachsen-Anhalt".
  - Flag: the two statements conflict a little. Most likely the dl-de/by-2-0 grant applies.
  - There is no layer for single tree Naturdenkmale.
- **(c)** Not available. The LAU sends people to the lower authorities (UNB), and the data come only "nach Anerkennung der Nutzungsbedingungen" with a handover protocol.

## Mecklenburg-Vorpommern
- The old `umweltkarten.mv-regierung.de` WFS answers HTTP 410. The new one is `https://umweltkarten.lung-mv.de/dienste/<name>`: MapServer WFS 1.1.0. `OUTPUTFORMAT=geojson` works but is not advertised. The bbox must be given in EPSG:5650.
- **(a)** `ng_schutzgebiete`, type `fnd_f` (flächige ND only).
  - Attributes: `NAME`, `LEG_TITEL`, `LEG_LINK`, `ORGAN_1`, `JAHR`.
  - Test, Greifswald: 4 features.
  - Personal data: `ORGAN_1` sometimes contains the name of an office holder ("Die Landrätin … <Name>"). Drop the name.
  - Tree points are kept by the districts. Example: Ludwigslust-Parchim deegree WFS `ps:ProtectedSite`, geometry only, dl-de/by-2-0.
- **(b)** Not available.
- **(c)** `nb_biotope`, type `bvz26_biotope` "Biotopverzeichnis 2026" (MultiPolygon).
  - Attributes: `bvz_nr`, `biotopname`, `ges_nr`/`ges_name` (category of legal protection), `unb` (lower authority in charge), `kart_jahr`.
  - Test, Greifswald: 50 features (capped).
- **Licence conflict** (flag): the capabilities say "Lizenz CC BY-SA". The GDI-DE record says "CC-BY-SA, Quellenvermerk: © LUNG M-V [Jahr des Datenabrufs] (CC BY-SA)". The LUNG AGB say CC BY-SA 3.0. The LUNG Umweltkarten page says CC BY 4.0. One question to LUNG would settle it.

## Schleswig-Holstein
- **(a) and (b)** INSPIRE WFS 2.0 `https://service.gdi-sh.de/SH_INSPIREDOWNLOAD_AI_PS`. GML 3.2 only (GeoJSON gives HTTP 400). EPSG:4258 with lat/lon axis order.
  - Types: `Naturdenkmale` (48 state-wide, mostly points) and `geschuetzte_Landschaftsbestandteile` (34).
  - Attributes: `localId`, `legalFoundationDate`, `text` (name or species).
  - Tests: Schlei, 1 ND ("6 Stieleichen an der Schlei …"); Kiel, 1 GLB.
  - Licence, Fees: "Creative Commons (CC BY 4.0) – Namensnennung 4.0 International … Der Quellenvermerk lautet "© GeoBasis-DE/LVermGeo SH/CC BY 4.0"".
  - Coverage: 47 of the 48 ND come from Kreis Schleswig-Flensburg.
- **(c)** No service. Only shapefile downloads on opendata.schleswig-holstein.de, with conflicting licences ("other-closed" versus cc-by/4.0). The Biotopportal needs a login. The LfU says its mapped biotopes "sind nicht gleichzusetzen mit gesetzlich geschützten Biotopen".

## Nordrhein-Westfalen
- **(a) and (b)** No state-wide service; the Kreise and Städte keep them in their Landschaftspläne.
  - Example: Bielefeld WFS 2.0 `https://www.bielefeld01.de/md/WFS/naturdenkmale/01`, GeoJSON. Types `ms:nd_innen_p` and others, plus `ms:nd_glb_lp_*`.
  - Test: 26 ND (`art_deutsc:'Stiel-Eiche'`).
  - Licence: CC BY 4.0, "Stadt Bielefeld - Umweltamt" (open.nrw).
  - `adresse` holds the address of the tree's site.
- **(c)** WFS 2.0 `https://www.wfs.nrw.de/umwelt/linfos`, `wfs_linfos:gbt_polygon`/`gbt_polyline`/`gbt_point`, `OUTPUTFORMAT=GEOJSON`.
  - Attributes: only `localid`. The biotope kind is in an HTML report (WMS GetFeatureInfo `LINK`).
  - Test, Bielefeld: 8 polygons.
  - Licence: the dataset says "Datenlizenz Deutschland – Zero – Version 2.0" (GDI-DE 759B42FB-…). The WMS says dl-de/by-2-0 with "Land NRW". Crediting "Land NRW" satisfies both.

## Niedersachsen
- **(a) and (b)** ArcGIS REST `https://www.umweltkarten-niedersachsen.de/arcgis/rest/services/Natur_wms/MapServer`, `f=geojson`, `outSR=4326`.
  - ND layers: 190 (Polygon), 6 (Line), 7 (Point, 2,710 objects).
  - GLB layers: 10 (343 polygons), 11, 12. The low GLB count suggests gaps.
  - Attributes: `kennz_ffn`, `name`, `kategorie`, `vollzug` (authority in charge), `recht_vom`, `veroeff_in` (where the act was published).
  - Tests: Lüneburg and Göttingen returned features (e.g. 19 ND points in Göttingen).
  - Licence: "Datenlizenz Deutschland – Namensnennung – Version 2.0", source NLWKN (GDI-DE B4D711F4…). The records also say the geometry is "nur auf Grundlage der DTK25 aussagekräftig".
- **(c)** No state-wide register; the districts keep it. The only partial substitute is layer 163, "Gesetzlich geschützte Biotope mit Moor- und Torfanteil".

## Bremen
- **(a) and (b)** Not available. Landtag paper Drs. 21/1406 (2025) asks for a digital list of Naturdenkmale in the first place.
- **(c)** No WFS:
  - WMS `https://geodienste.bremen.de/wms_geschuetzte_biotope` (GetFeatureInfo as GML or text only).
  - Download `https://gdi2.geo.bremen.de/inspire/download/Geschuetzte_Biotope/data/Geschuetzte_Biotope_HB_BHV.zip` (1.65 MB, 1,475 polygons).
  - Attributes: `Gebietsbez`, `vorherr_BT`, `geschuetzt`.
  - Licence, Fees: "Creative Commons Namensnennung (CC-BY), Quellenvermerk: Freie Hansestadt Bremen, Die Senatorin für Umwelt, Klima und Wissenschaft". The CC version is not stated.

## Hessen
- **(a)** REST `https://geodienste-umwelt.hessen.de/arcgis/rest/services/natureg/schutzgebiete/MapServer/0`, `f=geojson`.
  - Polygons only, 1,986 objects; trees are small circles.
  - Attributes: `NAME`, `NATUREG_NR`, `BEHOERDE` (e.g. "UNB Stadt Fulda").
  - Test, Fulda: 39 features.
  - Licence: only the copyright text "Datenbereitstellung: HLNUG / © HMLU". No metadata record found.
  - Sibling records carry HMLU terms. Free use is allowed "für gewerbliche/kommerzielle Nutzung, sofern nicht die Daten … weiterveräußert werden", and the "Erstellung eines „eigenen" Werks … ist zulässig". This is not confirmed for the ND layer.
- **(b)** Not available state-wide. Only regional records, e.g. Regionalverband FNP.
- **(c)** WFS 2.0 `https://geodienste-umwelt.hessen.de/arcgis/services/inspire/lebensraeume_biotope/MapServer/WFSServer`, `inspire_lebensraeume_biotope:GESCH_BIOTOPE`, GEOJSON.
  - Old mapping, 1992–2006. Attributes `BIOTOPNAME`, `BIOTYP_BEZ`, `Schutz`.
  - Test, Marburg: 22 features.
  - Licence, Fees: "… für private und kommerzielle Zwecke erlaubt. Lizenz: Creative Commons BY 4.0 … Quellenvermerk für Daten: „Darstellung auf der Grundlage von Daten des Hessischen Landesamtes für Naturschutz, Umwelt und Geologie (HLNUG)"".
  - The newer mapping (HLBK from 2014) is `natureg/biotope_lebensraeume/MapServer/8`. It is partial, carries `GGBT_GES` (legal basis), and its licence is unverified.

## Rheinland-Pfalz
- All three come as WFS 2.0 `https://geoportal.naturschutz.rlp.de/kartendienst_naturschutz/apps/service/wfs_getmap.php?mapfile=<name>`, `application/json; subtype=geojson`. The old host `geodaten.naturschutz.rlp.de` redirects there with 301.
  - (a) `ms:naturdenkmale` (Polygon) and `ms:naturdenkmale_punkte` (Point).
  - (b) `ms:glb` and `ms:glb_p`.
  - (c) `ms:geschuetztes_biotop_f`/`_l`/`_p`. The layer title itself says "(unvollständig)".
  - Attributes: `kennung`, `bezeichnung` (name or tree species), `rvolink` (ordinance PDF); for (c) `longname` (biotope type).
  - Tests, Trier-Olewig: 3 ND polygons, 50 ND points (capped), 2 GLB, 4/5/1 biotopes.
- Licence, LANIS metadata id 4889 and others: "Nutzung entsprechend den Lizenzbestimmungen der Odbl mit Quellenangabe zulässig", with source "Landesamt für Umwelt Rheinland-Pfalz".
- ODbL is share-alike for derived databases (`permits.json`), the same as OSM.

## Baden-Württemberg
- **(a)** LUBW, ArcGIS REST `https://rips-gdi.lubw.baden-wuerttemberg.de/arcgis/rest/services/wfs/Naturdenkmal_Einzelgebilde/MapServer/0` (END, Multipoint, 7,535 objects) and `…/wfs/Naturdenkmal_Flaechenhaft/…` (FND, Polygon, 5,974). `f=geojson`; also WFS 2.0.
  - Attributes: `OBJEKT` (name or species), `STATUS`, `VO_DATUM`, `DIENSTSTELLE_NR` (number only).
  - Tests: Tübingen 2 END, Freiburg 2 FND.
  - Credit, from the UIS Nutzungsvereinbarung: „Grundlage: Daten aus dem Umweltinformationssystem (UIS) der LUBW Landesanstalt für Umwelt Baden-Württemberg".
- **(b)** Not available. The municipalities designate GLB; per Landtag paper 17/9526, "der Landesregierung liegen hierzu keine Daten vor" (taken from a search result, not read in the PDF).
- **(c)** `…/wfs/Biotopkartierung/MapServer/0`, "Biotop nach BNatSchG/NatSchG/LWaldG". It covers both Offenland and Wald.
  - Attributes: `BIOTOP`, `BIOTOPTYP_NATSCHG_GRUPPE`, `KARTIERUNGSTYP`.
  - Test, Tübingen: 50 features (capped).
  - Restriction, verbatim from `https://www.lubw.baden-wuerttemberg.de/umweltinformationssystem/nutzungsvereinbarung`: **"Für den Datensatz Geschützte Biotope (Biotopkartierung) ist darüber hinaus eine kommerzielle Nutzung oder Veräußerung der Daten nicht pauschal zulässig, sie kann jedoch auf Anfrage gewährt werden."**
  - The metadata say: "In der Objektart sind personenbezogene Daten vorhanden". The public fields seen hold none.
  - Alternative for forest only: FVA INSPIRE WFS (`owsproxy.lgl-bw.de/.../WFS_INSP_BW_Lebensraeume_Biotope_Waldbiotopkartierung`). It is GML only, has no legal status field, and is under dl-de/by-2-0 "Datenquelle: FVA, www.fva-bw.de". But the FVA WMS record says commercial use needs an agreement.

## Bayern
- **(a) and (b)** WFS 2.0 `https://www.lfu.bayern.de/gdi/wfs/natur/schutzgebiete`, GEOJSON.
  - Types: `natur_wfsschutzgebiete:naturdenkmal_punktfoermig` (4,860), `naturdenkmal_flaechig` (1,811), `landschaftsbestandteil_flaechig` (1,532), `landschaftsbestandteil_punktfoermig` (185).
  - Attributes: `id`, `name`, `status`. No ordinance or authority field.
  - Test, Regensburg: 1 ND point, 18 ND areas, 1 GLB area.
  - Licence, Fees: "Creative Commons Namensnennung 4.0 International (CC BY 4.0) … "Datenquelle: Bayerisches Landesamt für Umwelt, www.lfu.bayern.de"".
- **(c)** No WFS:
  - WMS `https://www.lfu.bayern.de/gdi/wms/natur/biotopkartierung` (`bio_fbk`, `bio_sbk`, `bio_abk`). GetFeatureInfo returns geo+json but without geometry. It carries the field "Anteil Schutz Par.30 Art.23".
  - Geometry comes only from the ATOM shapefile download (Flachland 228 MB). That is over the limit, so I did not download it.
  - Licence: CC BY 4.0.
  - No biotopes mapped inside forests since 1993.

## Ranking

**Ready now, live service with a clear open licence**
- **(a) Naturdenkmale:** BB (Forst, 2018 data), HH, SL, SN (areas only), ST (areas), NI, RP (ODbL), BW (UIS credit), BY. SH works but covers almost only Schleswig-Flensburg.
- **(b) GLB:** BB (Forst), SL, ST, NI, RP (ODbL), BY. SH is incomplete.
- **(c) Biotope:** BB, BE (dl-de/zero, includes suspected areas), HH, SL, SN, NRW (kind only via an extra report lookup), HE (old INSPIRE mapping, CC BY 4.0), RP (incomplete, ODbL). HB is ready on the licence but only as a 1.65 MB zip or WMS.

**Needs a question to the authority**
- MV (a areas, c): CC BY-SA or CC BY 4.0?
- HE (a): no licence stated for the ND layer. HE (c, HLBK layer): licence unverified.
- BW (c): commercial use of the LUBW biotope data only "auf Anfrage".
- SH (c): no service, conflicting download licences.
- SN, MV point Naturdenkmale: district by district (Mittelsachsen licence "unbekannt").
- ST (a, b): small wording conflict, "Alle Rechte beim LAU" versus dl-de/by-2-0.

**Not available state-wide**
- (a): NRW (cities such as Bielefeld, CC BY 4.0), HB, ST tree points.
- (b): NRW, HB, HE, BW, SN, MV, HH.
- (c): ST (on request only), NI (districts; peat subset only), BY (WMS without geometry or a 228 MB download).

Not verified:
- How complete the BB Forst ND/GLB layers are.
- The NI GLB coverage.
- The HLBK licence.
- The SN field `BTP26_ZUS_`.
- What BW's `DIENSTSTELLE_NR` numbers stand for.

Sources:
- [GDI-DE CIR-Biotoptypen BB](https://gdk.gdi-de.org/geonetwork/srv/api/records/A7FBE31E-8C90-4DCC-81FB-699CE9536441)
- [Berlin Umweltatlas Biotoptypen 2024](https://www.berlin.de/umweltatlas/biotope/biotoptypen/2024/karten/)
- [Berlin gesetzlich geschützte Biotope](https://www.berlin.de/sen/uvk/natur-und-gruen/naturschutz/biotopschutz/gesetzlich-geschuetzte-biotope/)
- [Hamburg Baumschutzverordnung 2023](https://www.bund-hamburg.de/fileadmin/hamburg/Themen/Baumschutz/baumschutzverordnung-2023.pdf)
- [GDI-DE Saarland Geschützte Biotope](https://gdk.gdi-de.org/geonetwork/srv/api/records/0b645b03-7d75-3e4c-aaba-69a41b504c75)
- [Saarland Schutzgebietskataster](https://geoportal.saarland.de/article/Schutzgebietskataster/)
- [Bielefeld ND/GLB (open.nrw)](https://ckan.open.nrw.de/dataset/naturdenkmale-und-geschutzte-landschaftsbestandteile-bi)
- [NRW geschützte Biotope (open.nrw)](https://ckan.open.nrw.de/ca/dataset/759b42fb-24ee-4123-91a8-9e518396bbbc)
- [NLWKN gesetzlich geschützte Biotope](https://www.nlwkn.niedersachsen.de/startseite/naturschutz/biotopschutz/gesetzlich_geschutzte_biotope/gesetzlich-geschutzte-biotope-und-landschaftsbestandteile-in-niedersachsen-205667.html)
- [HLNUG Nutzungsbedingung](https://hebid.hessen.de/bdd/hlbk/Dokumente/Nutzungsbedingung_Fachdaten_HLNUG.pdf)
- [GDI-DE Hessen Naturparke](https://gdk.gdi-de.org/geonetwork/srv/api/records/8897f44d-33fe-c2d0-3568-4e4e02b4a980)
- [Bremen geschützte Biotope](https://inspire-geoportal.ec.europa.eu/srv/api/records/466CC833-7D60-4B0E-9CDE-07AC60EC9CC0)
- [Bremische Bürgerschaft Drs. 21/1406](https://www.bremische-buergerschaft.de/dokumente/wp21/land/drucksache/D21L1406.pdf)
- [LUBW Nutzungsvereinbarung](https://www.lubw.baden-wuerttemberg.de/umweltinformationssystem/nutzungsvereinbarung)
- [LAU ST digitale Daten](https://lau.sachsen-anhalt.de/fachthemen/naturschutz/naturschutzinformationen/digitale-daten)
