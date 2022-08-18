//! This crate provides a zero-allocation, iterator/slice-based parser for extracting HTTP
//! [content encoding](https://tools.ietf.org/html/rfc7231#section-3.1.2) types as they
//! appear in the `Content-Encoding` request header. [Standard
//! encodings](http://www.iana.org/assignments/http-parameters/http-parameters.xhtml#content-coding)
//! are extracted as enum values, and unknown encodings are extracted as slices
//! for further processing.
//!
//! ## Example
//!
//! ```rust
//! use uhttp_content_encoding::{content_encodings, ContentEncoding, StdContentEncoding};
//!
//! let mut encs = content_encodings(" gzip, identity, custom-enc");
//! assert_eq!(encs.next(), Some(ContentEncoding::Other("custom-enc")));
//! assert_eq!(encs.next(), Some(ContentEncoding::Std(StdContentEncoding::Identity)));
//! assert_eq!(encs.next(), Some(ContentEncoding::Std(StdContentEncoding::Gzip)));
//! assert_eq!(encs.next(), None);
//! ```

/// Create an iterator over content encoding layers from the given string in [the
/// form](https://tools.ietf.org/html/rfc7231#section-3.1.2.2) used by the
/// `Content-Encoding` header field.
///
/// Encodings are yielded in the order they must be decoded, with the outermost layer
/// yielded first and the innermost layer yielded last.
pub fn content_encodings<'a>(s: &'a str) -> impl Iterator<Item = ContentEncoding<'a>> {
    s.split(',').rev().map(ContentEncoding::new)
}

/// HTTP content encoding scheme.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum ContentEncoding<'a> {
    /// Standard defined scheme.
    Std(StdContentEncoding),
    /// Unknown/nonstandard scheme with the contained name.
    ///
    /// This is guaranteed to have no surrounding whitespace and requires case-insensitive
    /// comparison to other strings.
    Other(&'a str),
}

impl<'a> ContentEncoding<'a> {
    /// Parse a new `ContentEncoding` from the given string.
    pub fn new(s: &'a str) -> Self {
        let s = s.trim();

        match s.parse() {
            Ok(enc) => ContentEncoding::Std(enc),
            Err(_) => ContentEncoding::Other(s),
        }
    }
}

/// Standard content encoding scheme, as defined by
/// [IANA](http://www.iana.org/assignments/http-parameters/http-parameters.xhtml#content-coding).
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum StdContentEncoding {
    /// Brottli compressed data format.
    Brottli,
    /// Unix "compress" data format.
    Compress,
    /// Deflate compressed data format.
    Deflate,
    /// W3C Efficient XML Interchange.
    EfficientXML,
    /// Gzip compressed data format.
    Gzip,
    /// No encoding.
    Identity,
    /// Java archive network transfer format.
    Pack200Gzip,
}

impl std::str::FromStr for StdContentEncoding {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use self::StdContentEncoding::*;

        // Values are case-insensitive [RFC7231§3.1.2.1].
        if s.eq_ignore_ascii_case("br") {
            Ok(Brottli)
        } else if s.eq_ignore_ascii_case("compress") {
            Ok(Compress)
        } else if s.eq_ignore_ascii_case("deflate") {
            Ok(Deflate)
        } else if s.eq_ignore_ascii_case("exi") {
            Ok(EfficientXML)
        } else if s.eq_ignore_ascii_case("gzip") {
            Ok(Gzip)
        } else if s.eq_ignore_ascii_case("identity") {
            Ok(Identity)
        } else if s.eq_ignore_ascii_case("pack200-gzip") {
            Ok(Pack200Gzip)
        } else if s.is_empty() {
            // Assume empty means identity [RFC7231§5.3.4].
            Ok(Identity)
        } else {
            Err(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_ce() {
        use self::StdContentEncoding::*;
        use self::ContentEncoding::*;

        assert_eq!(ContentEncoding::new("br"), Std(Brottli));
        assert_eq!(ContentEncoding::new("\t\t\rBr  "), Std(Brottli));
        assert_eq!(ContentEncoding::new("compress"), Std(Compress));
        assert_eq!(ContentEncoding::new("  COMpress "), Std(Compress));
        assert_eq!(ContentEncoding::new("deflate"), Std(Deflate));
        assert_eq!(ContentEncoding::new("\t\n dEFLAte "), Std(Deflate));
        assert_eq!(ContentEncoding::new("exi"), Std(EfficientXML));
        assert_eq!(ContentEncoding::new("\tEXI\t"), Std(EfficientXML));
        assert_eq!(ContentEncoding::new("gzip"), Std(Gzip));
        assert_eq!(ContentEncoding::new("  \tgZIP"), Std(Gzip));
        assert_eq!(ContentEncoding::new("identity"), Std(Identity));
        assert_eq!(ContentEncoding::new("\niDENtiTY\r\r\r "), Std(Identity));
        assert_eq!(ContentEncoding::new(""), Std(Identity));
        assert_eq!(ContentEncoding::new("    \t "), Std(Identity));
        assert_eq!(ContentEncoding::new("pack200-gzip"), Std(Pack200Gzip));
        assert_eq!(ContentEncoding::new("  PaCK200-GZip "), Std(Pack200Gzip));
        assert_eq!(ContentEncoding::new("ÆØБД❤"), Other("ÆØБД❤"));
    }

    #[test]
    fn test_ces() {
        use self::StdContentEncoding::*;
        use self::ContentEncoding::*;

        let mut ce = content_encodings("deflate, br, identity");
        assert_eq!(ce.next().unwrap(), Std(Identity));
        assert_eq!(ce.next().unwrap(), Std(Brottli));
        assert_eq!(ce.next().unwrap(), Std(Deflate));
        assert!(ce.next().is_none());

        let mut ce = content_encodings("identity");
        assert_eq!(ce.next().unwrap(), Std(Identity));
        assert!(ce.next().is_none());

        let mut ce = content_encodings("");
        assert_eq!(ce.next().unwrap(), Std(Identity));
        assert!(ce.next().is_none());

        let mut ce = content_encodings("\t\t,,            ,     ,");
        assert_eq!(ce.next().unwrap(), Std(Identity));
        assert_eq!(ce.next().unwrap(), Std(Identity));
        assert_eq!(ce.next().unwrap(), Std(Identity));
        assert_eq!(ce.next().unwrap(), Std(Identity));
        assert_eq!(ce.next().unwrap(), Std(Identity));
        assert!(ce.next().is_none());

        let mut ce = content_encodings("Br, exi,pack200-GZip   ");
        assert_eq!(ce.next().unwrap(), Std(Pack200Gzip));
        assert_eq!(ce.next().unwrap(), Std(EfficientXML));
        assert_eq!(ce.next().unwrap(), Std(Brottli));
        assert!(ce.next().is_none());

        let mut ce = content_encodings("\t\t\t   gzip");
        assert_eq!(ce.next().unwrap(), Std(Gzip));
        assert!(ce.next().is_none());

        let mut ce = content_encodings("\tabc\t\t, def  ");
        assert_eq!(ce.next().unwrap(), Other("def"));
        assert_eq!(ce.next().unwrap(), Other("abc"));
        assert!(ce.next().is_none());
    }
}
