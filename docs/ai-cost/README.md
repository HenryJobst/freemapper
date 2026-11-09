# KI-Aufwand / AI effort

**Deutsch.** Das Badge in der README zeigt den Gegenwert aller Claude-Code-Sessions dieses Repositorys in
Claude-API-Listenpreisen der [offiziellen Preisseite](https://platform.claude.com/docs/en/about-claude/pricing), Subagenten
eingeschlossen. Die Preise liest der Dienst bei jedem Lauf neu. Läuft die Arbeit über ein Abo, ist die Zahl kein
Rechnungsbetrag, sondern zeigt nur, was dieselben Anfragen über die API gekostet hätten. Ein Dienst auf dem
Rechner des Entwicklers rechnet sie stündlich neu aus den Session-Logs aus. Jede Änderung landet als Zeile in
`history.csv`, der letzte Stand mit der Aufteilung nach Modell in `latest.json`.

**English.** The badge in the README shows what all Claude Code sessions of this repository, subagents
included, would cost at the Claude API list prices on the
[official pricing page](https://platform.claude.com/docs/en/about-claude/pricing), which the service reads on every run. When the work runs on a subscription, the number is not an
invoice amount; it only shows what the same requests would have cost through the API. A service on the
developer's machine recomputes it every hour from the session logs. Each change adds a line to
`history.csv`; `latest.json` holds the latest figure with the split by model.
