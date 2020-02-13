pub use stork::filters::FilterType;

use std::borrow::Cow;

use stork::filters::Filter;

use crate::Link;

#[derive(Debug, Clone)]
pub enum UrlFilterType {
    Path(FilterType),
    Domain,
    Scheme,
}

#[derive(Debug, Clone)]
pub struct DomainFilter<'a>(Cow<'a, str>);
impl<'a> DomainFilter<'a> {
    pub fn new<V: Into<Cow<'a, str>>>(value: V) -> Self {
        Self(value.into())
    }
}
impl<'a> Filter<Link> for DomainFilter<'a> {
    fn matches(&self, link: &Link) -> bool {
        link.url()
            .host_str()
            .map_or(false, |v| v == self.0.as_ref())
    }
}

#[derive(Debug, Clone)]
pub struct SchemeFilter<'a>(Cow<'a, str>);
impl<'a> SchemeFilter<'a> {
    pub fn new<V: Into<Cow<'a, str>>>(value: V) -> Self {
        Self(value.into())
    }
}
impl<'a> Filter<Link> for SchemeFilter<'a> {
    fn matches(&self, link: &Link) -> bool {
        link.url().scheme() == self.0.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct PathFilter<'a> {
    value: Cow<'a, str>,
    kind: FilterType,
}
impl<'a> PathFilter<'a> {
    pub fn new<V: Into<Cow<'a, str>>>(kind: FilterType, value: V) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }
}
impl<'a> Filter<Link> for PathFilter<'a> {
    fn matches(&self, link: &Link) -> bool {
        let url = link.url();

        match &self.kind {
            FilterType::StartsWith => url.path().starts_with(self.value.as_ref()),
            FilterType::EndsWith => url.path().ends_with(self.value.as_ref()),
            FilterType::Contains => url.path().contains(self.value.as_ref()),
            FilterType::Equals => url.path() == self.value.as_ref(),
        }
    }
}
