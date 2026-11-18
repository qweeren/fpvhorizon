// FPV Drone HUD gimbal overlays (Betaflight style stick displays).
// Port of DroneHud.java.

pub const BOX_SIZE: i32 = 56;
pub const MARGIN_X: i32 = 14;
pub const MARGIN_Y: i32 = 14;
pub const MARKER_HALF: i32 = 3;
pub const COLOR_OVERLAY: u32 = 0x99FFFFFF; // ARGB semi-transparent white (-1711276033)

pub trait HudPainter {
    /// Fills an axis-aligned rectangle [min_x, min_y, max_x, max_y] with an ARGB color.
    fn fill_rect(&mut self, min_x: i32, min_y: i32, max_x: i32, max_y: i32, argb: u32);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GimbalHud {
    pub yaw: f32,
    pub throttle: f32,
    pub roll: f32,
    pub pitch: f32,
}

impl GimbalHud {
    pub fn new(yaw: f32, throttle: f32, roll: f32, pitch: f32) -> Self {
        Self {
            yaw: yaw.clamp(-1.0, 1.0),
            throttle: throttle.clamp(-1.0, 1.0),
            roll: roll.clamp(-1.0, 1.0),
            pitch: pitch.clamp(-1.0, 1.0),
        }
    }

    /// Renders both stick gimbals:
    /// - Left gimbal: Yaw (X) & Throttle (Y)
    /// - Right gimbal: Roll (X) & Pitch (Y)
    pub fn render<P: HudPainter>(&self, painter: &mut P, screen_width: i32, screen_height: i32) {
        let top = screen_height - MARGIN_Y - BOX_SIZE;
        // Left gimbal (Throttle / Yaw)
        Self::draw_gimbal(painter, MARGIN_X, top, self.yaw, self.throttle);
        // Right gimbal (Roll / Pitch)
        Self::draw_gimbal(painter, screen_width - MARGIN_X - BOX_SIZE, top, self.roll, self.pitch);
    }

    fn draw_gimbal<P: HudPainter>(painter: &mut P, left: i32, top: i32, x: f32, y: f32) {
        let right = left + BOX_SIZE;
        let bottom = top + BOX_SIZE;
        let cx = left + BOX_SIZE / 2;
        let cy = top + BOX_SIZE / 2;

        // Crosshairs
        painter.fill_rect(left, cy, right, cy + 1, COLOR_OVERLAY);
        painter.fill_rect(cx, top, cx + 1, bottom, COLOR_OVERLAY);

        // Marker box
        let travel = 27.0 - MARKER_HALF as f32;
        let mx = cx + (x.clamp(-1.0, 1.0) * travel).round() as i32;
        let my = cy - (y.clamp(-1.0, 1.0) * travel).round() as i32;

        let l = mx - MARKER_HALF;
        let r = mx + MARKER_HALF + 1;
        let t = my - MARKER_HALF;
        let bt = my + MARKER_HALF + 1;

        // Border of marker
        painter.fill_rect(l, t, r, t + 1, COLOR_OVERLAY);
        painter.fill_rect(l, bt - 1, r, bt, COLOR_OVERLAY);
        painter.fill_rect(l, t + 1, l + 1, bt - 1, COLOR_OVERLAY);
        painter.fill_rect(r - 1, t + 1, r, bt - 1, COLOR_OVERLAY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPainter {
        rects: Vec<(i32, i32, i32, i32, u32)>,
    }

    impl HudPainter for MockPainter {
        fn fill_rect(&mut self, min_x: i32, min_y: i32, max_x: i32, max_y: i32, argb: u32) {
            self.rects.push((min_x, min_y, max_x, max_y, argb));
        }
    }

    #[test]
    fn test_hud_rendering_produces_rectangles() {
        let mut painter = MockPainter { rects: Vec::new() };
        let hud = GimbalHud::new(0.0, 0.5, 0.2, -0.3);
        hud.render(&mut painter, 854, 480);
        assert!(!painter.rects.is_empty());
        // Each gimbal produces 2 crosshair rects + 4 marker border rects = 6 rects * 2 = 12 rects
        assert_eq!(painter.rects.len(), 12);
    }
}
