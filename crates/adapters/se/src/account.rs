// SPDX-License-Identifier: AGPL-3.0-or-later

//! What a refused request of the download host means for the person: the Geotorget account has
//! not ordered the product, or it has used up what it may download.

use freemapper_adapter_api::error::FetchError;

/// The HTTP status a failed request carries, whichever way the fetcher reports it: a status
/// answer on a plain download comes as [`FetchError::Http`] with "http status: <code>" in its text.
fn status_of(error: &FetchError) -> Option<u16> {
    match error {
        FetchError::Refused { status, .. } => Some(*status),
        FetchError::Http { message, .. } => message
            .strip_prefix("http status: ")?
            .split(|c: char| !c.is_ascii_digit())
            .next()?
            .parse()
            .ok(),
        _ => None,
    }
}

/// Whether the error says the account may not download any more today: the download cap of
/// Geotorget (429). Another tile would be refused the same way.
pub(crate) fn is_cap(error: &FetchError) -> bool {
    status_of(error) == Some(429)
}

/// Whether the error says the account may not download the product (401 or 403).
pub(crate) fn is_not_ordered(error: &FetchError) -> bool {
    matches!(status_of(error), Some(401 | 403))
}

/// `error` as a message about the account where it is one, else unchanged. `order` is the name
/// of the product in Geotorget. The text starts with "http status: <code>;" so that it is not
/// taken for a busy service the next attempt may fix.
pub(crate) fn refusal(url: &str, order: &str, error: FetchError) -> FetchError {
    let Some(status) = status_of(&error) else {
        return error;
    };
    let message = match status {
        401 | 403 => format!(
            "http status: {status}; the account may not download this: order \"{order}\" in \
             Geotorget and check the user name and password in the environment variables"
        ),
        429 => format!(
            "http status: {status}; Lantmäteriet refused more downloads: the Geotorget account \
             has probably reached its download limit. Try again later; what is already \
             downloaded is kept"
        ),
        _ => return error,
    };
    FetchError::Http {
        url: url.to_string(),
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn http(message: &str) -> FetchError {
        FetchError::Http {
            url: "https://x/y".into(),
            message: message.into(),
        }
    }

    fn text(error: FetchError) -> String {
        error.to_string()
    }

    #[test]
    fn a_status_answer_of_a_plain_download_names_the_order() {
        for message in ["http status: 401", "http status: 403 Forbidden"] {
            let out = text(refusal("https://x/y", "Laserdata Skog", http(message)));
            assert!(out.contains("Laserdata Skog"), "{out}");
            assert!(out.contains("user name and password"), "{out}");
        }
    }

    #[test]
    fn a_refused_answer_names_the_order_too() {
        let refused = freemapper_adapter_api::fetch::status_answer_error("https://x/y", 401, b"");
        let out = text(refusal("https://x/y", "Marktäcke", refused));
        assert!(out.contains("Marktäcke"), "{out}");
    }

    #[test]
    fn too_many_requests_names_the_download_limit_not_a_busy_service() {
        let out = text(refusal(
            "https://x/y",
            "Marktäcke",
            http("http status: 429"),
        ));
        assert!(out.contains("download limit"), "{out}");
        assert!(!out.contains("overloaded"), "{out}");
        assert!(!out.contains("asked up to"), "{out}");
        assert!(
            !out.contains("Marktäcke"),
            "an order does not help against a cap: {out}"
        );
    }

    #[test]
    fn other_errors_stay_as_they_are() {
        for error in [
            http("http status: 404"),
            http("timed out"),
            FetchError::Cancelled,
        ] {
            let before = error.to_string();
            assert_eq!(text(refusal("https://x/y", "Marktäcke", error)), before);
        }
    }

    #[test]
    fn the_cap_is_told_from_the_missing_order() {
        assert!(is_cap(&http("http status: 429")));
        assert!(!is_cap(&http("http status: 401")));
        assert!(is_not_ordered(&http("http status: 403")));
        assert!(!is_not_ordered(&http("http status: 429")));
    }
}
