# Recorded answers of an adapter (fixtures)

This page describes the format. The [guide for contributors](en.md) says where recorded answers are used.

A recording lets freemapper run a fetch without the network: every request is answered from files. The golden net
of the built-in states uses one per recorded state. `freemapper adapters check` does not play recordings back;
it reads the folder and, with `--live`, asks the services. The code is `freemapper_sources::replay`.

## Layout

```
<adapter folder>/fixtures/
  index.toml            every request and the name of its answer
  <sha-256>.bin         one file per answer, named by the SHA-256 of its content
```

An answer that two requests share is kept once. The golden net keeps its answer files outside the repository,
in a fixture store, because the built-in states fetch whole tiles; the index has the same format there.

## The index

One `[[response]]` table per request, in the order the requests were first asked:

| Key | Meaning |
|---|---|
| `method` | `GET` or `HEAD` |
| `url` | the address asked, with its query |
| `range` | the `Range` header of a range request, e.g. `bytes=-65558` |
| `body` | the SHA-256 of the answer in lower-case hex: the name of its file without `.bin` |
| `bytes` | the size of the answer |
| `last_modified` | the date a `HEAD` answered with |
| `error` | the request failed: `not-found`, `refused`, `http`, `range-ignored` or `too-large` |
| `status`, `detail`, `message`, `limit_mb` | what belongs to that error |

Method, address and range together make a request. A request that is not in the index fails on playback like
a server error (`not in the recording`) and shows in the trace, so a new request is noticed.

## Rules

- An answer is named by its SHA-256 and nothing else. An index whose `body` is anything but 64 hex digits is
  refused, because the name becomes a path.
- On playback the content of an answer file is compared with its name; a changed file is an error.
- An OSM extract (`.osm.pbf`) is never recorded: it has 50 to 500 MB. The caller of the playback decides what
  such an address answers.
- A request whose address contains an `@` is not sent and not kept while recording. A recording is published;
  no address of a person belongs in it.
- Errors that are no answer of a service (a full disk, a cancel) are not kept.
