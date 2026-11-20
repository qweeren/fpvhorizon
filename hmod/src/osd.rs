//! The on-screen display, laid out as Betaflight's and DJI's goggles show it: crosshair and
//! artificial horizon in the middle, battery and timer in the corners, speed, altitude and the
//! way home at the bottom, warnings in the centre, and the drone's card while it waits to arm.

use crate::drone::{DroneState, GimbalHud, HudPainter, Preset};
use mod_api::DrawContext;

const WHITE: u32 = 0xFFFF_FFFF;
const DIM: u32 = 0xC0FF_FFFF;
const WARN: u32 = 0xFFFF_5050;
const AMBER: u32 = 0xFFFF_C040;
const PANEL: u32 = 0xA0000000;

/// What the OSD shows this frame.
pub struct Osd<'a> {
    pub state: &'a DroneState,
    pub preset: &'a Preset,
    pub custom: bool,
    /// Camera pitch and roll, degrees, for the horizon (FPV only).
    pub horizon: Option<(f32, f32)>,
    /// Link quality 0..100 and distance from the pilot, blocks.
    pub link: f32,
    pub distance: f64,
    /// Where the link is lost, blocks from the pilot.
    pub range: f64,
    /// Direction home relative to the drone's heading, degrees (0 straight ahead).
    pub home_bearing: f32,
    pub view: &'static str,
    /// Choosing a drone: the selection screen instead of the flight display.
    pub showcase: bool,
    /// Which preset of how many, for the selection screen.
    pub index: (usize, usize),
    /// The vertical field of view the world is drawn with, degrees.
    pub fov: f32,
    pub show_sticks: bool,
    /// The pad's glyphs while the pad is what the player uses; None for the keyboard.
    pub pad: Option<PadFamily>,
    /// Seconds, for blinking.
    pub time: f32,
}

fn text(d: &mut dyn DrawContext, s: &str, x: f32, y: f32, color: u32) {
    d.text(s, [x, y], color, true);
}

fn text_right(d: &mut dyn DrawContext, s: &str, right: f32, y: f32, color: u32) {
    let w = d.text_width(s);
    text(d, s, right - w, y, color);
}

fn text_centre(d: &mut dyn DrawContext, s: &str, cx: f32, y: f32, color: u32) {
    let w = d.text_width(s);
    text(d, s, cx - w / 2.0, y, color);
}

/// The pad buttons the hints name, by their place on the pad.
#[derive(Clone, Copy)]
pub enum PadButton {
    North,
    West,
    L1,
    R1,
    Up,
    Left,
    Right,
}

impl PadButton {
    /// The family whose art shows this part, and the button on the pad that plays it. A Nintendo
    /// pad going by its letters is read with North and West swapped (Horizon's
    /// `Button::by_letter`), so the hint shows the button that really does the job.
    fn on_pad(self, family: PadFamily) -> (PadFamily, PadButton) {
        match (family, self) {
            (PadFamily::NintendoLetters, PadButton::North) => (PadFamily::Nintendo, PadButton::West),
            (PadFamily::NintendoLetters, PadButton::West) => (PadFamily::Nintendo, PadButton::North),
            (PadFamily::NintendoLetters, button) => (PadFamily::Nintendo, button),
            other => other,
        }
    }

