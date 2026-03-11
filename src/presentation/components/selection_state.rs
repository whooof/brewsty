use std::collections::HashSet;

#[derive(Clone)]
pub struct SelectionState {
    selected_packages: HashSet<String>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self {
            selected_packages: HashSet::new(),
        }
    }

    pub fn select(&mut self, package_name: String) {
        self.selected_packages.insert(package_name);
    }

    pub fn deselect(&mut self, package_name: &str) {
        self.selected_packages.remove(package_name);
    }

    pub fn is_selected(&self, package_name: &str) -> bool {
        self.selected_packages.contains(package_name)
    }

    pub fn get_selected(&self) -> Vec<String> {
        self.selected_packages.iter().cloned().collect()
    }

    pub fn has_selection(&self) -> bool {
        !self.selected_packages.is_empty()
    }

    pub fn clear(&mut self) {
        self.selected_packages.clear();
    }
}

impl Default for SelectionState {
    fn default() -> Self {
        Self::new()
    }
}
