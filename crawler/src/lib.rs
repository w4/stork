//! `stork` is a simple library to recursively crawl websites for links
//! in a search engine-like fashion. stork was designed from the ground
//! to have a simple API that is easy to use.
//!
//! Your entry point into stork is the [Storkable::new] function. Have
//! a look through the [Storkable] struct's documentation for your
//! entry into the world of storking.
#![recursion_limit = "512"]

#[macro_use]
extern crate failure_derive;

pub mod errors;
pub mod filters;

pub use errors::StorkError;
pub use filters::FilterSet;

pub use url::Url;

use select::document::Document;
use select::predicate::{And, Attr, Name, Not};

use async_stream::try_stream;
use futures::pin_mut;
use futures::prelude::*;
use std::sync::Arc;

use failure::Error;
use failure::ResultExt;

/// A [Storkable] represents a "thing" (currently just a website link)
/// which is traversable.
///
/// To start "storking" a website an initial [Storkable] can be
/// constructed with [Storkable::new], once initialised filters can be
/// added using [Storkable::with_filters].
///
/// After a [Storkable] has been initialised, the storking can begin
/// with a call to [Storkable::exec] which will return a
/// stream of more [Storkable]s (with the filters from the parent
/// [Storkable] copied) which in turn can also be storked if necessary.
///
/// Example usage:
///
/// ```
/// # use failure::err_msg;
/// # use stork::{Storkable, FilterSet, filters::{UrlFilter, UrlFilterType}};
/// # use futures::StreamExt;
/// #
/// # #[tokio::main]
/// # async fn main() -> failure::Fallible<()> {
/// let stream = Storkable::new("https://example.com/".parse()?)
///     .with_filters(
///         FilterSet::default()
///             .add_url_filter(UrlFilter::new(UrlFilterType::Domain, String::from("www.iana.org")))
///             .add_url_filter(UrlFilter::new(UrlFilterType::Scheme, String::from("https")))
///     )
///     .exec();
/// # futures::pin_mut!(stream); // needed for iteration
/// let first_link: Storkable = stream.next().await.ok_or(err_msg("no links on page"))??;
/// assert_eq!(first_link.url().as_str(), "https://www.iana.org/domains/example");
/// assert_eq!(first_link.parent().unwrap().url().as_str(), "https://example.com/");
///
/// let stream = first_link.exec();
/// # futures::pin_mut!(stream); // needed for iteration
/// let inner_link = stream.next().await.ok_or(err_msg("no links on page"))??;
/// assert_eq!(inner_link.url().as_str(), "https://www.iana.org/");
/// assert_eq!(inner_link.parent().unwrap().url().as_str(), "https://www.iana.org/domains/example");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Storkable {
    url: Url,
    filters: Arc<FilterSet>,
    client: Arc<reqwest::Client>,
    parent: Option<Arc<Storkable>>,
}
impl Storkable {
    /// Instantiates a new [Storkable] from a [Url], storking can then
    /// begin on the given [Url] using the [Storkable::exec] method.
    pub fn new(url: Url) -> Self {
        Self {
            url,
            filters: Arc::new(FilterSet::default()),
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
            parent: None,
        }
    }

    /// Attaches a [FilterSet] to this, and child, [Storkable]s.
    pub fn with_filters(mut self, filters: FilterSet) -> Self {
        self.filters = Arc::new(filters);
        self
    }

    /// Set a custom [reqwest::Client] to use with this, and child,
    /// [Storkable]s.
    pub fn with_client(mut self, client: reqwest::Client) -> Self {
        self.client = Arc::new(client);
        self
    }

    /// Get the URL of this [Storkable].
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get the [Storkable] from which this [Storkable] was found on.
    pub fn parent(&self) -> Option<&Storkable> {
        // map to Arc::as_ref to hide the underlying Arc implementation
        self.parent.as_ref().map(Arc::as_ref)
    }

    /// Start storking this [Storkable].
    ///
    /// Finds all the followable links on this [Storkable] and returns
    /// a stream of more [Storkable]s with the same filters and the
    /// `parent` set to a reference of the current [Storkable].
    pub fn exec<'a>(self) -> impl futures::Stream<Item = Result<Storkable, Error>> + 'a {
        let this = Arc::new(self);

        try_stream! {
            let links = get_all_links_from_page(&this);
            pin_mut!(links); // needed for iteration

            while let Some(link) = links.next().await {
                let link = link?;

                if !this.filters.matches_url(&link.url) {
                    continue;
                }

                yield Storkable {
                    url: link.url,
                    client: Arc::clone(&this.client),
                    filters: Arc::clone(&this.filters),
                    parent: Some(Arc::clone(&this)),
                };
            }
        }
    }
}

struct PageLink {
    pub name: String,
    pub url: Url,
}

/// Sends a request to the [Storkable::url] and grabs all followable
/// links from it.
fn get_all_links_from_page<'a>(
    storkable: &'a Storkable,
) -> impl futures::Stream<Item = Result<PageLink, Error>> + 'a {
    try_stream! {
        let root = storkable.url.clone();

        // TODO: can we get this to stream into the Document? need some
        // TODO: compat layer between futures and std::io::Read
        let doc = storkable.client.get(root.clone())
            .send().await.context(StorkError::HttpError)?
            .bytes().await.context(StorkError::HttpError)?;
        let document = Document::from_read(&doc[..]).context(StorkError::HtmlParseError)?;

        for node in document.find(And(Name("a"), Not(Attr("rel", "nofollow")))) {
            let title = node.text().trim().to_string();
            let href = node.attr("href");

            if let Some(href) = href {
                // if this looks like a relative url append it to the root
                let href = if href.starts_with('/') || !href.contains("://") {
                    root.join(href).context(StorkError::UrlParseError)?
                } else {
                    Url::parse(href).context(StorkError::UrlParseError)?
                };

                yield PageLink {
                    name: title,
                    url: href,
                };
            }
        }
    }
}
