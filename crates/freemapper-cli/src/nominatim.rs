//! Pacing of Nominatim requests: at most one per second for the whole process (usage policy),
//! shared by the MCP server and the area selection page.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

const INTERVAL: Duration = Duration::from_secs(1);

/// Start of the latest claimed request slot.
static LAST: Mutex<Option<Instant>> = Mutex::new(None);

/// Claims the next request slot and returns how long to wait for it.
fn slot() -> Duration {
    let mut last = LAST.lock().unwrap_or_else(PoisonError::into_inner);
    let now = Instant::now();
    // `last` is the start of the latest claimed slot, possibly in the future.
    let next = last.map_or(now, |t| (t + INTERVAL).max(now));
    *last = Some(next);
    next - now
}

/// Waits until a Nominatim request is allowed.
pub async fn wait_turn() {
    tokio::time::sleep(slot()).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_are_spaced_one_second_apart() {
        // Three requests at once get slots one, two and three intervals after the first.
        let first = slot();
        let second = slot();
        let third = slot();
        assert!(first <= INTERVAL);
        let tolerance = Duration::from_millis(50);
        assert!(second >= first + INTERVAL - tolerance, "{second:?}");
        assert!(third >= second + INTERVAL - tolerance, "{third:?}");
    }
}
