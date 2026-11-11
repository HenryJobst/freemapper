//! Moved to `freemapper-sources` (ADR 0082); this module keeps the old paths working.

pub use freemapper_sources::wms::*;

// The tests of the reader that take a state's service as their example stay here until the
// states are adapters.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crs::BoundingBox;
    use crate::fetch::test_support::{MapFetcher, solid_jpeg};
    use crate::raster::GridSpec;
    use crate::sources::{geosn, hvbg, lgv, lvermgeo};

    fn grid(width: u32, height: u32) -> GridSpec {
        GridSpec {
            west: 476_000.0,
            north: 5_553_000.0,
            pixel_size: ORTHOPHOTO_PIXEL_M,
            width,
            height,
        }
    }

    #[test]
    fn hvbg_wms_pieces_are_at_most_3000_px() {
        let pieces = pieces(&hvbg::WMS, &grid(7000, 3500));
        let sizes: Vec<(u32, u32)> = pieces.iter().map(|(g, _)| (g.width, g.height)).collect();
        assert_eq!(
            sizes,
            [
                (3000, 3000),
                (3000, 500),
                (3000, 3000),
                (3000, 500),
                (1000, 3000),
                (1000, 500)
            ]
        );
        assert!(pieces[0].1.contains("LAYERS=he_dop20_rgb"));
        assert!(pieces[0].1.contains("CRS=EPSG:25832"));
        assert!(pieces[0].1.contains("WIDTH=3000&HEIGHT=3000"));
    }

    #[test]
    fn other_wms_pieces_stay_4000_px() {
        for service in [geosn::WMS, lvermgeo::WMS, lgv::WMS] {
            let pieces = pieces(&service, &grid(9000, 5000));
            assert_eq!(pieces.len(), 6, "{}", service.url);
            assert!(
                pieces
                    .iter()
                    .all(|(g, _)| g.width <= 4000 && g.height <= 4000),
                "{}",
                service.url
            );
            assert_eq!((pieces[0].0.width, pieces[0].0.height), (4000, 4000));
        }
    }

    fn bbox() -> BoundingBox {
        BoundingBox {
            min_e: 476_000.0,
            min_n: 5_552_800.0,
            max_e: 476_200.0,
            max_n: 5_553_000.0,
        }
    }

    #[test]
    fn xml_wms_answer_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &hvbg::WMS.url,
            br#"<?xml version="1.0" encoding="UTF-8"?>
