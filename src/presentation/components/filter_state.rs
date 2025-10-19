pub struct FilterState {
    show_formulae: bool,
    show_casks: bool,
    search_query: String,
    installed_search_query: String,
}

impl FilterState {
    pub fn new() -> Self {
        Self {
            show_formulae: true,
            show_casks: true,
            search_query: String::new(),
            installed_search_query: String::new(),
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
}

impl Default for FilterState {
    fn default() -> Self {
        Self::new()
    }
}
