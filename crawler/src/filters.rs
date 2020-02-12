use url::Url;

#[derive(Clone)]
pub struct Filters {
    url: Option<Vec<UrlFilter>>,
}
impl Filters {
    pub fn add_url_filter(mut self, filter: UrlFilter) -> Self {
        if self.url.is_none() {
            self.url = Some(Vec::new());
        }

        // unwrap can't panic here because we filled the value above
        self.url.as_mut().unwrap().push(filter);

        self
    }

    pub(crate) fn matches_url(&self, link: &crate::PageLink) -> bool {
        if let Some(filters) = &self.url {
            for filter in filters.iter() {
                if !filter.matches(&link.url) {
                    return false;
                }
            }
        }

        true
    }
}
impl Default for Filters {
    fn default() -> Self {
        Filters {
            url: None,
        }
    }
}

#[derive(Clone)]
pub enum FilterType {
    StartsWith, EndsWith, Contains
}

#[derive(Clone)]
pub enum UrlFilterType {
    Path(FilterType), Domain
}

#[derive(Clone)]
pub struct UrlFilter {
    kind: UrlFilterType,
    value: String,
    negated: bool,
}
impl UrlFilter {
    pub fn new(kind: UrlFilterType, value: String) -> Self {
        Self {
            kind,
            value,
            negated: false,
        }
    }

    pub fn negated(mut self) -> Self {
        self.negated = true;
        self
    }

    pub fn matches(&self, url: &Url) -> bool {
        let matches = match &self.kind {
            UrlFilterType::Path(FilterType::StartsWith) => url.path().starts_with(&self.value),
            UrlFilterType::Path(FilterType::EndsWith) => url.path().ends_with(&self.value),
            UrlFilterType::Path(FilterType::Contains) => url.path().contains(&self.value),
            UrlFilterType::Domain => url.host_str().map_or(false, |v| v == &self.value)
        };

        match self.negated {
            true => !matches,
            false => matches
        }
    }
}