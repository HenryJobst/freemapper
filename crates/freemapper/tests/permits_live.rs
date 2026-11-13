//! The time the permit notes take on a real project, against the real BfN and BKG services and
//! the real cache. Ignored: it asks the services. Run it with the folder of a copy of a project
//! whose OSM extract is cached:
//!
//! `FREEMAPPER_PERMITS_LIVE_DIR=/path/to/copy cargo test -p freemapper --test permits_live -- --ignored --nocapture`

use std::path::PathBuf;
use std::time::Instant;

use freemapper::fetch::{Cache, HttpFetcher};
use freemapper::permits::fetch::fetch_inputs;
use freemapper::permits::notes::SourceInput;
use freemapper::permits::store::write_permit_notes;
use freemapper::project::ProjectFile;

fn count<T>(input: &SourceInput<T>) -> String {
    match input {
        SourceInput::Read { items, fetched } => format!("{} read ({fetched})", items.len()),
        SourceInput::Unreachable { reason } => format!("unreachable: {reason}"),
        SourceInput::NotRead => "not read".to_string(),
    }
}

#[test]
#[ignore = "asks the BfN and the BKG; needs FREEMAPPER_PERMITS_LIVE_DIR"]
fn live_permit_notes_of_a_project() {
    let dir = PathBuf::from(
        std::env::var("FREEMAPPER_PERMITS_LIVE_DIR").expect("FREEMAPPER_PERMITS_LIVE_DIR"),
    );
    let project = ProjectFile::load(&dir).unwrap();
    let cache = Cache::default_location().expect("a cache folder");
    let fetcher = HttpFetcher::new();
    let started = Instant::now();
    let inputs = fetch_inputs(&project, &dir, &fetcher, &cache, false, &mut |note| {
        println!("{note:?}")
    })
    .unwrap();
    let fetched = started.elapsed();
    let notes = write_permit_notes(&dir, &project, &inputs, freemapper::clock::today()).unwrap();
    let total = started.elapsed();
    println!("fetch_inputs: {:.1} s", fetched.as_secs_f64());
    println!(
        "fetch_inputs + write_permit_notes: {:.1} s",
        total.as_secs_f64()
    );
    println!(
        "vg250: {}, bfn: {}, osm: {}",
        count(&inputs.admin),
        count(&inputs.protected),
        count(&inputs.osm)
    );
    for share in &notes.admin {
        println!(
            "admin {:?} {} ({}) {} %{}",
            share.item.level,
            share.item.name,
            share.item.kind,
            share.percent,
            if share.at_edge { " at edge" } else { "" }
        );
    }
    for share in &notes.protected {
        println!(
            "protected {:?} {} {} %{}",
            share.item.kind,
            share.item.name,
            share.percent,
            if share.at_edge { " at edge" } else { "" }
        );
    }
    for share in &notes.osm_protected {
        println!(
            "osm protected {} {:?} {} %",
            share.item.osm_id, share.item.name, share.percent
        );
    }
    for operator in &notes.operators {
        println!(
            "operator {} {:?} {:?}",
            operator.name, operator.roles, operator.objects
        );
    }
    for hint in &notes.hints {
        println!("hint {} {:?}", hint.key, hint.args);
    }
}
