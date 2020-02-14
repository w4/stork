//! `stork` is a simple futures-based library to recursively crawl
//! sources in a search engine-like fashion. stork was designed from the
//! ground to have a simple API that is easy to use and can be reused
//! across multiple protocols, yielding each result giving end users the
//! freedom to do BFS, DFS or any type of search they may so wish.
//!
//! Your entry point into stork is the [Storkable::new] function. Have
//! a look through the [Storkable] struct's documentation for your
//! entry into the world of storking.
//!
//! *Note: you're probably not looking for this library on its own but
//! a protocol implementation of it. See below for some first-party
//! implementations:*
//! - [stork_http](../../../stork_http/)
#![recursion_limit = "256"]

#[macro_use]
extern crate failure_derive;

pub mod errors;
pub mod filters;

pub use errors::StorkError;
pub use filters::FilterSet;

use async_stream::try_stream;
use futures::prelude::*;

use std::pin::Pin;
use std::sync::Arc;

use failure::Error;
use failure::ResultExt;

/// A [Storkable] represents a "thing" which is traversable ("storkable").
///
/// To start "storking" an initial [Storkable] can be constructed with
/// with [Storkable::new], once initialised filters can be added using
/// [Storkable::with_filters].
///
/// After a [Storkable] has been initialised, the storking can begin
/// with a call to [Storkable::exec] which will return a
/// stream of more [Storkable]s (with the filters from the parent
/// [Storkable] copied) which in turn can also be storked if necessary.
///
/// A Storkable derives its functionality from its two generics,
/// `T` and `C: StorkClient<T>`. The `StorkClient` implementation will
/// be called with a value of `T`, and is expected to return all the
/// values of `T` that can be found on the given `T`.
#[derive(Debug, Clone)]
pub struct Storkable<T: Unpin, C: StorkClient<T>> {
    value: T,
    filters: FilterSet<T>,
    client: Arc<C>,
    parent: Option<Arc<Storkable<T, C>>>,
}

impl<'a, T: Unpin + PartialEq + 'a, C: StorkClient<T> + 'a> Storkable<T, C> {
    /// Instantiates a new [Storkable] from a T, storking can then
    /// begin on the given entrypoint using the [Storkable::exec] method.
    pub fn new(val: T) -> Self {
        Self {
            value: val,
            filters: FilterSet::default(),
            client: Arc::new(C::default()),
            parent: None,
        }
    }

    /// Attaches a [FilterSet] to this [Storkable] and any children
    /// found after executing this one.
    pub fn with_filters(mut self, filters: FilterSet<T>) -> Self {
        self.filters = filters;
        self
    }

    /// Replaces the default [StorkClient] with a new one accepting
    /// and returning the same type for this [Storkable].
    pub fn with_client(mut self, client: C) -> Self {
        self.client = Arc::new(client);
        self
    }

    // Grab a reference to the filters set on this [Storkable].
    pub fn filters(&self) -> &FilterSet<T> {
        &self.filters
    }

    /// Get the value of this [Storkable].
    pub fn val(&self) -> &T {
        &self.value
    }

    /// Get the [Storkable] from which this [Storkable] was found on.
    pub fn parent(&self) -> Option<&Storkable<T, C>> {
        // map to Arc::as_ref to hide the underlying Arc implementation
        self.parent.as_ref().map(Arc::as_ref)
    }

    /// Start storking this [Storkable].
    ///
    /// Finds all the followable links on this [Storkable] and returns
    /// a stream of more [Storkable]s with the same filters and the
    /// `parent` set to a reference of the current [Storkable].
    pub fn exec<'b>(self) -> impl futures::Stream<Item = Result<Storkable<T, C>, Error>> + 'a {
        let this = Arc::new(self);

        try_stream! {
            let mut children = this.client.run(this.val());

            'child: while let Some(child) = children.next().await {
                let child = child.context(StorkError::ClientError)?;

                if !this.filters.matches(&child) {
                    continue;
                }

                // ensure we're not going to cause a recursive loop by
                // checking that the page we're about to yield isn't a
                // parent of it
                // TODO: should this happen before or after we try to
                // TODO: match it against the filters
                let mut current_parent = this.parent();
                while let Some(parent) = current_parent {
                    if parent.value == this.value {
                        continue 'child;
                    }
                    current_parent = parent.parent();
                }

                yield Storkable {
                    value: child,
                    client: Arc::clone(&this.client),
                    filters: this.filters.clone(),
                    parent: Some(Arc::clone(&this)),
                };
            }
        }
    }
}

/// A [StorkClient] is an underlying implementation of a storker. When a
/// [Storkable] is initialised a [StorkClient] will be created using
/// [Default::default] and the instance will be shared between all child
/// [Storkable]s.
///
/// The default [StorkClient] initialised by the [Storkable] can be
/// replaced using [Storkable::with_client].
///
/// [StorkClient]s may be used across threads and *must* be thread-safe.
pub trait StorkClient<T>: Default {
    /// Makes a call to `T` and returns the child `T`s it can find on the
    /// page.
    fn run(&self, src: &T) -> Pin<Box<dyn futures::Stream<Item = Result<T, Error>>>>;
}
