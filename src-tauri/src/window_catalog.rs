use shared_types::WindowDescriptor;
#[cfg(test)]
use std::collections::HashSet;

pub fn is_eligible_window(w: &WindowDescriptor) -> bool {
    w.state.movable && w.state.resizable && !w.state.fullscreen && !w.app_name.is_empty()
}

#[cfg(test)]
pub struct WindowCatalog<'a> {
    adapter: &'a dyn PlatformAdapter,
    known_ids: std::cell::RefCell<HashSet<String>>,
}

#[cfg(test)]
impl<'a> WindowCatalog<'a> {
    pub fn new(adapter: &'a dyn PlatformAdapter) -> Self {
        WindowCatalog {
            adapter,
            known_ids: std::cell::RefCell::new(HashSet::new()),
        }
    }

    pub fn refresh(&self, workspace: &str) -> Vec<WindowDescriptor> {
        let windows = self.adapter.enumerate_windows(workspace);
        let eligible: Vec<_> = windows
            .into_iter()
            .filter(|w| is_eligible_window(w))
            .collect();
        self.known_ids.borrow_mut().extend(eligible.iter().map(|w| w.id.clone()));
        eligible
    }

    pub fn validate(&self) -> Vec<String> {
        let known = self.known_ids.borrow();
        let mut stale = Vec::new();
        for id in known.iter() {
            if self.adapter.get_window_state(id).is_none() {
                stale.push(id.clone());
            }
        }
        stale
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_adapter::MockPlatformAdapter;
    use shared_types::*;

    #[test]
    fn test_excludes_non_movable_windows() {
        let adapter = MockPlatformAdapter::new();
        let mut adapter = adapter;
        adapter.windows = vec![
            WindowDescriptor {
                id: "w1".into(), app_name: "Firefox".into(), title: "MDN".into(),
                icon_color: "#ff7139".into(),
                state: WindowState { minimized: false, maximized: false, fullscreen: false, movable: false, resizable: true },
            },
            WindowDescriptor {
                id: "w2".into(), app_name: "Terminal".into(), title: "bash".into(),
                icon_color: "#2d2d2d".into(),
                state: WindowState { minimized: false, maximized: false, fullscreen: false, movable: true, resizable: true },
            },
        ];
        let catalog = WindowCatalog::new(&adapter);
        let windows = catalog.refresh("1");
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].id, "w2");
    }
}
