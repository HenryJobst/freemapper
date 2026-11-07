//! Pacing of Nominatim requests: at most one per second for the whole process (usage policy),
//! shared by the MCP server and the area selection page.

use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

const INTERVAL: Duration = Duration::from_secs(1);

static LAST: Mutex<Option<Instant>> = Mutex::new(None);

/// Claims the next request slot and returns how long to wait for it.
fn slot() -> Duration {
    let mut last = LAST.lock().unwrap_or_else(PoisonError::into_inner);
    let now = Instant::now();
    let wait = last
        .map(|t| INTERVAL.saturating_sub(now - t))
        .unwrap_or_default();
    *last = Some(now + wait);
    wait
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
        let first = slot();
        let second = slot();
        assert!(first <= INTERVAL);
        assert!(second >= INTERVAL - Duration::from_millis(50), "{second:?}");
    }
}