<ServiceExceptionReport version="1.3.0"><ServiceException code="InvalidDimensionValue">
msWMSLoadGetMapParams(): WMS server error. Image size out of range, WIDTH and HEIGHT must be between 1 and 3000 pixels.
</ServiceException></ServiceExceptionReport>"#
                .to_vec(),
        );
        let error = fetch_orthophoto(
            &fetcher,
            &hvbg::WMS,
            &bbox(),
            dir.path(),
            &dir.path().join("o.jpg"),
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("Image size out of range") && !error.contains("JPEG decode"),
            "{error}"
        );
        assert!(error.contains("the WMS answered with an error"), "{error}");
        // The same exception would come every time: asked once.
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn xml_wms_answer_names_the_inner_service_message() {
        let message = "msWMSLoadGetMapParams(): WMS server error. Image size out of range, \
                       WIDTH and HEIGHT must be between 1 and 3000 pixels.";
        let report = format!(
            "<?xml version='1.0' encoding=\"UTF-8\" standalone=\"no\" ?>\n\
             <!DOCTYPE ServiceExceptionReport SYSTEM \"http://schemas.opengis.net/wms/1.3.0/exception_1_3_0.dtd\">\n\
             <ServiceExceptionReport version=\"1.3.0\" xmlns=\"http://www.opengis.net/ogc\">\n\
             <ServiceException code=\"InvalidDimensionValue\">\n{message}\n</ServiceException>\n\
             </ServiceExceptionReport>"
        );
        assert_eq!(xml_message(&report).as_deref(), Some(message));
        let ows = "<ows:ExceptionReport><ows:Exception exceptionCode=\"X\">\
                   <ows:ExceptionText>Layer  not\n found</ows:ExceptionText></ows:Exception></ows:ExceptionReport>";
        assert_eq!(xml_message(ows).as_deref(), Some("Layer not found"));
        assert_eq!(xml_message("\u{89}PNG"), None);
    }

    #[test]
    fn non_jpeg_non_xml_answer_is_an_error_not_a_panic() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        // Invalid UTF-8 becomes 3-byte replacement characters; byte 80 of that is mid-character.
        let mut png = vec![0x89];
        png.extend(b"PNG\r\n\x1a\n");
        png.extend([0x89u8; 60]);
        fetcher.insert_prefix(&hvbg::WMS.url, png);
        let error = fetch_orthophoto(
            &fetcher,
            &hvbg::WMS,
            &bbox(),
            dir.path(),
            &dir.path().join("o.jpg"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("not a JPEG"), "{error}");
        assert!(!error.contains("the WMS answered"), "{error}");
        assert!(error.contains("not a JPEG ("), "{error}");
    }

    fn dop_source() -> OrthophotoSource {
        OrthophotoSource::Wcs(crate::sources::laiv_mv::DOP)
    }

    #[test]
    fn dop_wcs_pieces_stitch_on_the_grid() {
        // 4500 × 1000 px: two pieces side by side, 4000 and 500 px wide.
        let grid = GridSpec {
            west: 312_000.0,
            north: 5_996_400.0,
            pixel_size: ORTHOPHOTO_PIXEL_M,
            width: 4500,
            height: 1000,
        };
        let pieces = source_pieces(&dop_source(), &grid);
        assert_eq!(pieces.len(), 2);
        let base = "https://www.geodaten-mv.de/dienste/inspire_oi_dop_wcs?SERVICE=WCS&VERSION=2.0.1\
                    &REQUEST=GetCoverage&COVERAGEID=mv_dop20&FORMAT=image/jpeg";
        assert_eq!(
            pieces[0].1,
            format!("{base}&SUBSET=x(312000,312800)&SUBSET=y(5996200,5996400)")
        );
        assert_eq!(
            pieces[1].1,
            format!("{base}&SUBSET=x(312800,312900)&SUBSET=y(5996200,5996400)")
        );
        // Whole multiples of 0.2 m, no gap and no overlap.
        assert_eq!((pieces[0].0.width, pieces[1].0.width), (4000, 500));
        assert_eq!(pieces[1].0.west, pieces[0].0.bounds().max_e);
        for (g, _) in &pieces {
            let steps = g.west / ORTHOPHOTO_PIXEL_M;
            assert!((steps - steps.round()).abs() < 1e-6, "{}", g.west);
        }

        // Stitch: a box of 4500 × 1000 px in the same grid, served as two JPEGs of distinct colours.
        let bbox = grid.bounds();
        let fetcher = MapFetcher::default();
        fetcher.insert(&pieces[0].1, solid_jpeg(4000, 1000, [200, 30, 30]));
        fetcher.insert(&pieces[1].1, solid_jpeg(500, 1000, [30, 30, 200]));
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("o.jpg");
        fetch_orthophoto_from(&fetcher, &dop_source(), &bbox, out.parent().unwrap(), &out).unwrap();
        let image = image::open(&out).unwrap().into_rgb8();
        assert_eq!((image.width(), image.height()), (4500, 1000));
        let near =
            |p: &image::Rgb<u8>, c: [u8; 3]| p.0.iter().zip(c).all(|(a, b)| a.abs_diff(b) < 12);
        for (x, c) in [
            (0, [200, 30, 30]),
            (3995, [200, 30, 30]),
            (4005, [30, 30, 200]),
            (4499, [30, 30, 200]),
        ] {
            assert!(
                near(image.get_pixel(x, 500), c),
                "x={x}: {:?}",
                image.get_pixel(x, 500)
            );
        }
        // The seam is at pixel 4000 exactly.
        assert!(near(image.get_pixel(3999, 10), [200, 30, 30]));
        assert!(near(image.get_pixel(4001, 10), [30, 30, 200]));
        assert!(out.with_extension("jgw").exists());
        assert_eq!(fetcher.calls.get(), 2);
    }

    #[test]
    fn xml_dop_wcs_answer_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &crate::sources::laiv_mv::DOP.url,
            br#"<?xml version="1.0" encoding="UTF-8"?>
