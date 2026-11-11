//! Metalink documents (RFC 5854): the list of the files a download directory offers, as the
//! portal of Rhineland-Palatinate publishes one per height model.

use crate::fetch::one_line;

/// The names of the files a Metalink lists (`<file name="…">`), in the order of the document.
/// An answer that is not a Metalink (an HTML page, say) is an error.
pub fn file_names(text: &str) -> Result<Vec<&str>, String> {
    let start = text.trim_start().trim_start_matches('\u{feff}');
    let root = start
        .strip_prefix("<?xml")
        .and_then(|rest| rest.split_once("?>"))
        .map_or(start, |(_, rest)| rest)
        .trim_start();
    if !root.starts_with("<metalink") {
        return Err(format!("not a Metalink: {}", one_line(text, 200)));
    }
    Ok(text
        .split("<file name=\"")
        .skip(1)
        .filter_map(|element| element.split('"').next())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const METALINK: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
        <metalink xmlns=\"urn:ietf:params:xml:ns:metalink\">\n\
        <published>2026-08-12 09:59:48+00:00</published>\n\
        <file name=\"dgm1_32_400_5579_1_rp_2024.tif\"><size>3</size></file>\n\
        <file name=\"readme.txt\"><size>1</size></file>\n\
        </metalink>";

    #[test]
    fn a_metalink_lists_its_files_in_order() {
        assert_eq!(
            file_names(METALINK).unwrap(),
            ["dgm1_32_400_5579_1_rp_2024.tif", "readme.txt"]
        );
        // With a byte order mark, and without the XML declaration.
        assert_eq!(file_names(&format!("\u{feff}{METALINK}")).unwrap().len(), 2);
        let bare = METALINK.split_once("?>\n").unwrap().1;
        assert_eq!(file_names(bare).unwrap().len(), 2);
        assert_eq!(
            file_names("<metalink></metalink>").unwrap(),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn an_answer_that_is_no_metalink_is_an_error() {
        let error = file_names("<html>Proxy\nError</html>").unwrap_err();
        assert_eq!(error, "not a Metalink: <html>Proxy Error</html>");
    }
}
