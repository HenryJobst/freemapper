# Corrections of addresses

`freemapper-sources.json` is the list of corrections of addresses that programs fetch with
`freemapper sources update` (issue #273, ADR 0189). It is signed and published by the workflow
[`sources-corrections.yml`](../.github/workflows/sources-corrections.yml), never by hand.

A correction names one address of a built-in manifest, as `freemapper sources list` shows it:

```json
{
  "adapter": "hamburg",
  "path": "alkis.fallback.url",
  "from": "https://old.example.org/wfs",
  "to": "https://new.example.org/wfs",
  "add_host": true,
  "note": "LGV moved the service (reported in issue #123)"
}
```

- `from` is the address the shipped manifest holds. A program applies the correction only where it
  still finds it, so a program that already has the new address, or a different one, is left alone.
- `add_host` is needed when the host of `to` is not among the hosts of the manifest; the host then
  joins them. A correction that would break the manifest is refused alone.
- `issued` is the day the list was made. A program never replaces a stored list by an older one:
  raise it with every change.

To publish a change: change the list in a pull request (a test checks that every correction still
applies to the shipped manifests), merge it, then start the workflow `sources-corrections` and
approve the environment `release`. A correction that a release has brought into the manifests is
deleted from the list in that release's pull request; the test fails until it is.
