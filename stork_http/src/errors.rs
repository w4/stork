#[derive(Debug, Fail)]
pub enum StorkHttpError {
    #[fail(display = "failed to parse url")]
    UrlParseError,
    #[fail(display = "failed to parse html")]
    HtmlParseError,
    #[fail(display = "failed to send http request")]
    HttpError,
}
