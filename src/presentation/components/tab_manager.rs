use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Tab {
    Installed,
    Outdated,
    Browse,
    Maintenance,
}

pub struct TabState {
    pub loaded: bool,
}

impl TabState {
    pub fn new() -> Self {
        Self { loaded: false }
    }
}

pub struct TabManager {
    current_tab: Tab,
    tab_states: HashMap<Tab, TabState>,
}

impl TabManager {
    pub fn new() -> Self {
        let mut tab_states = HashMap::new();
        tab_states.insert(Tab::Installed, TabState::new());
        tab_states.insert(Tab::Outdated, TabState::new());
        tab_states.insert(Tab::Browse, TabState::new());
        tab_states.insert(Tab::Maintenance, TabState::new());

        Self {
            current_tab: Tab::Installed,
            tab_states,
        }
    }

    pub fn switch_to(&mut self, tab: Tab) {
        self.current_tab = tab;
    }

    pub fn current(&self) -> Tab {
        self.current_tab
    }

    pub fn is_current(&self, tab: Tab) -> bool {
        self.current_tab == tab
    }

    pub fn is_loaded(&self, tab: Tab) -> bool {
        self.tab_states
            .get(&tab)
            .map(|state| state.loaded)
            .unwrap_or(false)
    }

    pub fn mark_loaded(&mut self, tab: Tab) {
        if let Some(state) = self.tab_states.get_mut(&tab) {
            state.loaded = true;
        }
    }

    pub fn mark_unloaded(&mut self, tab: Tab) {
        if let Some(state) = self.tab_states.get_mut(&tab) {
            state.loaded = false;
        }
    }
}
