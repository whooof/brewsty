#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum SortField {
    Name,
    Version,
    Type,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

pub struct FilterState {
    show_formulae: bool,
    show_casks: bool,
    search_query: String,
    installed_search_query: String,
    sort_field: SortField,
    sort_order: SortOrder,
}

impl FilterState {
    pub fn new() -> Self {
        Self {
            show_formulae: true,
            show_casks: true,
            search_query: String::new(),
            installed_search_query: String::new(),
            sort_field: SortField::Name,
            sort_order: SortOrder::Ascending,
        }
    }

    pub fn show_formulae(&self) -> bool {
        self.show_formulae
    }

    pub fn set_show_formulae(&mut self, value: bool) {
        self.show_formulae = value;
    }

    pub fn show_casks(&self) -> bool {
        self.show_casks
    }

    pub fn set_show_casks(&mut self, value: bool) {
        self.show_casks = value;
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn search_query_mut(&mut self) -> &mut String {
        &mut self.search_query
    }

    pub fn installed_search_query(&self) -> &str {
        &self.installed_search_query
    }

    pub fn installed_search_query_mut(&mut self) -> &mut String {
        &mut self.installed_search_query
    }

    pub fn sort_field(&self) -> SortField {
        self.sort_field
    }

    pub fn sort_order(&self) -> SortOrder {
        self.sort_order
    }

    pub fn toggle_sort(&mut self, field: SortField) {
        if self.sort_field == field {
            self.sort_order = match self.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            self.sort_field = field;
            self.sort_order = SortOrder::Ascending;
        }
    }
}

impl Default for FilterState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state() {
        let fs = FilterState::new();
        assert!(fs.show_formulae());
        assert!(fs.show_casks());
        assert!(fs.search_query().is_empty());
        assert!(fs.installed_search_query().is_empty());
        assert_eq!(fs.sort_field(), SortField::Name);
        assert_eq!(fs.sort_order(), SortOrder::Ascending);
    }

    #[test]
    fn toggle_sort_same_field_reverses_order() {
        let mut fs = FilterState::new();
        assert_eq!(fs.sort_order(), SortOrder::Ascending);
        fs.toggle_sort(SortField::Name);
        assert_eq!(fs.sort_field(), SortField::Name);
        assert_eq!(fs.sort_order(), SortOrder::Descending);
        fs.toggle_sort(SortField::Name);
        assert_eq!(fs.sort_order(), SortOrder::Ascending);
    }

    #[test]
    fn toggle_sort_different_field_resets_to_ascending() {
        let mut fs = FilterState::new();
        fs.toggle_sort(SortField::Name); // Descending
        assert_eq!(fs.sort_order(), SortOrder::Descending);
        fs.toggle_sort(SortField::Type); // Switch field → Ascending
        assert_eq!(fs.sort_field(), SortField::Type);
        assert_eq!(fs.sort_order(), SortOrder::Ascending);
    }

    #[test]
    fn set_filters() {
        let mut fs = FilterState::new();
        fs.set_show_formulae(false);
        assert!(!fs.show_formulae());
        fs.set_show_casks(false);
        assert!(!fs.show_casks());
    }
}
