# Rules

| File | Source | Licence |
|---|---|---|
| `osm-candidates.toml` | freemapper. The last block of rules (marsh, hunting stands, feeding and watering places, cut lines, caves, bare rock, stones, sand, orchards, vineyards, bushes) follows the OSM tables of OpenOrienteering Mapper, `symbol sets/OSM-ISSprOM 2019.crt` and `symbol sets/OSM-ISOM 2017-2.crt`, commit `edd22fce3ec3975ce0f7ce06368ccccee9ac56c3` (2026-11-02), https://github.com/OpenOrienteering/mapper. Those tables are in turn based on `ISOM_rules.yaml` of osm2xmap by Semyon Yakimov (2016), https://github.com/sembruk/osm2xmap | AGPL-3.0-or-later; the Mapper tables are GPL-3.0-or-later ("version 3 or any later version" in their headers), which GPLv3 section 13 allows to combine with an AGPLv3 work |
| `adv-codes.toml` | freemapper | AGPL-3.0-or-later |
| `state-candidates.toml` | freemapper | AGPL-3.0-or-later |

freemapper took the pairs of an OSM tag and a symbol number from the Mapper tables, not the files themselves. Each
rule keeps its own conditions, targets and codes for the three map kinds, and every one of the new rules is a
candidate only.

The symbol sets are listed in [`../symbol-sets/README.md`](../symbol-sets/README.md).