    /// Horizon's art for the button in `family` (`horizon:gui/controller/`, which a resource pack
    /// can replace), and its texture's width: 32 by 32, a shoulder button 48 by 32.
    fn sprite(self, family: PadFamily) -> (&'static str, f32) {
        use PadButton as B;
        use PadFamily as F;
        let (family, button) = self.on_pad(family);
        let name = match (family, button) {
            (F::Xbox, B::North) => "xb_y",
            (F::Xbox, B::West) => "xb_x",
            (F::Xbox, B::L1) => "xb_lb",
            (F::Xbox, B::R1) => "xb_rb",
            (F::Xbox, B::Up) => "xb_up",
            (F::Xbox, B::Left) => "xb_left",
            (F::Xbox, B::Right) => "xb_right",
            (F::PlayStation, B::North) => "psx_triangle",
            (F::PlayStation, B::West) => "psx_square",
            (F::PlayStation, B::L1) => "psx_l1",
            (F::PlayStation, B::R1) => "psx_r1",
            (F::PlayStation, B::Up) => "psx_up",
            (F::PlayStation, B::Left) => "psx_left",
            (F::PlayStation, B::Right) => "psx_right",
            (F::Nintendo, B::North) => "nx_x",
            (F::Nintendo, B::West) => "nx_y",
            (F::Nintendo, B::L1) => "nx_l",
            (F::Nintendo, B::R1) => "nx_r",
            (F::Nintendo, B::Up) => "nx_up",
            (F::Nintendo, B::Left) => "nx_left",
            (F::Nintendo, B::Right) => "nx_right",
            (F::NintendoLetters, _) => unreachable!("on_pad maps letters to Nintendo"),
        };
        let wide = matches!(button, B::L1 | B::R1);
        (name, if wide { 48.0 } else { 32.0 })
    }

    /// The button's print, where its art is missing.
    fn label(self, family: PadFamily) -> &'static str {
        use PadButton as B;
        match self.on_pad(family) {
            (PadFamily::Xbox, B::North) => "Y",
            (PadFamily::Xbox, B::West) => "X",
            (PadFamily::Xbox, B::L1) => "LB",
            (PadFamily::Xbox, B::R1) => "RB",
            (PadFamily::PlayStation, B::North) => "Triangle",
            (PadFamily::PlayStation, B::West) => "Square",
            (PadFamily::PlayStation, B::L1) => "L1",
            (PadFamily::PlayStation, B::R1) => "R1",
            (PadFamily::Nintendo, B::North) => "X",
            (PadFamily::Nintendo, B::West) => "Y",
            (PadFamily::Nintendo, B::L1) => "L",
            (PadFamily::Nintendo, B::R1) => "R",
            (_, B::Up) => "Up",
            (_, B::Left) => "Left",
            (_, B::Right) => "Right",
            (PadFamily::NintendoLetters, _) => unreachable!("on_pad maps letters to Nintendo"),
        }
    }
}

/// Whose button art the pad's hints use: the player's choice of glyphs in Horizon.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PadFamily {
    Xbox,
    PlayStation,
    Nintendo,
    /// Nintendo art on a pad going by its letters (Horizon's "Nintendo letters" setting).
    NintendoLetters,
}

