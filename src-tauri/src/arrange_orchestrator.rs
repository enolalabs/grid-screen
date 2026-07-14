use shared_types::*;
use crate::platform_adapter::PlatformAdapter;
use crate::layout_engine::LayoutEngine;

pub struct ArrangeOrchestrator;

impl ArrangeOrchestrator {
    pub fn arrange(
        request: &ArrangeRequest,
        layouts: &[Layout],
        screens: &[ScreenInfo],
        adapter: &dyn PlatformAdapter,
        engine: &LayoutEngine,
    ) -> ArrangeResult {
        let layout = match layouts.iter().find(|l| l.id == request.layout_id) {
            Some(l) => l,
            None => return ArrangeResult {
                success: false,
                results: vec![PerWindowResult {
                    window_id: "".into(),
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some("Layout not found".into()),
                }],
            },
        };

        let screen = match screens.iter().find(|s| s.id == request.screen_id) {
            Some(s) => s,
            None => return ArrangeResult {
                success: false,
                results: vec![PerWindowResult {
                    window_id: "".into(),
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some("Screen not found".into()),
                }],
            },
        };

        let zones = match LayoutEngine::compute_zones(layout, screen) {
            Ok(z) => z,
            Err(e) => return ArrangeResult {
                success: false,
                results: vec![PerWindowResult {
                    window_id: "".into(),
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some(e),
                }],
            },
        };

        // Validate all assignments first
        let mut errors = Vec::new();
        for (zone_idx, window_id) in &request.assignments {
            if *zone_idx as usize >= zones.len() {
                errors.push((window_id.clone(), format!("Zone {} out of range", zone_idx)));
                continue;
            }
            if adapter.get_window_state(window_id).is_none() {
                errors.push((window_id.clone(), "Window no longer exists".into()));
                continue;
            }
            let state = adapter.get_window_state(window_id).unwrap();
            if !state.movable || !state.resizable {
                errors.push((window_id.clone(), "Window is not movable or resizable".into()));
            }
        }

        if !errors.is_empty() {
            return ArrangeResult {
                success: false,
                results: errors.into_iter().map(|(wid, err)| PerWindowResult {
                    window_id: wid,
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some(err),
                }).collect(),
            };
        }

        // Execute arrangement
        let mut results = Vec::new();
        for (zone_idx, window_id) in &request.assignments {
            let zone = &zones[*zone_idx as usize];

            let state = adapter.get_window_state(window_id).unwrap();
            if state.minimized {
                adapter.restore_window(window_id);
            }

            let frame = adapter.get_frame_extents(window_id);
            let adjusted = Rect {
                x: zone.x - frame.x,
                y: zone.y - frame.y,
                width: (zone.width as i32 - frame.width as i32 - frame.x).max(0) as u32,
                height: (zone.height as i32 - frame.height as i32 - frame.y).max(0) as u32,
            };

            match adapter.move_resize_window(window_id, adjusted) {
                Ok(actual) => results.push(PerWindowResult {
                    window_id: window_id.clone(),
                    status: MoveStatus::Moved,
                    actual_rect: Some(actual),
                    error: None,
                }),
                Err(e) => results.push(PerWindowResult {
                    window_id: window_id.clone(),
                    status: MoveStatus::Failed,
                    actual_rect: None,
                    error: Some(e),
                }),
            }
        }

        let all_moved = results.iter().all(|r| matches!(r.status, MoveStatus::Moved));
        ArrangeResult { success: all_moved, results }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform_adapter::MockPlatformAdapter;
    use crate::layout_engine::LayoutEngine;

    #[test]
    fn test_arrange_success() {
        let adapter = MockPlatformAdapter::new();
        let mut adapter = adapter;
        adapter.windows = vec![
            WindowDescriptor {
                id: "w1".into(), app_name: "Firefox".into(), title: "MDN".into(),
                icon_color: "#ff7139".into(),
                state: WindowState { minimized: false, maximized: false, fullscreen: false, movable: true, resizable: true },
            },
        ];
        let engine = LayoutEngine;
        let layouts = vec![Layout {
            id: "2col".into(), name: "Two Columns".into(),
            layout_type: LayoutType::Preset,
            zones: 2, columns: "1fr 1fr".into(),
            rows: None, span_first: None,
            ratio: Some(50), gap_px: 10, margin_px: 0,
            created_at: "".into(), updated_at: "".into(),
        }];
        let screens = adapter.enumerate_screens();

        let request = ArrangeRequest {
            layout_id: "2col".into(),
            screen_id: "DP-1".into(),
            assignments: std::collections::HashMap::from([(0, "w1".into())]),
        };

        let result = ArrangeOrchestrator::arrange(&request, &layouts, &screens, &adapter, &engine);
        assert!(result.success);
        assert_eq!(result.results.len(), 1);
        assert_eq!(result.results[0].window_id, "w1");
        assert!(matches!(result.results[0].status, MoveStatus::Moved));
        let log = adapter.move_log.lock().unwrap();
        assert_eq!(log.len(), 1);
    }

    #[test]
    fn test_arrange_rejects_stale_window() {
        let adapter = MockPlatformAdapter::new();
        let engine = LayoutEngine;
        let layouts = vec![Layout {
            id: "2col".into(), name: "Two Columns".into(),
            layout_type: LayoutType::Preset,
            zones: 2, columns: "1fr 1fr".into(),
            rows: None, span_first: None,
            ratio: Some(50), gap_px: 0, margin_px: 0,
            created_at: "".into(), updated_at: "".into(),
        }];
        let screens = adapter.enumerate_screens();
        let request = ArrangeRequest {
            layout_id: "2col".into(),
            screen_id: "DP-1".into(),
            assignments: std::collections::HashMap::from([(0, "nonexistent".into())]),
        };
        let result = ArrangeOrchestrator::arrange(&request, &layouts, &screens, &adapter, &engine);
        assert!(!result.success);
        assert_eq!(result.results[0].status, MoveStatus::Failed);
    }
}
