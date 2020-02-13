//! # stork_http
//! This is a [stork](../../../stork/) implementation for the HTTP
//! protocol and specifically HTML-based web scraping. Given an initial
//! page to scrape, stork_http will find all indexable links on the page
//! and yield them back to you - ready to scrape again in an instant
//! or store for later to come back to at another time, all using futures
//! to allow for parallel processing.
//!
//! At this time `rel="nofollow"` is strictly enforced and not possible
//! to change although this will come in time as more filters are added.
//!
//! Example usage:
//!
//! ```
//! # use stork::FilterSet;
//! # use failure::err_msg;
//! # use stork_http::{HttpStorkable, filters::*};
//! # use futures::StreamExt;
//! #
//! # #[tokio::main]
//! # async fn main() -> failure::Fallible<()> {
//! // start scanning https://example.com/ for links with the given filters
//! let stream = HttpStorkable::new("https://example.com/".parse()?)
//!     .with_filters(
//!         FilterSet::default()
//!             .add_filter(DomainFilter::new("www.iana.org"))
//!             .add_filter(SchemeFilter::new("https"))
//!     )
//!     .exec();
//! # futures::pin_mut!(stream); // needed for iteration
//! // get the first link from example.com and ensure its the one we expected
//! // it to be
//! let first_link_on_example: HttpStorkable = match stream.next().await {
//!     Some(Ok(link)) => {
//!         assert_eq!(link.val().text(), Some("More information...".to_string()));
//!         assert_eq!(link.val().url().as_str(), "https://www.iana.org/domains/example");
//!         assert_eq!(link.parent().unwrap().val().url().as_str(), "https://example.com/");
//!         link
//!     },
//!     _ => panic!("failed to get links from page")
//! };
//!
//! // add another filter looking for the root path and start scanning for links
//! let filters = first_link_on_example.filters().clone()
//!     .add_filter(PathFilter::new(FilterType::Equals, "/"));
//! let stream = first_link_on_example
//!     .with_filters(filters)
//!     .exec();
//! # futures::pin_mut!(stream); // needed for iteration
//! // get the first link from the stream and ensure its a link to the homepage
//! match stream.next().await {
//!     Some(Ok(link)) => {
//!         assert_eq!(link.val().url().as_str(), "https://www.iana.org/");
//!         assert_eq!(link.parent().unwrap().val().url().as_str(), "https://www.iana.org/domains/example");
//!         assert_eq!(link.parent().unwrap().parent().unwrap().val().url().as_str(), "https://example.com/")
//!     },
//!     _ => panic!("failed to get links from page")
//! }
//! // ensure theres no other unexpected links on the homepage
//! assert!(stream.next().await.is_none(), "should've been only one homepage link on the page!");
//! # Ok(())
//! # }
//! ```

#![recursion_limit = "256"]

#[macro_use]
extern crate failure_derive;

mod errors;
pub mod filters;

pub use errors::StorkHttpError;
pub use url::Url;

use stork::{StorkClient, Storkable};

use std::pin::Pin;

use select::document::Document;
use select::predicate::{And, Attr, Name, Not};

use async_stream::try_stream;

use failure::Error;
use failure::ResultExt;

use std::sync::Arc;

pub use reqwest::Client as ReqwestClient;

pub type HttpStorkable = Storkable<Link, HttpStorkClient>;

#[derive(Debug)]
pub struct Link {
    url: Url,
    text: Option<String>,
}
impl Link {
    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn text(&self) -> Option<String> {
        self.text.clone()
    }
}
impl std::str::FromStr for Link {
    type Err = failure::Error;

    fn from_str(input: &str) -> Result<Link, Error> {
        Ok(Self {
            url: Url::parse(input).context(StorkHttpError::UrlParseError)?,
            text: None,
        })
    }
}
impl From<Url> for Link {
    fn from(url: Url) -> Self {
        Self { url, text: None }
    }
}

pub struct HttpStorkClient {
    client: Arc<reqwest::Client>,
}

impl HttpStorkClient {
    pub fn new(client: ReqwestClient) -> Self {
        Self {
            client: Arc::new(client),
        }
    }
}

impl Default for HttpStorkClient {
    fn default() -> Self {
        Self {
            client: Arc::new(
                reqwest::Client::builder()
                    .user_agent(concat!(
                        env!("CARGO_PKG_NAME"),
                        "/",
                        env!("CARGO_PKG_VERSION")
                    ))
                    .build()
                    .unwrap(),
            ),
        }
    }
}

impl StorkClient<Link> for HttpStorkClient {
    fn run(&self, src: &Link) -> Pin<Box<dyn futures::Stream<Item = Result<Link, Error>>>> {
        let root = src.url.clone();
        let client = Arc::clone(&self.client);

        Box::pin(try_stream! {
            // TODO: can we get this to stream into the Document? need some
            // TODO: compat layer between futures and std::io::Read
            let doc = client.get(root.clone())
                .send().await.context(StorkHttpError::HttpError)?
                .bytes().await.context(StorkHttpError::HttpError)?;
            let document = Document::from_read(&doc[..]).context(StorkHttpError::HtmlParseError)?;

            for node in document.find(And(Name("a"), Not(Attr("rel", "nofollow")))) {
                let title = node.text().trim().to_string();
                let href = node.attr("href");

                if let Some(href) = href {
                    // if this looks like a relative url append it to the root
                    let href = if href.starts_with('/') || !href.contains("://") {
                        root.join(href).context(StorkHttpError::UrlParseError)?
                    } else {
                        Url::parse(href).context(StorkHttpError::UrlParseError)?
                    };

                    yield Link {
                        url: href,
                        text: Some(title).filter(|x| !x.is_empty())
                    };
                }
            }
        })
    }
}