/// A piece of a hint line: words, or a pad button drawn as Horizon draws it.
#[derive(Clone, Copy)]
pub enum Hint {
    Text(&'static str),
    Pad(PadButton),
}

/// The art is drawn 16 GUI pixels tall (its texture at half size), with a clear edge of 2 all
/// round, so it shows 12 tall: a text line and 2 above and below it.
const GLYPH_DRAWN: f32 = 16.0;
const GLYPH_MARGIN: f32 = 2.0;

/// A button's width in a hint, as it shows (its clear edges left out).
fn glyph_width(button: PadButton, family: PadFamily) -> f32 {
    let (_, texture_w) = button.sprite(family);
    texture_w / 2.0 - 2.0 * GLYPH_MARGIN
}

fn hint_width(d: &dyn DrawContext, hint: &[Hint], family: PadFamily) -> f32 {
    hint.iter()
        .map(|h| match *h {
            Hint::Text(s) => d.text_width(s),
            Hint::Pad(b) => glyph_width(b, family) + 2.0,
        })
        .sum()
}

/// A hint line centred on `cx`, its buttons drawn with Horizon's art in `family`'s glyphs, or
/// by their print where a pack took the art away.
fn hint_centre(d: &mut dyn DrawContext, hint: &[Hint], family: PadFamily, cx: f32, y: f32, color: u32) {
    let mut x = cx - hint_width(d, hint, family) / 2.0;
    for h in hint {
        match *h {
            Hint::Text(s) => {
                text(d, s, x, y, color);
                x += d.text_width(s);
            }
            Hint::Pad(b) => {
                let (name, texture_w) = b.sprite(family);
                let w = glyph_width(b, family);
                let dst = [x + 1.0 - GLYPH_MARGIN, y + 4.0 - GLYPH_DRAWN / 2.0, texture_w / 2.0, GLYPH_DRAWN];
                let sprite = format!("horizon:gui/controller/{name}");
                if !d.sprite(&sprite, [0.0, 0.0, texture_w, 32.0], dst, None) {
                    let label = b.label(family);
                    let left = x + 1.0 + (w - d.text_width(label)) / 2.0;
                    text(d, label, left, y, color);
                }
                x += w + 2.0;
            }
        }
    }
}

/// The hint for what is in use: the keyboard's words, or the pad's with its buttons drawn.
fn input_hint(d: &mut dyn DrawContext, osd: &Osd<'_>, keys: &str, pad: &[Hint], cx: f32, y: f32, color: u32) {
    match osd.pad {
        Some(family) => hint_centre(d, pad, family, cx, y, color),
        None => text_centre(d, keys, cx, y, color),
    }
}

/// A vertical gradient from `top` to `bottom` colour over `rect`, in thin bands (the draw
/// context has no gradient fill of its own).
fn gradient(d: &mut dyn DrawContext, rect: [f32; 4], top: u32, bottom: u32) {
    let [x, y, w, h] = rect;
    if h <= 0.0 {
        return;
    }
    let bands = (h / 3.0).ceil().clamp(1.0, 48.0) as u32;
    let band_h = h / bands as f32;
    let lerp = |shift: u32, t: f32| {
        let (a, b) = ((top >> shift & 0xFF) as f32, (bottom >> shift & 0xFF) as f32);
        ((a + (b - a) * t).round() as u32) << shift
    };
    for i in 0..bands {
        let t = (i as f32 + 0.5) / bands as f32;
        let c = lerp(24, t) | lerp(16, t) | lerp(8, t) | lerp(0, t);
        d.fill([x, y + i as f32 * band_h, w, band_h], c);
    }
}

/// A line `width` thick from `a` to `b`.
fn line(d: &mut dyn DrawContext, a: [f32; 2], b: [f32; 2], width: f32, color: u32) {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt().max(1e-3);
    let (nx, ny) = (-dy / len * width / 2.0, dx / len * width / 2.0);
    d.quad([[a[0] + nx, a[1] + ny], [b[0] + nx, b[1] + ny], [b[0] - nx, b[1] - ny], [a[0] - nx, a[1] - ny]], color);
}

/// Words of `s` wrapped to lines no wider than `width`.
fn wrap(d: &dyn DrawContext, s: &str, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in s.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if d.text_width(&candidate) > width && !line.is_empty() {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Link quality below which the picture starts breaking up.
const STATIC_START: f32 = 75.0;
/// Link quality below which the LOW SIGNAL badge shows.
const LOW_SIGNAL: f32 = 40.0;

/// A well-mixed 32-bit hash, for noise that changes every frame but is stable within one.
fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

/// 0..1 from a hash.
fn unit(r: u32) -> f32 {
    (r & 0xFFFF) as f32 / 65535.0
}

fn grey(alpha: f32, level: u32) -> u32 {
    ((alpha.clamp(0.0, 1.0) * 255.0) as u32) << 24 | level << 16 | level << 8 | level
}

/// Analog video breaking up as the link weakens: darkening edges, a rolling bar, torn
/// scanlines and snow, all growing in a straight line with the lost link quality, then a grey
/// screen of static when the link is gone.
fn draw_interference(d: &mut dyn DrawContext, w: f32, h: f32, link: f32, time: f32) {
    let n = ((STATIC_START - link) / STATIC_START).clamp(0.0, 1.0);
    if n <= 0.0 {
        return;
    }
    let lost = link <= 0.0;
    // Noise refreshes at 30 Hz, as a VTX's frames do; faster just looks like flicker.
    let frame = (time * 30.0) as u32;
    let rnd = |i: u32, salt: u32| hash(frame.wrapping_mul(0x9E37_79B9) ^ i.wrapping_mul(0x85EB_CA6B) ^ salt);

    if lost {
        d.fill([0.0, 0.0, w, h], 0xF018_1A1C);
    } else {
        // Edges sink into the dark first.
        let edge = grey(n * 0.7, 0);
        gradient(d, [0.0, 0.0, w, h * 0.28], edge, 0);
        gradient(d, [0.0, h * 0.72, w, h * 0.28], 0, edge);
        // A soft bar rolling down the picture, like a sync fault.
        let bar_h = h * 0.16;
        let bar_y = (time * 0.35).fract() * (h + bar_h) - bar_h;
        let bar = grey(n * 0.35, 0x10);
        gradient(d, [0.0, bar_y, w, bar_h / 2.0], 0, bar);
        gradient(d, [0.0, bar_y + bar_h / 2.0, w, bar_h / 2.0], bar, 0);
    }

    // Torn scanlines: bright, thin, full-width streaks at random heights.
    let tears = if lost { 18 } else { (n * 7.0) as u32 };
    for i in 0..tears {
        let r = rnd(i, 0x51);
        let y = unit(r) * h;
        let thick = 1.0 + (r >> 20 & 3) as f32;
        let x = unit(r >> 8) * w * 0.4;
        let alpha = if lost { 0.25 } else { 0.08 + n * 0.22 };
        d.fill([x, y, w - x, thick], grey(alpha, 0xD0 + (r >> 24 & 0x2F)));
    }

    // Snow: short white and dark dashes scattered over the frame.
    let specks = if lost { 900 } else { (n * 420.0) as u32 };
    let speck_alpha = if lost { 0.75 } else { 0.15 + n * 0.55 };
    for i in 0..specks {
        let r = rnd(i, 0xA7);
        let x = unit(r) * w;
        let y = unit(r >> 16 | r << 16) * h;
        let len = 1.0 + (r >> 27) as f32 * 0.4 * (0.5 + n);
        let level = if r & 1 == 0 { 0xF0 } else { 0x20 };
        d.fill([x, y, len, 1.0], grey(speck_alpha * (0.5 + unit(r >> 5) * 0.5), level));
    }

    // Near the end of range whole frames drop out now and then.
    if !lost && n > 0.6 {
        let chance = (n - 0.6) / 0.4 * 0.25;
        if unit(rnd(0, 0xD1)) < chance {
            d.fill([0.0, 0.0, w, h], grey(0.35 + n * 0.3, 0x30));
        }
    }
}

/// Four signal bars, `x` the left edge and `y` their baseline; `level` 0..100.
fn signal_bars(d: &mut dyn DrawContext, x: f32, y: f32, level: f32, color: u32) {
    for i in 0..4 {
        let bh = 3.0 + i as f32 * 2.0;
        let lit = level > i as f32 * 25.0 + 1.0;
        let c = if lit { color } else { 0x50FF_FFFF };
        d.fill([x + i as f32 * 3.0, y - bh, 2.0, bh], c);
    }
}

/// The LOW SIGNAL badge: a dark pill with bars, the warning, and how far is left before the
/// link is lost. It breathes instead of blinking, so it stays readable while you turn back.
fn draw_low_signal(d: &mut dyn DrawContext, osd: &Osd<'_>, cx: f32, y: f32) {
    let critical = osd.link < 15.0;
    let rate = if critical { 7.0 } else { 3.5 };
    let pulse = 0.7 + 0.3 * (osd.time * rate).sin();
    let base = if critical { WARN } else { AMBER };
    let color = ((pulse * 255.0) as u32) << 24 | (base & 0x00FF_FFFF);

    let title = "LOW SIGNAL";
    let left = (osd.range - osd.distance).max(0.0);
    let hint = if critical { format!("TURN BACK  {left:.0}m left") } else { format!("{left:.0}m to range") };
    let tw = d.text_width(title);
    let hw = d.text_width(&hint);
    let inner = (tw + 18.0).max(hw);
    let pw = inner + 16.0;
    let ph = 27.0;
    let x = cx - pw / 2.0;

    d.fill([x, y, pw, ph], 0xA810_1214);
    d.fill([x, y, pw, 1.0], color);
    // How much link is left, as a thin bar along the bottom edge.
    d.fill([x, y + ph - 2.0, pw, 2.0], 0x40FF_FFFF);
    d.fill([x, y + ph - 2.0, pw * (osd.link / LOW_SIGNAL).clamp(0.0, 1.0), 2.0], base);

    let row_x = cx - (tw + 18.0) / 2.0;
    signal_bars(d, row_x, y + 12.0, osd.link, color);
    text(d, title, row_x + 18.0, y + 4.0, color);
    text_centre(d, &hint, cx, y + 14.0, DIM);
}

struct Painter<'a>(&'a mut dyn DrawContext);

impl HudPainter for Painter<'_> {
    fn fill_rect(&mut self, min_x: i32, min_y: i32, max_x: i32, max_y: i32, argb: u32) {
        self.0.fill([min_x as f32, min_y as f32, (max_x - min_x) as f32, (max_y - min_y) as f32], argb);
    }
}

/// `s` in the big pixel font, centred on `cx`, its top at `y`, each font pixel `px` wide.
fn big_text(d: &mut dyn DrawContext, s: &str, cx: f32, y: f32, px: f32, color: u32) {
    crate::font::draw_centred(d, s, cx, y, px, color);
}

/// The drone selection screen, over the turntable camera: big name and class, arrows to the
/// others, and the drone's description and real specs.
fn draw_showcase(d: &mut dyn DrawContext, osd: &Osd<'_>) {
    use crate::drone::BuildParam as B;
    let [w, h] = d.size();
    let cx = w / 2.0;
    let p = osd.preset;
    let b = &osd.state.build;

    gradient(d, [0.0, 0.0, w, h * 0.22], 0xB000_0000, 0x0000_0000);
    gradient(d, [0.0, h * 0.62, w, h * 0.38], 0x0000_0000, 0xC000_0000);

    let title = if osd.custom { format!("{} (custom)", p.name) } else { p.name.to_string() };
    big_text(d, &title, cx, h * 0.05, 4.0, WHITE);
    big_text(d, p.class, cx, h * 0.05 + 36.0, 2.0, AMBER);
    let (i, n) = osd.index;
    text_centre(d, &format!("{} / {}", i + 1, n), cx, h * 0.05 + 56.0, DIM);

    // Arrows to the neighbours, pulsing.
    let pulse = (0xC0 as f32 + (osd.time * 3.0).sin() * 63.0) as u32;
    let arrow = (pulse << 24) | 0x00FF_FFFF;
    big_text(d, "<", w * 0.18, h * 0.45, 6.0, arrow);
    big_text(d, ">", w * 0.82, h * 0.45, 6.0, arrow);
    input_hint(d, osd, "[", &[Hint::Pad(PadButton::Left)], w * 0.18, h * 0.45 + 50.0, DIM);
    input_hint(d, osd, "]", &[Hint::Pad(PadButton::Right)], w * 0.82, h * 0.45 + 50.0, DIM);

    // Description and real specs.
    let panel_w = (w * 0.55).clamp(240.0, 520.0);
    let lines = wrap(d, p.description, panel_w);
    let mut y = h - 30.0 - (lines.len() as f32 + 3.0) * 11.0;
    for l in &lines {
        text_centre(d, l, cx, y, DIM);
        y += 11.0;
    }
    y += 5.0;
    let specs = format!(
        "{:.0} g   {:.0}S {:.0} mAh   {:.0} KV   {:.1} in x{:.0} props   {:.0} mm   {:.0}:1 thrust",
        b.get(B::DroneMass) * 1000.0,
        b.get(B::BatteryCells),
        b.get(B::BatteryMah),
        b.get(B::MotorKv),
        b.get(B::PropRadius) * 2.0 / 0.0254,
        b.get(B::Blades),
        p.wheelbase * 1000.0,
        p.target_twr,
    );
    text_centre(d, &specs, cx, y, WHITE);
    y += 16.0;
    input_hint(
        d,
        osd,
        "Throttle down + R: arm and fly    H: back to you",
        &[
            Hint::Text("Throttle down + "),
            Hint::Pad(PadButton::North),
            Hint::Text(": arm and fly    "),
            Hint::Pad(PadButton::L1),
            Hint::Text("+"),
            Hint::Pad(PadButton::R1),
            Hint::Text(": back to you"),
        ],
        cx,
        y,
        0xFF9A_D0FF,
    );
}

pub fn draw(d: &mut dyn DrawContext, osd: &Osd<'_>) {
    if osd.showcase {
        draw_showcase(d, osd);
        return;
    }
    let [w, h] = d.size();
    let (cx, cy) = (w / 2.0, h / 2.0);
    let s = osd.state;
    let blink = (osd.time * 2.0).fract() < 0.6;

    // Video breaks up as the link weakens.
    draw_interference(d, w, h, osd.link, osd.time);

    // Crosshair.
    d.fill([cx - 6.0, cy - 0.5, 4.0, 1.0], DIM);
    d.fill([cx + 2.0, cy - 0.5, 4.0, 1.0], DIM);
    d.fill([cx - 0.5, cy - 6.0, 1.0, 4.0], DIM);
    d.fill([cx - 0.5, cy + 2.0, 1.0, 4.0], DIM);

    // Artificial horizon: the world's level line, tilted by roll and moved by pitch.
    if let Some((pitch, roll)) = osd.horizon {
        // Where an angle above the view's centre lands on screen, through the projection.
        let focal = (h / 2.0) / (osd.fov.clamp(10.0, 170.0).to_radians() / 2.0).tan();
        let at_angle = |deg: f32| focal * deg.clamp(-80.0, 80.0).to_radians().tan();
        // Camera pitched up (negative) puts the horizon below the centre.
        let offset = at_angle(-pitch);
        // The world turns against the camera: rolling right (roll < 0) raises the horizon's
        // right end, as tilting your head right does.
        let (sin, cos) = (-roll).sin_cos();
        // Along the line (cos, -sin) in screen space (y down), moved along its normal by pitch.
        let at = |t: f32| [cx + cos * t + sin * offset, cy - sin * t + cos * offset];
        for (a, b) in [(-70.0, -18.0), (18.0, 70.0)] {
            line(d, at(a), at(b), 1.0, DIM);
        }
        // Ladder marks every 10 degrees above and below.
        for step in [-20.0_f32, -10.0, 10.0, 20.0] {
            let o = at_angle(-pitch + step);
            let m = |t: f32| [cx + cos * t + sin * o, cy - sin * t + cos * o];
            line(d, m(-14.0), m(-6.0), 1.0, 0x80FF_FFFF);
            line(d, m(6.0), m(14.0), 1.0, 0x80FF_FFFF);
        }
    }

    // Top: drone name, timer, link.
    let name = if osd.custom { format!("{} (custom)", osd.preset.name) } else { osd.preset.name.to_string() };
    text(d, &name, 6.0, 6.0, DIM);
    let t = s.flight_time as u32;
    text_centre(d, &format!("{:02}:{:02}", t / 60, t % 60), cx, 6.0, WHITE);
    let lq_color = if osd.link < 15.0 { WARN } else if osd.link < LOW_SIGNAL { AMBER } else { WHITE };
    let lq = format!("LQ {:3.0}  {:.0}m", osd.link, osd.distance);
    let lq_w = d.text_width(&lq);
    text_right(d, &lq, w - 6.0, 6.0, lq_color);
    signal_bars(d, w - 6.0 - lq_w - 15.0, 14.0, osd.link, lq_color);
    text_right(d, osd.view, w - 6.0, 17.0, DIM);

    // Battery, bottom left.
    let cells = s.physics.cells().max(1) as f32;
    let volts = s.physics.pack_voltage();
    let per_cell = volts / cells;
    let bat_color = if per_cell < 3.4 { WARN } else if per_cell < 3.6 { AMBER } else { WHITE };
    let y0 = h - 30.0;
    text(d, &format!("{volts:.1}V  {per_cell:.2}V/c"), 6.0, y0, bat_color);
    text(d, &format!("{:.0}mAh  {:.0}A", s.physics.mah_used, s.physics.pack_current), 6.0, y0 + 11.0, DIM);
    let charge = s.physics.charge();
    d.fill([6.0, y0 - 8.0, 40.0, 4.0], 0x60FF_FFFF);
    d.fill([6.0, y0 - 8.0, 40.0 * charge, 4.0], bat_color);

    // Speed and altitude, bottom right.
    let speed = s.velocity.length() * 3.6;
    let altitude = s.position[1] - s.launch[1];
    text_right(d, &format!("{speed:.0} km/h"), w - 6.0, y0, WHITE);
    text_right(d, &format!("ALT {altitude:.1} m"), w - 6.0, y0 + 11.0, DIM);

    // Throttle and home arrow, bottom centre.
    let thr = (s.throttle * 100.0).round();
    text_centre(d, &format!("THR {thr:.0}%"), cx, h - 19.0, WHITE);
    let home = (s.position[0] - s.launch[0]).hypot(s.position[2] - s.launch[2]);
    let a = osd.home_bearing.to_radians();
    let (ax, ay) = (cx, h - 40.0);
    let tip = [ax + a.sin() * 7.0, ay - a.cos() * 7.0];
    let left = [ax + (a + 2.5).sin() * 5.0, ay - (a + 2.5).cos() * 5.0];
    let right = [ax + (a - 2.5).sin() * 5.0, ay - (a - 2.5).cos() * 5.0];
    if home > 1.0 {
        d.quad([tip, right, [ax, ay], left], WHITE);
        text_centre(d, &format!("H {home:.0}m"), cx, ay + 6.0, DIM);
    }

    // Warnings, centre.
    let mut warnings: Vec<(&str, u32)> = Vec::new();
    if osd.link <= 0.0 {
        warnings.push(("FAILSAFE", WARN));
    } else if s.crashed {
        warnings.push(("CRASHED", WARN));
    } else if !s.armed {
        warnings.push(("DISARMED", WHITE));
    }
    if per_cell < 3.4 && s.armed {
        warnings.push(("LOW BATTERY", WARN));
    }
    if s.physics.is_overheat() {
        warnings.push(("MOTORS HOT", AMBER));
    }
    if osd.link > 0.0 && osd.link < LOW_SIGNAL {
        draw_low_signal(d, osd, cx, cy - 58.0);
    }
    let mut y = cy + 20.0;
    for (msg, color) in warnings {
        if blink || color == WHITE {
            text_centre(d, msg, cx, y, color);
        }
        y += 11.0;
    }

    // Waiting to arm: the drone's card and the keys.
    if !s.armed {
        let card_w = (w * 0.42).clamp(200.0, 320.0);
        let x = cx - card_w / 2.0;
        let lines = wrap(d, osd.preset.description, card_w - 12.0);
        let card_h = 40.0 + lines.len() as f32 * 10.0;
        let top = 34.0;
        d.fill([x, top, card_w, card_h], PANEL);
        text(d, osd.preset.name, x + 6.0, top + 5.0, WHITE);
        text_right(d, osd.preset.class, x + card_w - 6.0, top + 5.0, AMBER);
        for (i, l) in lines.iter().enumerate() {
            text(d, l, x + 6.0, top + 18.0 + i as f32 * 10.0, DIM);
        }
        let land = [Hint::Pad(PadButton::L1), Hint::Text("+"), Hint::Pad(PadButton::R1), Hint::Text(": land")];
        let (keys, pad): (&str, Vec<Hint>) = if s.crashed {
            (
                "Backspace: reset    H: land",
                [Hint::Pad(PadButton::West), Hint::Text(": reset    ")].into_iter().chain(land).collect(),
            )
        } else {
            (
                "Throttle down + R: arm    [ ]: drone    V: view    H: land",
                [
                    Hint::Text("Throttle down + "),
                    Hint::Pad(PadButton::North),
                    Hint::Text(": arm    "),
                    Hint::Pad(PadButton::Left),
                    Hint::Pad(PadButton::Right),
                    Hint::Text(": drone    "),
                    Hint::Pad(PadButton::Up),
                    Hint::Text(": view    "),
                ]
                .into_iter()
                .chain(land)
                .collect(),
            )
        };
        input_hint(d, osd, keys, &pad, cx, top + card_h - 12.0, 0xFF9AD0FF);
    }

    if osd.show_sticks {
        let hud = GimbalHud::new(s.hud_yaw, s.hud_throttle, s.hud_roll, s.hud_pitch);
        hud.render(&mut Painter(d), w as i32, h as i32 - 44);
    }
}
