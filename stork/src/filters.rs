/// List of filters that can be used to filter down results from a
/// [Storkable](crate::Storkable). Once constructed, these can be
/// attached using [Storkable::with_filters](crate::Storkable::with_filters).
#[derive(Debug)]
pub struct FilterSet<T> {
    filters: Option<Vec<Box<dyn Filter<T>>>>,
}
impl<T> FilterSet<T> {
    /// Filter results by a given predicate.
    pub fn add_filter<F: Filter<T> + 'static>(mut self, filter: F) -> Self {
        if self.filters.is_none() {
            self.filters = Some(Vec::new());
        }

        // unwrap can't panic here because we filled the value above
        self.filters.as_mut().unwrap().push(Box::new(filter));

        self
    }

    /// Check if this `Filters` matches the given `link`.
    pub(crate) fn matches(&self, val: &T) -> bool {
        if let Some(filters) = &self.filters {
            for filter in filters.iter() {
                if !filter.matches(&val) {
                    return false;
                }
            }
        }

        true
    }
}
impl<T> Default for FilterSet<T> {
    /// Creates an empty filter set.
    fn default() -> Self {
        FilterSet { filters: None }
    }
}
/// We need to manually implement [Clone] for this struct because
/// otherwise it won't be derived on values where T doesn't implement
/// Clone (which would be an unnecessary restriction on our API as T
/// is a type param on a method).
impl<T> Clone for FilterSet<T> {
    fn clone(&self) -> Self {
        Self {
            filters: self.filters.clone(),
        }
    }
}

/// Predicate for any values of <T> passing through a
/// [Storkable](crate::Storkable). See [html_filters] for example
/// implementations.
///
/// Note: *all* implementations of `Filter` should have an impl of
/// [Clone] so they can be passed to children and modified without
/// modifying FilterSets on parents.
///
/// [html_filters]: (../stork_html/filters)
pub trait Filter<T>: std::fmt::Debug + dyn_clone::DynClone {
    fn matches(&self, val: &T) -> bool;
}

/// we need to use dyn_clone's impl of cloning a boxed dynamically
/// dispatched trait because implementing it involves a bit of unsafe
/// code with recent changes to the compiler, so we'll trust them to
/// handle it.
impl<T> std::clone::Clone for Box<dyn Filter<T>> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(self.as_ref())
    }
}

#[derive(Debug, Clone)]
pub enum FilterType {
    StartsWith,
    EndsWith,
    Contains,
    Equals
}
