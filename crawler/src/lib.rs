#![recursion_limit="512"]

#[macro_use] extern crate failure_derive;

pub mod filters;
pub mod errors;

pub use filters::Filters;
pub use errors::StorkError;

pub use url::Url;

use select::document::Document;
use select::predicate::{Attr, Name, And, Not};

use futures::prelude::*;
use futures::pin_mut;
use async_stream::try_stream;
use std::sync::Arc;

use failure::Error;
use failure::ResultExt;

/// A `Storkable` represents a website link which is traversable.
pub struct Storkable {
    url: Url,
    filters: Arc<Filters>,
    client: Arc<reqwest::Client>,
    parent: Option<Arc<Storkable>>,
}
impl Storkable {
    pub fn new(url: Url) -> Self {
        Self {
            url,
            filters: Arc::new(Filters::default()),
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

    pub fn with_filters(mut self, filters: Filters) -> Self {
        self.filters = Arc::new(filters);
        self
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn parent(&self) -> Option<&Storkable> {
        // map to Arc::as_ref to hide the underlying Arc implementation
        self.parent.as_ref().map(Arc::as_ref)
    }

    pub fn exec<'a>(self) -> impl futures::Stream<Item = Result<Storkable, Error>> + 'a {
        let this = Arc::new(self);

        try_stream! {
            let links = get_all_links_from_page(&this);
            pin_mut!(links); // needed for iteration

            while let Some(link) = links.next().await {
                let link = link?;

                if !this.filters.matches_url(&link) {
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

pub(crate) struct PageLink {
    pub name: String,
    pub url: Url
}
fn get_all_links_from_page<'a>(storkable: &'a Storkable) -> impl futures::Stream<Item = Result<PageLink, Error>> + 'a {
    try_stream! {
        let root = storkable.url.clone();

        // TODO: can we get this to stream into the Document? need some compat layer
        // TODO: between futures and std::io::Read
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