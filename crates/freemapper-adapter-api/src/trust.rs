//! Trust anchors for a host whose certificate chain the default roots cannot verify.

/// The certificates one host is verified against, in place of the default roots.
///
/// A backend names them for a server of its provider that sends an incomplete chain. The HTTP
/// client then verifies exactly that host against exactly these certificates, and follows no
/// redirect of it: a followed redirect would keep the trust of the URL it started from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustAnchors {
    /// The host, as it appears in the URLs (`geodaten.schleswig-holstein.de`).
    pub host: &'static str,
    /// The trust anchors as PEM, each one certificate.
    pub pems: &'static [&'static [u8]],
}
