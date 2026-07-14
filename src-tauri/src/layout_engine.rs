use shared_types::{Layout, Rect, ScreenInfo};

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn compute_zones(layout: &Layout, screen: &ScreenInfo) -> Result<Vec<Rect>, String> {
        let inner_width = screen.work_area.width as i32 - (layout.margin_px as i32 * 2);
        let inner_height = screen.work_area.height as i32 - (layout.margin_px as i32 * 2);

        if inner_width <= 0 || inner_height <= 0 {
            return Err("Layout too small for screen".into());
        }

        let zones = layout.zones as usize;
        let _num_cols = layout.columns.split_whitespace().count();

        if zones == 2 && layout.ratio.is_some() {
            let ratio = layout.ratio.unwrap() as i32; // 10-90
            let available = inner_width - layout.gap_px as i32;
            let w0 = available * ratio / 100;
            let w1 = available - w0;
            let x0 = screen.work_area.x + layout.margin_px as i32;
            let x1 = x0 + w0 + layout.gap_px as i32;
            let y0 = screen.work_area.y + layout.margin_px as i32;
            Ok(vec![
                Rect { x: x0, y: y0, width: w0 as u32, height: inner_height as u32 },
                Rect { x: x1, y: y0, width: w1 as u32, height: inner_height as u32 },
            ])
        } else if zones == 3 && layout.rows.is_some() {
            // Focus + Stack: zone 0 = left span-2, zone 1 = top-right, zone 2 = bottom-right
            let available = inner_width - layout.gap_px as i32;
            let w_left = available * 2 / 3;
            let w_right = available - w_left;
            let available_h = inner_height - layout.gap_px as i32;
            let h_top = available_h / 2;
            let h_bottom = available_h - h_top;
            let x0 = screen.work_area.x + layout.margin_px as i32;
            let x1 = x0 + w_left + layout.gap_px as i32;
            let y0 = screen.work_area.y + layout.margin_px as i32;
            Ok(vec![
                Rect { x: x0, y: y0, width: w_left as u32, height: inner_height as u32 },
                Rect { x: x1, y: y0, width: w_right as u32, height: h_top as u32 },
                Rect { x: x1, y: y0 + h_top + layout.gap_px as i32, width: w_right as u32, height: h_bottom as u32 },
            ])
        } else {
            // Equal-width columns — use zones as authoritative count
            let parts = zones;
            let gaps = (parts - 1) as i32 * layout.gap_px as i32;
            let available = inner_width - gaps;
            let zone_width = available / parts as i32;
            let remainder = available % parts as i32;
            let mut rects = Vec::new();
            let mut x = screen.work_area.x + layout.margin_px as i32;
            let y = screen.work_area.y + layout.margin_px as i32;
            for i in 0..zones {
                let w = if i < remainder as usize { zone_width + 1 } else { zone_width };
                rects.push(Rect { x, y, width: w as u32, height: inner_height as u32 });
                x += w + layout.gap_px as i32;
            }
            Ok(rects)
        }
    }

    pub fn validate_layout(layout: &Layout) -> Result<(), String> {
        if layout.name.is_empty() || layout.name.len() > 64 {
            return Err("Layout name must be 1-64 characters".into());
        }
        if layout.zones < 2 || layout.zones > 3 {
            return Err("Layout must have 2 or 3 zones".into());
        }
        if let Some(ratio) = layout.ratio {
            if ratio < 10 || ratio > 90 {
                return Err("Ratio must be between 10 and 90".into());
            }
        }
        if layout.gap_px > 40 {
            return Err("Gap must be <= 40px".into());
        }
        if layout.margin_px > 60 {
            return Err("Margin must be <= 60px".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_screen() -> ScreenInfo {
        ScreenInfo {
            id: "DP-1".into(),
            label: "DP-1".into(),
            resolution: "2560 x 1440".into(),
            work_area: Rect { x: 0, y: 0, width: 2560, height: 1440 },
        }
    }

    #[test]
    fn test_two_columns_equal() {
        let layout = Layout {
            id: "2col".into(),
            name: "Two Columns".into(),
            layout_type: LayoutType::Preset,
            zones: 2, columns: "1fr 1fr".into(),
            rows: None, span_first: None,
            ratio: Some(50), gap_px: 10, margin_px: 16,
            created_at: "".into(), updated_at: "".into(),
        };
        let zones = LayoutEngine::compute_zones(&layout, &test_screen()).unwrap();
        assert_eq!(zones.len(), 2);
        // outer margin 16, so inner width = 2560 - 32 = 2528
        // gap 10, each zone gets (2528 - 10) / 2 = 1259
        assert_eq!(zones[0].width, 1259);
        assert_eq!(zones[1].width, 1259);
    }

    #[test]
    fn test_ratio_splits_unevenly() {
        let layout = Layout {
            id: "main-side".into(),
            name: "Main + Sidebar".into(),
            layout_type: LayoutType::Preset,
            zones: 2, columns: "3fr 1fr".into(),
            rows: None, span_first: None,
            ratio: Some(75), gap_px: 10, margin_px: 0,
            created_at: "".into(), updated_at: "".into(),
        };
        let zones = LayoutEngine::compute_zones(&layout, &test_screen()).unwrap();
        assert_eq!(zones.len(), 2);
        // full width 2560, gap 10, available = 2550
        // w0 = 2550 * 75 / 100 = 1912 (integer truncation), w1 = 2550 - 1912 = 638
        assert_eq!(zones[0].width, 1912);
        assert_eq!(zones[1].width, 638);
    }
}
