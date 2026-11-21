// SPDX-License-Identifier: AGPL-3.0-or-later

//! The `.ocd` of an example, opened by the importer of OpenOrienteering Mapper, holds what the
//! `.omap` holds: colours, objects, scale and georeferencing (issue 230).
//!
//! The test needs `mapper-inspect`, a small program built against the Mapper sources
//! (`tools/mapper-inspect/README.md`). Set `FREEMAPPER_MAPPER_INSPECT` to its path; without it
//! the test says so and passes.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

const EXAMPLES: [&str; 2] = ["baden-wuerttemberg-heidelberg", "brandenburg-rauener-berge"];

fn inspect(tool: &Path, file: &Path) -> Value {
    let output = Command::new(tool)
        .arg(file)
        .env("QT_QPA_PLATFORM", "offscreen")
        .output()
        .expect("mapper-inspect starts");
    assert!(
        output.status.success(),
        "Mapper cannot open {}: {}",
        file.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("mapper-inspect prints JSON")
}

fn number(value: &Value, path: &[&str]) -> f64 {
    let mut at = value;
    for key in path {
        at = &at[*key];
    }
    at.as_f64()
        .unwrap_or_else(|| panic!("{path:?} is not a number"))
}

fn example_dir(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(name)
}

#[test]
fn mapper_reads_the_ocd_like_the_omap() {
    let Some(tool) = std::env::var_os("FREEMAPPER_MAPPER_INSPECT").map(PathBuf::from) else {
        eprintln!("FREEMAPPER_MAPPER_INSPECT is not set; the Mapper check is skipped");
        return;
    };
    for name in EXAMPLES {
        let dir = example_dir(name);
        let omap = inspect(&tool, &dir.join(format!("{name}.omap")));
        let ocd = inspect(&tool, &dir.join(format!("ocad/{name}.ocd")));

        assert_eq!(omap["objects"], ocd["objects"], "{name}: objects");
        assert_eq!(
            ocd["objects_without_symbol"], 0,
            "{name}: objects without a symbol"
        );
        assert_eq!(omap["scale"], ocd["scale"], "{name}: scale");
        assert_eq!(
            omap["colors"].as_array().map(Vec::len),
            ocd["colors"].as_array().map(Vec::len),
            "{name}: colours"
        );

        // State 2 is `Geospatial`; 1 would be `BrokenGeospatial` (CRS not usable).
        assert_eq!(ocd["georef"]["state"], 2, "{name}: georeferencing state");
        assert_eq!(
            omap["georef"]["state"], 2,
            "{name}: georeferencing state of the omap"
        );
        assert_eq!(
            omap["georef"]["crs_spec"], ocd["georef"]["crs_spec"],
            "{name}: CRS"
        );
        assert_eq!(
            omap["georef"]["scale"], ocd["georef"]["scale"],
            "{name}: map scale"
        );
        // OCAD keeps the reference point in whole metres.
        for axis in ["ref_x", "ref_y"] {
            let gap = (number(&omap, &["georef", axis]) - number(&ocd, &["georef", axis])).abs();
            assert!(gap < 1.0, "{name}: {axis} differs by {gap} m");
        }
        for axis in ["lat", "lon"] {
            let gap = (number(&omap, &["georef", axis]) - number(&ocd, &["georef", axis])).abs();
            assert!(gap < 1e-4, "{name}: {axis} differs by {gap}");
        }
        let declination =
            number(&omap, &["georef", "declination"]) - number(&ocd, &["georef", "declination"]);
        assert!(
            declination.abs() < 0.01,
            "{name}: declination differs by {declination}"
        );

        // The candidate layers must open too.
        for layer in std::fs::read_dir(dir.join("ocad")).expect("ocad folder") {
            let path = layer.expect("entry").path();
            if path.extension().is_some_and(|e| e == "ocd")
                && path
                    .file_name()
                    .is_some_and(|n| n.to_string_lossy().contains("-candidates-"))
            {
                inspect(&tool, &path);
            }
        }
    }
}