<ows:ExceptionReport xmlns:ows="http://www.opengis.net/ows/2.0" version="2.0.1"><ows:Exception exceptionCode="InvalidSubsetting"><ows:ExceptionText>subset out of range</ows:ExceptionText></ows:Exception></ows:ExceptionReport>"#
                .to_vec(),
        );
        let error = fetch_orthophoto_from(
            &fetcher,
            &dop_source(),
            &mbbox(),
            dir.path(),
            &dir.path().join("o.jpg"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("the WCS answered with an error"), "{error}");
        assert!(
            error.contains("InvalidSubsetting: subset out of range"),
            "{error}"
        );
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn wrong_size_dop_piece_is_an_error() {
        let grid = GridSpec {
            west: 312_000.0,
            north: 5_996_400.0,
            pixel_size: ORTHOPHOTO_PIXEL_M,
            width: 4000,
            height: 100,
        };
        let fetcher = MapFetcher::default();
        let (_, url) = &source_pieces(&dop_source(), &grid)[0];
        fetcher.insert(url, solid_jpeg(3999, 100, [1, 2, 3]));
        let dir = tempfile::tempdir().unwrap();
        let error = fetch_orthophoto_from(
            &fetcher,
            &dop_source(),
            &grid.bounds(),
            dir.path(),
            &dir.path().join("o.jpg"),
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("3999 × 100 pixels instead of 4000 × 100"),
            "{error}"
        );
        // A wrong size would come again: final, not retried.
        assert_eq!(fetcher.calls.get(), 1);
    }

    #[test]
    fn dop_wcs_error_text_is_one_clean_line() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &crate::sources::laiv_mv::DOP.url,
            "<html>\x1b]0;owned\x07\u{202E}evil\nline two</html>"
                .as_bytes()
                .to_vec(),
        );
        let error = fetch_orthophoto_from(
            &fetcher,
            &dop_source(),
            &mbbox(),
            dir.path(),
            &dir.path().join("o.jpg"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("not a JPEG"), "{error}");
        assert!(
            !error.chars().any(|c| c.is_control() || c == '\u{202E}'),
            "{error:?}"
        );
    }

    #[test]
    fn wms_error_text_is_one_clean_line() {
        let dir = tempfile::tempdir().unwrap();
        let fetcher = MapFetcher::default();
        fetcher.insert_prefix(
            &hvbg::WMS.url,
            "<ServiceExceptionReport>bad\x1b]0;owned\x07\u{202E}x\nthing</ServiceExceptionReport>"
                .as_bytes()
                .to_vec(),
        );
        let error = fetch_orthophoto(
            &fetcher,
            &hvbg::WMS,
            &bbox(),
            dir.path(),
            &dir.path().join("o.jpg"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("the WMS answered"), "{error}");
        assert!(
            !error.chars().any(|c| c.is_control() || c == '\u{202E}'),
            "{error:?}"
        );
    }

    fn mbbox() -> BoundingBox {
        BoundingBox {
            min_e: 312_057.0,
            min_n: 5_996_217.0,
            max_e: 312_257.0,
            max_n: 5_996_417.0,
        }
    }

    #[test]
    #[ignore = "live: asks the LAiV M-V DOP WCS"]
    fn live_rostock_school_orthophoto() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("o.jpg");
        let fetcher = crate::fetch::HttpFetcher::new();
        fetch_orthophoto_from(
            &fetcher,
            &dop_source(),
            &mbbox(),
            out.parent().unwrap(),
            &out,
        )
        .unwrap();
        let image = image::open(&out).unwrap().into_rgb8();
        assert_eq!((image.width(), image.height()), (1000, 1000));
        let jgw = std::fs::read_to_string(out.with_extension("jgw")).unwrap();
        let v: Vec<f64> = jgw.lines().map(|l| l.trim().parse().unwrap()).collect();
        let expected = GridSpec::covering(&mbbox(), ORTHOPHOTO_PIXEL_M).world_file();
        eprintln!("jgw:\n{jgw}expected:\n{expected}");
        assert!(
            (v[0] - 0.2).abs() < 1e-9 && (v[3] + 0.2).abs() < 1e-9,
            "{v:?}"
        );
        assert!(
            (v[4] - 312_057.1).abs() < 0.001 && (v[5] - 5_996_416.9).abs() < 0.001,
            "{v:?}"
        );
    }

    #[test]
    fn stalled_wms_request_times_out_and_is_retried() {
        use std::io::{Read as _, Write as _};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url: &'static str =
            Box::leak(format!("http://{}/wms", listener.local_addr().unwrap()).into_boxed_str());
        let jpeg = solid_jpeg(1000, 1000, [1, 2, 3]);
        std::thread::spawn(move || {
            // The first connection is held without an answer; the second gets the image.
            let (_stalled, _) = listener.accept().unwrap();
            let (mut conn, _) = listener.accept().unwrap();
            let mut request = [0u8; 4096];
            let _ = conn.read(&mut request);
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                jpeg.len()
            );
            conn.write_all(head.as_bytes()).unwrap();
            conn.write_all(&jpeg).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(2));
        });
        let service = WmsService {
            url: std::borrow::Cow::Borrowed(url),
            ..hvbg::WMS
        };
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("o.jpg");
        let fetcher =
            crate::fetch::HttpFetcher::with_timeout(std::time::Duration::from_millis(300));
        let (sender, receiver) = std::sync::mpsc::channel();
        let path = out.clone();
        std::thread::spawn(move || {
            sender.send(fetch_orthophoto(
                &fetcher,
                &service,
                &bbox(),
                path.parent().unwrap(),
                &path,
            ))
        });
        receiver
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("the stalled request did not time out")
            .unwrap();
        assert!(out.exists());
    }

    #[test]
    fn k5_template_has_world_file() {
        // 1100 m × 300 m at 0.25 m: two GetMap pieces of at most 4000 px.
        let bbox = BoundingBox {
            min_e: 392_000.0,
            min_n: 5_820_000.0,
            max_e: 393_100.0,
            max_n: 5_820_300.0,
        };
        let grid = GridSpec::covering(&bbox, K5_PIXEL_M);
        let fetcher = MapFetcher::default();
        let pieces = k5_pieces(&grid);
        assert_eq!(pieces.len(), 2);
        for (piece, url) in &pieces {
            assert!(
                url.starts_with("https://gdi.berlin.de/services/wms/k5_farbe?"),
                "{url}"
            );
            assert!(url.contains("LAYERS=k5_farbe"), "{url}");
            assert!(url.contains("CRS=EPSG:25833"), "{url}");
            assert!(url.contains("FORMAT=image/png"), "{url}");
            fetcher.insert(url, solid_png(piece.width, piece.height, [200, 180, 160]));
        }
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("k5.png");
        fetch_k5(&fetcher, &bbox, K5_PIXEL_M, out.parent().unwrap(), &out).unwrap();
        let png = image::open(&out).unwrap().into_rgb8();
        assert_eq!((png.width(), png.height()), (grid.width, grid.height));
        assert_eq!(png.get_pixel(grid.width - 1, 0).0, [200, 180, 160]);
        let world = std::fs::read_to_string(out.with_extension("pgw")).unwrap();
        let stored = GridSpec::from_world_file(&world, png.width(), png.height()).unwrap();
        assert!((stored.west - grid.west).abs() < 1e-6 && (stored.north - grid.north).abs() < 1e-6);
        assert!((stored.pixel_size - 0.25).abs() < 1e-12);
        // An XML error report is no image.
        let broken = MapFetcher::default();
        for (_, url) in &pieces {
            broken.insert(
                url,
                b"<ServiceExceptionReport>no layer</ServiceExceptionReport>".to_vec(),
            );
        }
        let err = fetch_k5(
            &broken,
            &bbox,
            K5_PIXEL_M,
            dir.path(),
            &dir.path().join("x.png"),
        )
        .unwrap_err();
        assert!(err.to_string().contains("no layer"), "{err}");
    }

    #[test]
    fn k5_image_is_capped_at_6000_px() {
        // 3000 m at 0.25 m would be 12 000 px: the pixels grow to 0.5 m.
        let bbox = BoundingBox {
            min_e: 392_000.0,
            min_n: 5_820_000.0,
            max_e: 395_000.0,
            max_n: 5_820_100.0,
        };
        let grid = GridSpec::covering(&bbox, 0.5);
        let fetcher = MapFetcher::default();
        for (piece, url) in k5_pieces(&grid) {
            fetcher.insert(&url, solid_png(piece.width, piece.height, [1, 2, 3]));
        }
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("k5.png");
        fetch_k5(&fetcher, &bbox, K5_PIXEL_M, out.parent().unwrap(), &out).unwrap();
        let png = image::open(&out).unwrap();
        assert_eq!(png.width(), 6000);
        let world = std::fs::read_to_string(out.with_extension("pgw")).unwrap();
        let stored = GridSpec::from_world_file(&world, png.width(), png.height()).unwrap();
        assert!((stored.pixel_size - 0.5).abs() < 1e-12, "{stored:?}");
    }

    fn solid_png(width: u32, height: u32, rgb: [u8; 3]) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(width, height, image::Rgb(rgb));
        let mut bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();
        bytes
    }
}
