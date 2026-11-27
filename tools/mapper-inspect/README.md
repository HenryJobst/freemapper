# mapper-inspect

A small check tool, not part of the freemapper program. It opens an `.omap` or `.ocd` with the importer of
[OpenOrienteering Mapper](https://github.com/OpenOrienteering/mapper) and prints what Mapper read as JSON: format,
warnings, colours, symbols with their object counts, object total, scale and the georeferencing (state, CRS,
reference point, declination).

We own no OCAD, so Mapper is the only independent reader of our `.ocd`. The test
`crates/freemapper/tests/mapper_ocd.rs` runs this tool on the examples and compares the `.ocd` with the `.omap`.

## Licence

The source file is GPL-3.0-or-later like the Mapper sources it is built against. The tool is never part of a release
archive; the freemapper program stays AGPL-3.0-or-later.

## Build

Needs CMake, Ninja, Qt 5 (Core, Widgets, LinguistTools), PROJ and a C++ compiler. Mapper 0.9 does not build with Qt 6.
On macOS: `brew install qt@5 proj ninja cmake`.

```sh
git clone --depth 1 https://github.com/OpenOrienteering/mapper.git /path/to/mapper
tools/mapper-inspect/build.sh /path/to/mapper /path/to/build
# the binary: /path/to/build/build/src/mapper-inspect
```

The script copies the checkout, leaves out the manual and packaging and builds only the importer library and this
tool. GDAL is off; the first build takes several minutes.

## Use

```sh
QT_QPA_PLATFORM=offscreen mapper-inspect examples/baden-wuerttemberg-heidelberg/ocad/baden-wuerttemberg-heidelberg.ocd
FREEMAPPER_MAPPER_INSPECT=/path/to/build/build/src/mapper-inspect cargo test -p freemapper --test mapper_ocd
```

Without `FREEMAPPER_MAPPER_INSPECT` the test says so and passes.

## What differs on purpose

- The `.ocd` holds only the symbols in use; the `.omap` holds the whole set.
- Symbol numbers follow OCAD's decimal rule (`501.1` becomes `501.100`, see `docs/ocad/ocd12-reference.md` section 3.2).
- OCAD stores the reference point in whole metres.
