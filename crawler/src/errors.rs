// This is a new error type that you've created. It represents the ways a
// toolchain could be invalid.
//
// The custom derive for Fail derives an impl of both Fail and Display.
// We don't do any other magic like creating new types.
#[derive(Debug, Fail)]
pub enum StorkError {
    #[fail(display = "failed to parse url")]
    UrlParseError,
    #[fail(display = "failed to parse html")]
    HtmlParseError,
    #[fail(display = "failed to send http request")]
    HttpError,
}
