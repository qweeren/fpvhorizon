//! The setup screen: the mod's own screen over the world (J, or a pad's Select while a drone is
//! out, or the button on the pause menu), drawn in the OSD's language: dark glass, an amber
//! edge, thin bars. Each page has a live picture at its top (the drone picker, the lens, the
//! rate curves, the sticks, the video link) over its settings, and every change applies at once.
//! The settings are the game's (`settings`), so Options > Mods shows the same values.
//!
//! The mouse drags, clicks and scrolls; the keyboard and a pad's D-pad move between rows and
//! set them. A drone in the air holds still while the screen is open.

pub mod model;

use crate::drone::{DroneBuildConfig, Sticks, PRESETS};
use crate::osd::{self, Hint, PadButton, PadFamily, AMBER, DIM, GLASS, HINT, WARN, WHITE};
use crate::settings::{self, MOD_ID};
use mod_api::{ClientHost, DrawContext};
use model::{Action, Hero, Kind, Page, Row, Value, RATE_PROFILES};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

const FAINT: u32 = 0x80FF_FFFF;
const LINE: u32 = 0x24FF_FFFF;
const TRACK: u32 = 0x40FF_FFFF;
const HOVER: u32 = 0x16FF_FFFF;
const INK: u32 = 0xFF10_1214;
/// The panel's glass: the badge's, a little darker for a long read.
const PANEL: u32 = 0xD010_1214;

/// GLFW's shift bit in an event's modifiers.
const SHIFT: u32 = 1;

/// What the screen shows of the drone and the pad, copied from the mod's state each frame.
#[derive(Clone, Copy, Default)]
pub struct Live {
    /// The pad's sticks, while the pad is what the player uses.
    pub sticks: Option<Sticks>,
    pub pad: Option<PadFamily>,
    /// A drone is out, and armed.
    pub drone_out: bool,
    pub armed: bool,
    /// The drone's distance from the pilot, while it is out.
    pub distance: f64,
}

/// Pad buttons for the screen, as the tick module's bits.
#[derive(Clone, Copy, Default)]
pub struct PadInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub select: bool,
    pub reset: bool,
    pub prev_page: bool,
    pub next_page: bool,
}

/// What a place on the screen does under the pointer.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Target {
    Tab(Page),
    /// A row as a whole: focus, and right-click for its default. The row's index on its page.
    Row(usize),
    /// A slider's bar, `[left, width]` of it.
    Slider(usize, f32, f32),
    /// One option of a choice.
    Segment(usize, usize),
    Profile(usize),
    Button(Action),
    PrevDrone,
    NextDrone,
    Reset,
    Done,
}

#[derive(Clone, Copy)]
struct Hit {
    rect: [f32; 4],
    target: Target,
}

fn inside(r: [f32; 4], p: [f32; 2]) -> bool {
    p[0] >= r[0] && p[0] < r[0] + r[2] && p[1] >= r[1] && p[1] < r[1] + r[3]
}

/// `r` cut to `clip`, or None when nothing is left.
fn cut(r: [f32; 4], clip: [f32; 4]) -> Option<[f32; 4]> {
    let x0 = r[0].max(clip[0]);
    let y0 = r[1].max(clip[1]);
    let x1 = (r[0] + r[2]).min(clip[0] + clip[2]);
    let y1 = (r[1] + r[3]).min(clip[1] + clip[3]);
    (x1 > x0 && y1 > y0).then_some([x0, y0, x1 - x0, y1 - y0])
}

/// The screen's state while it is open (and its page and scroll for the next time).
struct Ui {
    page: Page,
    scroll: f32,
    shown_scroll: f32,
    /// The row the keyboard or pad is on, by its index on the page.
    focus: Option<usize>,
    /// The focus follows the pointer until a key moves it.
    keys_lead: bool,
    /// The rate profile the keyboard is on, in the profiles row.
    profile: usize,
    /// A slider held by the mouse: its setting.
    drag: Option<usize>,
    mouse: [f32; 2],
    hits: Vec<Hit>,
    /// The page's content: its height and the visible part's, for the scroll.
    content_h: f32,
    view_h: f32,
    /// Row tops and heights on the page, from its first row, for bringing the focus into view.
    row_spans: Vec<(f32, f32)>,
    opened: Instant,
    last_frame: Instant,
    /// Reset was pressed once: a second press within a few seconds resets the page.
    confirm_reset: Option<Instant>,
    /// Done was pressed: the next tick closes the screen.
    close: bool,
    /// The toggles' knobs, easing to their side.
    knobs: HashMap<usize, f32>,
    /// The thrust-to-weight and hover of the last build looked at.
    performance: Option<([f32; 12], (f32, f32))>,
    /// A held D-pad direction and how long it has been held, for repeats.
    held: Option<(u8, u32)>,
}

impl Ui {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            page: Page::Drone,
            scroll: 0.0,
            shown_scroll: 0.0,
            focus: None,
            keys_lead: false,
            profile: 1,
            drag: None,
            mouse: [-1.0; 2],
            hits: Vec::new(),
            content_h: 0.0,
            view_h: 0.0,
            row_spans: Vec::new(),
            opened: now,
            last_frame: now,
            confirm_reset: None,
            close: false,
            knobs: HashMap::new(),
            performance: None,
            held: None,
        }
    }

    fn hit(&self, at: [f32; 2]) -> Option<Target> {
        self.hits.iter().rev().find(|h| inside(h.rect, at)).map(|h| h.target)
    }

    fn set_page(&mut self, page: Page) {
        if page != self.page {
            self.page = page;
            self.scroll = 0.0;
            self.shown_scroll = 0.0;
            self.focus = None;
            self.confirm_reset = None;
        }
    }
}

static UI: Mutex<Option<Ui>> = Mutex::new(None);
/// The pause menu's button was clicked: ticks left to close the pause menu and then open the
/// screen (the menu's close lands after the tick that asks for it).
static OPEN_FROM_PAUSE: Mutex<u8> = Mutex::new(0);

fn with_ui<R>(f: impl FnOnce(&mut Ui) -> R) -> R {
    let mut guard = UI.lock().unwrap();
    f(guard.get_or_insert_with(Ui::new))
}

/// Whether the setup screen is the open screen.
pub fn is_open(host: &dyn ClientHost) -> bool {
    host.mod_screen() == Some(MOD_ID)
}

/// Opens the screen (true when it opened).
pub fn open(host: &mut dyn ClientHost) -> bool {
    if !host.mod_screen_open(MOD_ID) {
        return false;
    }
    with_ui(|ui| {
        let now = Instant::now();
        ui.opened = now;
        ui.last_frame = now;
        ui.drag = None;
        ui.close = false;
        ui.confirm_reset = None;
        ui.held = None;
        ui.mouse = host.cursor();
        ui.keys_lead = false;
        ui.knobs.clear();
    });
    true
}

// The settings, read and kept through the game.

fn get(host: &dyn ClientHost, i: usize) -> Value {
    let s = &model::settings()[i];
    let read = match s.kind {
        Kind::Range { .. } => host.setting_f64(&s.key).map(Value::Range),
        Kind::Toggle { .. } => host.setting_bool(&s.key).map(Value::Toggle),
        Kind::Choice { .. } => host.setting_choice(&s.key).map(|c| Value::Choice(c as usize)),
    };
    read.unwrap_or_else(|| s.default_value())
}

fn set(host: &dyn ClientHost, i: usize, value: Value) {
    let s = &model::settings()[i];
    // Custom Build turned on with its sliders never touched starts from the chosen drone, not
    // from the sliders' generic defaults.
    let fresh_build = s.key == "custom_build" && value == Value::Toggle(true) && build_untouched(host);
    if let Some(text) = s.encode(value) {
        host.set_setting(MOD_ID, &s.key, &text);
    }
    if fresh_build {
        act(host, Action::CopyPreset);
    }
}

/// Whether every build slider is still at its default.
fn build_untouched(host: &dyn ClientHost) -> bool {
    crate::drone::BuildParam::ALL.iter().all(|p| {
        model::index(&format!("build_{}", p.key())).is_none_or(|i| get(host, i) == model::settings()[i].default_value())
    })
}

fn range(host: &dyn ClientHost, i: usize) -> f64 {
    match get(host, i) {
        Value::Range(v) => v,
        _ => 0.0,
    }
}

fn key_value(host: &dyn ClientHost, key: &str) -> Value {
    model::index(key).map_or(Value::Toggle(false), |i| get(host, i))
}

fn custom_build(host: &dyn ClientHost) -> bool {
    key_value(host, "custom_build") == Value::Toggle(true)
}

fn chosen(host: &dyn ClientHost) -> usize {
    settings::chosen_preset(host)
}

/// Whether the drone can be changed now: not while it flies.
fn drone_locked(live: &Live) -> bool {
    live.drone_out && live.armed
}

fn step_drone(host: &dyn ClientHost, live: &Live, by: i32) {
    if drone_locked(live) {
        return;
    }
    let n = PRESETS.len() as i32;
    let next = (chosen(host) as i32 + by).rem_euclid(n) as usize;
    if let Some(i) = model::index("drone") {
        set(host, i, Value::Choice(next));
    }
}

fn reset_setting(host: &dyn ClientHost, i: usize) {
    set(host, i, model::settings()[i].default_value());
}

fn act(host: &dyn ClientHost, action: Action) {
    match action {
        Action::CopyPreset => {
            let build = PRESETS[chosen(host)].build();
            for p in crate::drone::BuildParam::ALL {
                if let Some(i) = model::index(&format!("build_{}", p.key())) {
                    set(host, i, Value::Range(f64::from(build.get(p))));
                }
            }
        }
        Action::RateProfile(n) => {
            let p = RATE_PROFILES[n];
            for axis in model::RATE_AXES {
                let a = axis.key();
                for (key, v) in [("rc", p.rc), ("super", p.super_rate), ("expo", p.expo)] {
                    if let Some(i) = model::index(&format!("rate_{a}_{key}")) {
                        set(host, i, Value::Range(v));
                    }
                }
            }
        }
    }
}

/// The rate profile the three axes are on, if they all are on one.
fn current_profile(host: &dyn ClientHost) -> Option<usize> {
    RATE_PROFILES.iter().position(|p| {
        model::RATE_AXES.iter().all(|axis| {
            let a = axis.key();
            [("rc", p.rc), ("super", p.super_rate), ("expo", p.expo)]
                .iter()
                .all(|(key, v)| model::index(&format!("rate_{a}_{key}")).is_some_and(|i| (range(host, i) - v).abs() < 1e-6))
        })
    })
}

// Input.

/// A row the keyboard can stop on.
fn focusable(row: Row) -> bool {
    matches!(row, Row::Setting(_) | Row::RateProfiles | Row::Button(_) | Row::Hero(Hero::Drone))
}

/// The value of setting `i` moved `by` presses (a toggle flips, a choice steps).
fn adjust(host: &dyn ClientHost, i: usize, by: i32, fine: bool) {
    let s = &model::settings()[i];
    let next = match (&s.kind, get(host, i)) {
        (Kind::Range { .. }, Value::Range(v)) => Value::Range(s.nudge(v, by, fine)),
        (Kind::Toggle { .. }, Value::Toggle(on)) => Value::Toggle(!on),
        (Kind::Choice { options, .. }, Value::Choice(c)) => {
            Value::Choice((c as i32 + by).clamp(0, options.len() as i32 - 1) as usize)
        }
        _ => return,
    };
    set(host, i, next);
}

/// Left or right (`by`) on the focused row.
fn adjust_row(host: &dyn ClientHost, ui: &mut Ui, live: &Live, row: Row, by: i32, fine: bool) {
    match row {
        Row::Setting(i) => adjust(host, i, by, fine),
        Row::RateProfiles => ui.profile = (ui.profile as i32 + by).clamp(0, RATE_PROFILES.len() as i32 - 1) as usize,
        Row::Hero(Hero::Drone) => step_drone(host, live, by),
        _ => {}
    }
}

/// Enter (or a pad's confirm) on the focused row.
fn activate_row(host: &dyn ClientHost, ui: &mut Ui, live: &Live, row: Row) {
    match row {
        Row::Setting(i) => match model::settings()[i].kind {
            Kind::Toggle { .. } => adjust(host, i, 1, false),
            Kind::Choice { ref options, .. } => {
                let Value::Choice(c) = get(host, i) else { return };
                set(host, i, Value::Choice((c + 1) % options.len()));
            }
            Kind::Range { .. } => {}
        },
        Row::RateProfiles => act(host, Action::RateProfile(ui.profile)),
        Row::Button(a) => act(host, a),
        Row::Hero(Hero::Drone) => step_drone(host, live, 1),
        _ => {}
    }
}

fn press_reset(host: &dyn ClientHost, ui: &mut Ui) {
    if ui.confirm_reset.is_some_and(|t| t.elapsed().as_secs_f32() < 3.0) {
        for i in model::page_settings(ui.page) {
            reset_setting(host, i);
        }
        ui.confirm_reset = None;
    } else {
        ui.confirm_reset = Some(Instant::now());
    }
}

/// Moves the focus `by` focusable rows, and scrolls it into view.
fn move_focus(host: &dyn ClientHost, ui: &mut Ui, by: i32) {
    let rows = model::rows(ui.page, custom_build(host));
    let stops: Vec<usize> = (0..rows.len()).filter(|&r| focusable(rows[r])).collect();
    if stops.is_empty() {
        return;
    }
    let at = ui.focus.and_then(|f| stops.iter().position(|&s| s == f));
    let next = match at {
        Some(p) => (p as i32 + by).clamp(0, stops.len() as i32 - 1) as usize,
        None if by >= 0 => 0,
        None => stops.len() - 1,
    };
    ui.focus = Some(stops[next]);
    ui.keys_lead = true;
    if let Some(&(top, h)) = ui.row_spans.get(stops[next]) {
        // The first row brings the page's top along with it.
        let top = if next == 0 { 0.0 } else { top };
        if top < ui.scroll {
            ui.scroll = top;
        } else if top + h > ui.scroll + ui.view_h {
            ui.scroll = top + h - ui.view_h;
        }
        ui.scroll = ui.scroll.clamp(0.0, (ui.content_h - ui.view_h).max(0.0));
    }
}

fn focused_row(host: &dyn ClientHost, ui: &Ui) -> Option<Row> {
    let rows = model::rows(ui.page, custom_build(host));
    ui.focus.and_then(|f| rows.get(f).copied())
}

/// A mouse button went down at `at` on the screen.
pub fn mouse_pressed(host: &dyn ClientHost, live: &Live, at: [f32; 2], button: i32) {
    with_ui(|ui| {
        ui.mouse = at;
        ui.keys_lead = false;
        let Some(target) = ui.hit(at) else { return };
        let rows = model::rows(ui.page, custom_build(host));
        if button == 1 {
            // Right-click: the row's default.
            let row = match target {
                Target::Row(r) => rows.get(r).copied(),
                Target::Slider(i, ..) | Target::Segment(i, _) => Some(Row::Setting(i)),
                _ => None,
            };
            if let Some(Row::Setting(i)) = row {
                reset_setting(host, i);
            }
            return;
        }
        if button != 0 {
            return;
        }
        match target {
            Target::Tab(page) => ui.set_page(page),
            Target::Row(r) => {
                ui.focus = Some(r);
                if let Some(Row::Setting(i)) = rows.get(r).copied()
                    && matches!(model::settings()[i].kind, Kind::Toggle { .. })
                {
                    adjust(host, i, 1, false);
                }
            }
            Target::Slider(i, left, width) => {
                ui.drag = Some(i);
                let s = &model::settings()[i];
                set(host, i, Value::Range(s.at_fraction(f64::from((at[0] - left) / width))));
            }
            Target::Segment(i, option) => set(host, i, Value::Choice(option)),
            Target::Profile(n) => {
                ui.profile = n;
                act(host, Action::RateProfile(n));
            }
            Target::Button(a) => act(host, a),
            Target::PrevDrone => step_drone(host, live, -1),
            Target::NextDrone => step_drone(host, live, 1),
            Target::Reset => press_reset(host, ui),
            Target::Done => ui.close = true,
        }
    });
}

pub fn mouse_released() {
    with_ui(|ui| ui.drag = None);
}

/// The pointer moved to `at` with a button held. True while it drags a slider.
pub fn mouse_dragged(host: &dyn ClientHost, at: [f32; 2]) -> bool {
    with_ui(|ui| {
        ui.mouse = at;
        let Some(i) = ui.drag else { return false };
        let slider = ui.hits.iter().find_map(|h| match h.target {
            Target::Slider(j, left, width) if j == i => Some((left, width)),
            _ => None,
        });
        if let Some((left, width)) = slider {
            let s = &model::settings()[i];
            set(host, i, Value::Range(s.at_fraction(f64::from((at[0] - left) / width))));
        }
        true
    })
}

/// The wheel turned `dy` notches at `at`: a slider's bar moves its value, a choice steps,
/// anywhere else the page scrolls.
pub fn mouse_scrolled(host: &dyn ClientHost, at: [f32; 2], dy: f64, modifiers: u32) {
    with_ui(|ui| {
        ui.mouse = at;
        let notches = if dy > 0.0 { 1 } else if dy < 0.0 { -1 } else { 0 };
        match ui.hit(at) {
            Some(Target::Slider(i, ..)) => adjust(host, i, notches, modifiers & SHIFT != 0),
            Some(Target::Tab(_)) => {
                let page = ui.page.step(-notches);
                ui.set_page(page);
            }
            _ => {
                let max = (ui.content_h - ui.view_h).max(0.0);
                ui.scroll = (ui.scroll - dy as f32 * 24.0).clamp(0.0, max);
            }
        }
    });
}

/// A key went down. True when the screen used it.
pub fn key_pressed(host: &dyn ClientHost, live: &Live, key: &str, modifiers: u32) -> bool {
    with_ui(|ui| {
        let fine = modifiers & SHIFT != 0;
        let row = focused_row(host, ui);
        match key {
            "key.keyboard.up" => move_focus(host, ui, -1),
            "key.keyboard.down" => move_focus(host, ui, 1),
            "key.keyboard.page.up" => move_focus(host, ui, -6),
            "key.keyboard.page.down" => move_focus(host, ui, 6),
            "key.keyboard.home" => {
                ui.focus = None;
                move_focus(host, ui, 1);
            }
            "key.keyboard.end" => {
                ui.focus = None;
                move_focus(host, ui, -1);
            }
            "key.keyboard.left" | "key.keyboard.right" => {
                let by = if key.ends_with("left") { -1 } else { 1 };
                match row {
                    Some(row) => adjust_row(host, ui, live, row, by, fine),
                    None => move_focus(host, ui, 1),
                }
                ui.keys_lead = true;
            }
            "key.keyboard.enter" | "key.keyboard.keypad.enter" | "key.keyboard.space" => match row {
                Some(row) => activate_row(host, ui, live, row),
                None => move_focus(host, ui, 1),
            },
            "key.keyboard.tab" => {
                let page = ui.page.step(if fine { -1 } else { 1 });
                ui.set_page(page);
            }
            "key.keyboard.delete" | "key.keyboard.backspace" => {
                if let Some(Row::Setting(i)) = row {
                    reset_setting(host, i);
                }
            }
            _ => return false,
        }
        true
    })
}

/// Each client tick: the setup key, the pad's Select, and the pad's buttons on the screen.
/// `pressed` holds the presses since the last tick, `held` the buttons down now, and
/// `select_pressed` whether the pad's Select was pressed.
pub fn tick(host: &mut dyn ClientHost, live: &Live, pressed: PadInput, held: PadInput, select_pressed: bool) {
    let mut from_pause = false;
    {
        let mut wait = OPEN_FROM_PAUSE.lock().unwrap();
        if *wait > 0 {
            *wait -= 1;
            if host.playing() {
                *wait = 0;
                from_pause = true;
            } else if host.mod_screen().is_none() {
                host.close_screen();
            }
        }
    }
    if is_open(host) {
        // The key's clicks don't come while a screen is open; drop any from before.
        while host.consume_click(MOD_ID, settings::SETUP_KEY) {}
        let close = with_ui(|ui| {
            pad_navigation(host, ui, live, pressed, held);
            std::mem::take(&mut ui.close)
        });
        if close {
            host.mod_screen_close(MOD_ID);
        }
        return;
    }
    let mut want = from_pause;
    while host.consume_click(MOD_ID, settings::SETUP_KEY) {
        want |= host.playing();
    }
    // The pad opens it with Select while a drone is out (the pad flies the drone then, so
    // Select is free).
    want |= select_pressed && live.drone_out && host.playing();
    if want {
        open(host);
    }
}

fn pad_navigation(host: &dyn ClientHost, ui: &mut Ui, live: &Live, pressed: PadInput, held: PadInput) {
    // D-pad directions repeat while held: after a third of a second, five times a second.
    let dirs = [(0u8, pressed.up, held.up), (1, pressed.down, held.down), (2, pressed.left, held.left), (3, pressed.right, held.right)];
    let mut fire: Option<u8> = dirs.iter().find(|d| d.1).map(|d| d.0);
    match (fire, ui.held) {
        (Some(d), _) => ui.held = Some((d, 0)),
        (None, Some((d, n))) if dirs[d as usize].2 => {
            ui.held = Some((d, n + 1));
            if n + 1 >= 7 && (n + 1 - 7) % 4 == 0 {
                fire = Some(d);
            }
        }
        _ => ui.held = None,
    }
    let row = focused_row(host, ui);
    match fire {
        Some(0) => move_focus(host, ui, -1),
        Some(1) => move_focus(host, ui, 1),
        Some(d) => {
            let by = if d == 2 { -1 } else { 1 };
            match row {
                Some(row) => adjust_row(host, ui, live, row, by, false),
                None => move_focus(host, ui, 1),
            }
        }
        None => {}
    }
    if pressed.select {
        match focused_row(host, ui) {
            Some(row) => activate_row(host, ui, live, row),
            None => move_focus(host, ui, 1),
        }
    }
    if pressed.reset
        && let Some(Row::Setting(i)) = focused_row(host, ui)
    {
        reset_setting(host, i);
    }
    if pressed.prev_page || pressed.next_page {
        let page = ui.page.step(if pressed.next_page { 1 } else { -1 });
        ui.set_page(page);
        move_focus(host, ui, 1);
    }
}

// The pause menu's way in.

/// Where the pause menu's button sits.
fn pause_button(d: &dyn DrawContext) -> [f32; 4] {
    let w = d.text_width("FPV HORIZON SETUP") + 30.0;
    [6.0, 6.0, w, 18.0]
}

/// Draws the button to the setup screen on the pause menu.
pub fn draw_pause_button(d: &mut dyn DrawContext, mouse: [f32; 2]) {
    let r = pause_button(d);
    let hover = inside(r, mouse);
    let time = crate::time();
    d.fill(r, if hover { 0xC81A_1D20 } else { GLASS });
    let pulse = if hover { 1.0 } else { 0.75 + 0.25 * (time * 2.0).sin() };
    d.fill([r[0], r[1], r[2], 1.0], osd::fade(AMBER, pulse));
    osd::signal_bars(d, r[0] + 6.0, r[1] + 13.0, 100.0, AMBER);
    osd::text(d, "FPV HORIZON SETUP", r[0] + 22.0, r[1] + 6.0, if hover { WHITE } else { DIM });
    with_ui(|ui| ui.hits = vec![Hit { rect: r, target: Target::Done }]);
}

/// A click on the pause menu: true when it was on the setup button (the next tick opens it).
pub fn pause_click(at: [f32; 2]) -> bool {
    let on = with_ui(|ui| ui.hit(at) == Some(Target::Done));
    if on {
        *OPEN_FROM_PAUSE.lock().unwrap() = 10;
    }
    on
}

// Drawing.

fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Draws the screen. `mouse` is the pointer, `setup_key` the key's name for the hints.
pub fn draw(d: &mut dyn DrawContext, host: &dyn ClientHost, live: &Live, mouse: [f32; 2]) {
    let mut guard = UI.lock().unwrap();
    let ui = guard.get_or_insert_with(Ui::new);
    let now = Instant::now();
    let dt = now.duration_since(ui.last_frame).as_secs_f32().min(0.1);
    ui.last_frame = now;
    if mouse != ui.mouse {
        ui.mouse = mouse;
        ui.keys_lead = false;
    }
    ui.shown_scroll += (ui.scroll - ui.shown_scroll) * (dt * 16.0).min(1.0);
    if (ui.scroll - ui.shown_scroll).abs() < 0.3 {
        ui.shown_scroll = ui.scroll;
    }
    ui.hits.clear();
    let time = crate::time();
    let [w, h] = d.size();

    // The world dims a little top and bottom, as the drone picker's does.
    let intro = ease_out(ui.opened.elapsed().as_secs_f32() / 0.22);
    osd::gradient(d, [0.0, 0.0, w, h * 0.3], osd::fade(0xB000_0000, intro), 0);
    osd::gradient(d, [0.0, h * 0.7, w, h * 0.3], 0, osd::fade(0xB000_0000, intro));

    let margin = 8.0;
    let pw = (w * 0.64).clamp(296.0, 460.0).min(w - 2.0 * margin);
    let px = margin - (1.0 - intro) * 14.0;
    let py = margin;
    let ph = h - 2.0 * margin;
    d.fill([px, py, pw, ph], PANEL);
    d.fill([px, py, pw, 1.0], AMBER);

    let paused = live.drone_out && live.armed;
    draw_header(d, live, [px, py, pw], paused, time);

    let header_h = 30.0;
    let footer_h = 36.0;
    let rail_w = 74.0;
    let body_y = py + header_h;
    let body_h = ph - header_h - footer_h;
    d.fill([px, body_y - 1.0, pw, 1.0], LINE);
    d.fill([px + rail_w, body_y, 1.0, body_h], LINE);
    d.fill([px, body_y + body_h, pw, 1.0], LINE);

    // The rail: the pages, and Reset and Done at its foot.
    for (n, page) in Page::ALL.iter().enumerate() {
        let r = [px, body_y + 4.0 + n as f32 * 18.0, rail_w, 18.0];
        let selected = *page == ui.page;
        let hover = inside(r, mouse);
        if selected {
            d.fill(r, 0x26FF_FFFF);
            d.fill([r[0], r[1], 2.0, r[3]], AMBER);
        } else if hover {
            d.fill(r, HOVER);
        }
        let color = if selected || hover { WHITE } else { DIM };
        osd::text(d, page.title(), r[0] + 9.0, r[1] + 5.0, color);
        ui.hits.push(Hit { rect: r, target: Target::Tab(*page) });
    }
    let done = [px + 6.0, body_y + body_h - 22.0, rail_w - 12.0, 16.0];
    let reset = [px + 6.0, done[1] - 20.0, rail_w - 12.0, 16.0];
    let confirming = ui.confirm_reset.is_some_and(|t| t.elapsed().as_secs_f32() < 3.0);
    if !confirming {
        ui.confirm_reset = None;
    }
    if reset[1] > body_y + 4.0 + Page::ALL.len() as f32 * 18.0 + 4.0 {
        let label = if confirming { "SURE?" } else { "RESET PAGE" };
        button(d, reset, label, inside(reset, mouse), if confirming { Style::Warn } else { Style::Plain });
        ui.hits.push(Hit { rect: reset, target: Target::Reset });
    }
    button(d, done, "DONE", inside(done, mouse), Style::Primary);
    ui.hits.push(Hit { rect: done, target: Target::Done });

    // The page.
    let view = [px + rail_w + 1.0, body_y, pw - rail_w - 1.0, body_h];
    let help = draw_page(d, host, ui, live, view, mouse, time, dt);

    // The footer: what the row under the pointer does, and the keys.
    let fy = body_y + body_h + 1.0;
    let help = help.unwrap_or_else(|| page_blurb(ui.page).to_string());
    let lines = clipped_lines(d, &help, pw - 16.0, 2);
    for (n, l) in lines.iter().enumerate() {
        osd::text(d, l, px + 8.0, fy + 4.0 + n as f32 * 9.0, DIM);
    }
    let hint_y = fy + footer_h - 10.0;
    match live.pad {
        Some(family) => {
            let hint = [
                Hint::Pad(PadButton::Up),
                Hint::Pad(PadButton::Down),
                Hint::Text(": choose   "),
                Hint::Pad(PadButton::Left),
                Hint::Pad(PadButton::Right),
                Hint::Text(": set   "),
                Hint::Pad(PadButton::South),
                Hint::Text(": select   "),
                Hint::Pad(PadButton::L1),
                Hint::Pad(PadButton::R1),
                Hint::Text(": page   "),
                Hint::Pad(PadButton::East),
                Hint::Text(": close"),
            ];
            let width = osd::hint_width(d, &hint, family);
            osd::hint_centre(d, &hint, family, px + 8.0 + width / 2.0, hint_y, FAINT);
        }
        None => {
            let hint = "Drag or wheel: set    Right-click: default    Tab: page    Esc: close";
            let hint = if d.text_width(hint) <= pw - 16.0 { hint } else { "Right-click: default    Tab: page    Esc: close" };
            osd::text(d, hint, px + 8.0, hint_y, FAINT);
        }
    }
}

fn page_blurb(page: Page) -> &'static str {
    match page {
        Page::Drone => "Pick a real drone to fly, or build your own from its parts.",
        Page::Camera => "What the drone's camera sees, and what is drawn over it.",
        Page::Rates => "How fast the drone turns at each stick position, as Betaflight's rates do.",
        Page::Controls => "How the sticks feel, and the keys and buttons that fly.",
        Page::Link => "How far the video reaches. Too far and the picture breaks up, then the drone disarms.",
        Page::Sound => "How loud the drone is, as you hear it.",
        Page::Physics => "The world the drone flies in.",
    }
}

fn draw_header(d: &mut dyn DrawContext, live: &Live, [px, py, pw]: [f32; 3], paused: bool, time: f32) {
    osd::signal_bars(d, px + 9.0, py + 21.0, 100.0, AMBER);
    crate::font::draw(d, "FPV HORIZON", px + 26.0, py + 8.0, 2.0, WHITE);
    let right = px + pw - 8.0;
    osd::text_right(d, "SETUP", right, py + 6.0, DIM);
    if paused {
        let pulse = 0.7 + 0.3 * (time * 3.5).sin();
        osd::text_right(d, "FLIGHT PAUSED", right, py + 16.0, osd::fade(AMBER, pulse));
    } else if live.drone_out {
        osd::text_right(d, "ON THE GROUND", right, py + 16.0, FAINT);
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Style {
    Plain,
    Primary,
    Warn,
}

/// Words centred on `cx`, without the shadow: dark words on amber, where a shadow smudges.
fn ink_centre(d: &mut dyn DrawContext, s: &str, cx: f32, y: f32, color: u32) {
    let w = d.text_width(s);
    d.text(s, [cx - w / 2.0, y], color, false);
}

/// Words on a filled chip: dark on amber without a shadow, light ones as the OSD writes them.
fn chip_text(d: &mut dyn DrawContext, s: &str, cx: f32, y: f32, color: u32) {
    if color == INK { ink_centre(d, s, cx, y, color) } else { osd::text_centre(d, s, cx, y, color) }
}

fn button(d: &mut dyn DrawContext, r: [f32; 4], label: &str, hover: bool, style: Style) {
    let (bg, fg, edge) = match style {
        Style::Primary => (if hover { 0xFFFF_D470 } else { AMBER }, INK, 0),
        Style::Warn => (0x60FF_5050, WHITE, WARN),
        Style::Plain => (if hover { 0x34FF_FFFF } else { 0x1EFF_FFFF }, if hover { WHITE } else { DIM }, if hover { AMBER } else { 0 }),
    };
    d.fill(r, bg);
    if edge != 0 {
        d.fill([r[0], r[1], r[2], 1.0], edge);
    }
    chip_text(d, label, r[0] + r[2] / 2.0, r[1] + (r[3] - 8.0) / 2.0, fg);
}

fn row_height(row: Row, d: &dyn DrawContext, width: f32) -> f32 {
    match row {
        Row::Hero(Hero::Drone) => 110.0,
        Row::Hero(Hero::Camera) => 92.0,
        Row::Hero(Hero::Rates) => 106.0,
        Row::Hero(Hero::Sticks) => 104.0,
        Row::Hero(Hero::Link) => 106.0,
        Row::Hero(Hero::Physics) => 40.0,
        Row::Header(_) => 18.0,
        Row::Setting(i) => match model::settings()[i].kind {
            Kind::Range { .. } => 26.0,
            Kind::Toggle { .. } => 20.0,
            Kind::Choice { .. } => 30.0,
        },
        Row::RateProfiles => 22.0,
        Row::Button(_) => 22.0,
        Row::Keys => 9.0 * 11.0 + 4.0,
        Row::Note(s) => 6.0 + osd::wrap(d, s, width).len() as f32 * 10.0,
    }
}

/// Draws the page into `view` and records its hits. The help for the row under the pointer
/// (or the focused one), if there is one.
#[allow(clippy::too_many_arguments)]
fn draw_page(d: &mut dyn DrawContext, host: &dyn ClientHost, ui: &mut Ui, live: &Live, view: [f32; 4], mouse: [f32; 2], time: f32, dt: f32) -> Option<String> {
    let custom = custom_build(host);
    let rows = model::rows(ui.page, custom);
    let x0 = view[0] + 10.0;
    let x1 = view[0] + view[2] - 12.0;
    let width = x1 - x0;
    let pad_top = 6.0;

    // Lay the rows out from the page's top.
    let mut spans = Vec::with_capacity(rows.len());
    let mut y = pad_top;
    for row in &rows {
        let rh = row_height(*row, d, width);
        spans.push((y, rh));
        y += rh;
    }
    ui.content_h = y + 8.0;
    ui.view_h = view[3];
    ui.row_spans = spans.clone();
    let max = (ui.content_h - ui.view_h).max(0.0);
    ui.scroll = ui.scroll.clamp(0.0, max);
    ui.shown_scroll = ui.shown_scroll.clamp(0.0, max);
    if ui.focus.is_some_and(|f| f >= rows.len()) {
        ui.focus = None;
    }

    let mouse_in = inside(view, mouse);
    let hovered_row = if mouse_in && !ui.keys_lead {
        spans.iter().position(|&(top, rh)| {
            let y = view[1] + top - ui.shown_scroll;
            mouse[1] >= y && mouse[1] < y + rh
        })
    } else {
        None
    };
    if !ui.keys_lead && hovered_row.is_some_and(|r| focusable(rows[r])) {
        ui.focus = hovered_row;
    }
    let lit = if ui.keys_lead { ui.focus } else { hovered_row.filter(|r| focusable(rows[*r])) };
    let mut help: Option<String> = None;

    let _ = d.push_clip(view);
    for (r, row) in rows.iter().enumerate() {
        let (top, rh) = spans[r];
        let y = view[1] + top - ui.shown_scroll;
        if y + rh < view[1] || y > view[1] + view[3] {
            continue;
        }
        let row_rect = [x0 - 5.0, y, width + 10.0, rh];
        let on = lit == Some(r);
        if on && !matches!(row, Row::Hero(_)) {
            d.fill(row_rect, HOVER);
            d.fill([row_rect[0], y, 1.0, rh], AMBER);
        }
        let push = |ui: &mut Ui, rect: [f32; 4], target: Target| {
            if let Some(rect) = cut(rect, view) {
                ui.hits.push(Hit { rect, target });
            }
        };
        if focusable(*row) && !matches!(row, Row::Hero(_)) {
            push(ui, row_rect, Target::Row(r));
        }
        match *row {
            Row::Hero(hero) => {
                let card = [x0 - 5.0, y + 2.0, width + 10.0, rh - 8.0];
                d.fill(card, 0x44000000);
                d.fill([card[0], card[1], card[2], 1.0], osd::fade(AMBER, if on { 0.9 } else { 0.5 }));
                match hero {
                    Hero::Drone => {
                        let hits = hero_drone(d, host, ui, live, card, mouse, time);
                        for (rect, t) in hits {
                            push(ui, rect, t);
                        }
                        if on {
                            help = Some(if drone_locked(live) {
                                "Land the drone to change it.".to_string()
                            } else {
                                "Click the arrows, or press left and right, to change drone.".to_string()
                            });
                        }
                    }
                    Hero::Camera => hero_camera(d, host, card, time),
                    Hero::Rates => {
                        let axis = hovered_row.or(ui.focus).and_then(|r| match rows.get(r) {
                            Some(Row::Setting(i)) => model::rate_axis(&model::settings()[*i].key),
                            _ => None,
                        });
                        hero_rates(d, host, live, card, axis);
                    }
                    Hero::Sticks => hero_sticks(d, host, live, card),
                    Hero::Link => hero_link(d, host, live, card, time),
                    Hero::Physics => {
                        osd::text(d, "ADVANCED", card[0] + 6.0, card[1] + 6.0, AMBER);
                        let words = "These change how every drone flies. Right-click a row for its default.";
                        for (n, l) in osd::wrap(d, words, card[2] - 12.0).iter().take(2).enumerate() {
                            osd::text(d, l, card[0] + 6.0, card[1] + 17.0 + n as f32 * 9.0, DIM);
                        }
                    }
                }
            }
            Row::Header(title) => {
                osd::text(d, title, x0, y + 7.0, osd::fade(AMBER, 0.9));
                let tw = d.text_width(title);
                d.fill([x0 + tw + 5.0, y + 11.0, (width - tw - 5.0).max(0.0), 1.0], LINE);
            }
            Row::Note(words) => {
                for (n, l) in osd::wrap(d, words, width).iter().enumerate() {
                    osd::text(d, l, x0, y + 4.0 + n as f32 * 10.0, FAINT);
                }
            }
            Row::Setting(i) => {
                let s = &model::settings()[i];
                let value = get(host, i);
                let modified = value != s.default_value();
                let label_color = if on || ui.drag == Some(i) { WHITE } else { DIM };
                if on || ui.drag == Some(i) {
                    help = Some(s.help.to_string());
                }
                let label_y = match s.kind {
                    Kind::Toggle { .. } => y + 6.0,
                    _ => y + 4.0,
                };
                osd::text(d, &s.label, x0, label_y, label_color);
                if modified {
                    // A changed value is marked, as an edited field is.
                    let lw = d.text_width(&s.label);
                    d.fill([x0 + lw + 3.0, label_y + 3.0, 2.0, 2.0], AMBER);
                }
                match s.kind {
                    Kind::Range { default, .. } => {
                        let Value::Range(v) = value else { continue };
                        let dragging = ui.drag == Some(i);
                        osd::text_right(d, &s.format(value), x1, y + 4.0, if dragging { AMBER } else { WHITE });
                        let f = s.fraction(v) as f32;
                        let ty = y + 17.0;
                        d.fill([x0, ty, width, 2.0], TRACK);
                        d.fill([x0, ty, width * f, 2.0], AMBER);
                        let df = s.fraction(default) as f32;
                        d.fill([x0 + width * df - 0.5, ty - 2.0, 1.0, 6.0], 0x90FF_FFFF);
                        let hx = x0 + width * f;
                        d.fill([hx - 2.0, ty - 4.0, 5.0, 10.0], INK);
                        d.fill([hx - 1.0, ty - 3.0, 3.0, 8.0], if dragging || on { WHITE } else { 0xFFD8_D8D8 });
                        push(ui, [x0 - 4.0, y + 10.0, width + 8.0, 14.0], Target::Slider(i, x0, width));
                    }
                    Kind::Toggle { .. } => {
                        let Value::Toggle(state) = value else { continue };
                        let knob = ui.knobs.entry(i).or_insert(if state { 1.0 } else { 0.0 });
                        let goal = if state { 1.0 } else { 0.0 };
                        *knob += (goal - *knob) * (dt * 18.0).min(1.0);
                        let k = *knob;
                        let sw = [x1 - 22.0, y + 5.0, 22.0, 10.0];
                        d.fill(sw, TRACK);
                        d.fill([sw[0], sw[1], sw[2] * k, sw[3]], osd::fade(AMBER, 0.9));
                        let kx = sw[0] + 1.0 + k * 12.0;
                        d.fill([kx, sw[1] + 1.0, 8.0, 8.0], if state { WHITE } else { 0xFFB8_B8B8 });
                        osd::text_right(d, if state { "ON" } else { "OFF" }, sw[0] - 5.0, y + 6.0, if state { AMBER } else { FAINT });
                    }
                    Kind::Choice { ref options, .. } => {
                        let Value::Choice(c) = value else { continue };
                        let n = options.len() as f32;
                        let gap = 2.0;
                        let sw = (width - gap * (n - 1.0)) / n;
                        for (k, (_, label)) in options.iter().enumerate() {
                            let r = [x0 + k as f32 * (sw + gap), y + 15.0, sw, 12.0];
                            let hover = inside(r, mouse) && mouse_in;
                            let (bg, fg) = if k == c {
                                (AMBER, INK)
                            } else if hover {
                                (0x3AFF_FFFF, WHITE)
                            } else {
                                (0x1EFF_FFFF, DIM)
                            };
                            d.fill(r, bg);
                            chip_text(d, label, r[0] + sw / 2.0, r[1] + 2.0, fg);
                            push(ui, r, Target::Segment(i, k));
                        }
                    }
                }
            }
            Row::RateProfiles => {
                let current = current_profile(host);
                let n = RATE_PROFILES.len() as f32;
                let gap = 2.0;
                let sw = (width - gap * (n - 1.0)) / n;
                for (k, p) in RATE_PROFILES.iter().enumerate() {
                    let r = [x0 + k as f32 * (sw + gap), y + 4.0, sw, 14.0];
                    let hover = inside(r, mouse) && mouse_in;
                    let picked = current == Some(k);
                    let keyed = on && ui.keys_lead && ui.profile == k;
                    let (bg, fg) = if picked {
                        (AMBER, INK)
                    } else if hover || keyed {
                        (0x3AFF_FFFF, WHITE)
                    } else {
                        (0x1EFF_FFFF, DIM)
                    };
                    d.fill(r, bg);
                    if keyed {
                        d.fill([r[0], r[1] + r[3] - 1.0, r[2], 1.0], WHITE);
                    }
                    chip_text(d, p.name, r[0] + sw / 2.0, r[1] + 3.0, fg);
                    push(ui, r, Target::Profile(k));
                    if hover || keyed {
                        let max = model::rate_at(p.rc, p.super_rate, p.expo, 1.0);
                        help = Some(format!("{}: all three axes, up to {max:.0}°/s at full stick.", p.name));
                    }
                }
                if on && help.is_none() {
                    help = Some("Rate profiles: set all three axes at once, then fine-tune below.".to_string());
                }
            }
            Row::Button(action) => {
                let r = [x0, y + 3.0, width, 16.0];
                let label = match action {
                    Action::CopyPreset => format!("Start from the {}", PRESETS[chosen(host)].name),
                    Action::RateProfile(n) => RATE_PROFILES[n].name.to_string(),
                };
                button(d, r, &label, on, Style::Plain);
                push(ui, r, Target::Button(action));
                if on {
                    help = Some("Copies the chosen drone's real parts into the sliders below.".to_string());
                }
            }
            Row::Keys => draw_keys(d, host, live, [x0, y, width]),
        }
    }
    let _ = d.pop_clip();

    // The scroll bar, while the page is taller than the view.
    if ui.content_h > ui.view_h + 1.0 {
        let bar_h = (view[3] * view[3] / ui.content_h).max(12.0);
        let bar_y = view[1] + (view[3] - bar_h) * (ui.shown_scroll / max.max(1.0));
        d.fill([view[0] + view[2] - 4.0, view[1], 2.0, view[3]], 0x14FF_FFFF);
        d.fill([view[0] + view[2] - 4.0, bar_y, 2.0, bar_h], 0x70FF_FFFF);
    }
    help
}

/// The drone picker: maker and model, class, the dots for the others, its words and how it
/// flies. Gives its arrows' hits.
fn hero_drone(d: &mut dyn DrawContext, host: &dyn ClientHost, ui: &mut Ui, live: &Live, card: [f32; 4], mouse: [f32; 2], time: f32) -> Vec<([f32; 4], Target)> {
    let [x, y, w, h] = card;
    let cx = x + w / 2.0;
    let index = chosen(host);
    let p = &PRESETS[index];
    let custom = custom_build(host);
    let locked = drone_locked(live);

    // The arrows, pulsing as the turntable's do.
    let mut hits = Vec::new();
    for (left, target) in [(true, Target::PrevDrone), (false, Target::NextDrone)] {
        let r = if left { [x + 2.0, y + 6.0, 22.0, 30.0] } else { [x + w - 24.0, y + 6.0, 22.0, 30.0] };
        let hover = inside(r, mouse);
        let alpha = if locked { 0.25 } else if hover { 1.0 } else { 0.6 + 0.25 * (time * 3.0).sin() };
        let color = osd::fade(if hover { AMBER } else { WHITE }, alpha);
        crate::font::draw_centred(d, if left { "<" } else { ">" }, r[0] + r[2] / 2.0, r[1] + 8.0, 2.0, color);
        hits.push((r, target));
    }

    let (maker, model_name) = p.name.split_once(' ').unwrap_or(("", p.name));
    osd::text_centre(d, maker, cx, y + 5.0, FAINT);
    let px = if crate::font::width(model_name, 2.0) <= w - 56.0 { 2.0 } else { 1.0 };
    crate::font::draw_centred(d, model_name, cx, y + 15.0, px, WHITE);
    let class = if custom { format!("{} (custom)", p.class) } else { p.class.to_string() };
    osd::text_centre(d, &class, cx, y + 32.0, AMBER);

    // One dot a drone, the chosen one lit; while flying, why it can't change.
    if locked {
        osd::text_centre(d, "Land to change drone", cx, y + 43.0, osd::fade(AMBER, 0.8));
    } else {
        let n = PRESETS.len() as f32;
        let left = cx - (n * 6.0 - 2.0) / 2.0;
        for k in 0..PRESETS.len() {
            let c = if k == index { AMBER } else { 0x50FF_FFFF };
            d.fill([left + k as f32 * 6.0, y + 45.0, 4.0, 2.0], c);
        }
    }
    for (n, l) in clipped_lines(d, p.description, w - 16.0, 2).iter().enumerate() {
        osd::text_centre(d, l, cx, y + 54.0 + n as f32 * 10.0, DIM);
    }

    // How it flies: the build that will fly, preset or sliders.
    let build = settings::build(host, index);
    let (twr, hover) = performance(ui, &build);
    use crate::drone::BuildParam as B;
    let cells = [
        ("THRUST", format!("{twr:.1}:1")),
        ("HOVER", format!("{:.0}%", hover * 100.0)),
        ("PACK", format!("{:.0}S {:.0}", build.get(B::BatteryCells), build.get(B::BatteryMah))),
        ("WEIGHT", format!("{:.0} g", build.get(B::DroneMass) * 1000.0)),
    ];
    let cw = (w - 8.0) / cells.len() as f32;
    let sy = y + h - 22.0;
    d.fill([x + 4.0, sy - 3.0, w - 8.0, 1.0], LINE);
    for (n, (caption, value)) in cells.iter().enumerate() {
        let ccx = x + 4.0 + cw * (n as f32 + 0.5);
        osd::text_centre(d, caption, ccx, sy, FAINT);
        osd::text_centre(d, value, ccx, sy + 9.0, WHITE);
    }
    hits
}

/// `s` wrapped to `width`, at most `max` lines, the last ending in an ellipsis when cut.
fn clipped_lines(d: &dyn DrawContext, s: &str, width: f32, max: usize) -> Vec<String> {
    let mut lines = osd::wrap(d, s, width);
    if lines.len() > max {
        lines.truncate(max);
        let last = &mut lines[max - 1];
        while !last.is_empty() && d.text_width(&format!("{last}...")) > width {
            match last.rfind(' ') {
                Some(i) => last.truncate(i),
                None => last.clear(),
            }
        }
        let trimmed = last.trim_end_matches([',', '.', ':', ';']).to_string();
        *last = format!("{trimmed}...");
    }
    lines
}

/// The thrust-to-weight and hover throttle of `build`, worked out once per build.
fn performance(ui: &mut Ui, build: &DroneBuildConfig) -> (f32, f32) {
    match ui.performance {
        Some((values, result)) if values == build.values => result,
        _ => {
            let result = crate::drone::presets::performance(build);
            ui.performance = Some((build.values, result));
            result
        }
    }
}

/// The lens: a test card of lines every 15 degrees seen through the field of view and the
/// fisheye, and the camera's view cone from the side, with its uptilt.
fn hero_camera(d: &mut dyn DrawContext, host: &dyn ClientHost, card: [f32; 4], time: f32) {
    let [x, y, w, h] = card;
    let fov = match key_value(host, "fpv_fov") {
        Value::Range(v) => v as f32,
        _ => 110.0,
    };
    let lens = match key_value(host, "fisheye") {
        Value::Choice(c) => c.min(3),
        _ => model::FISHEYE_DEFAULT,
    };
    let strength = model::FISHEYE_STRENGTH[lens];
    let build = settings::build(host, chosen(host));
    let uptilt = build.get(crate::drone::BuildParam::CameraAngle);

    // The monitor, 16:9.
    let mh = h - 22.0;
    let mw = (mh * 16.0 / 9.0).min(w * 0.55);
    let mh = mw * 9.0 / 16.0;
    let mx = x + 6.0;
    let my = y + 6.0;
    let aspect = 16.0 / 9.0;
    d.fill([mx, my, mw, mh], 0x70000000);
    let _ = d.push_clip([mx, my, mw, mh]);
    let half = (fov.clamp(10.0, 170.0) / 2.0).to_radians().tan();
    let to_px = |p: [f32; 2]| [mx + (p[0] + 0.5) * mw, my + (0.5 - p[1]) * mh];
    // Lines every 15 degrees out from the centre, each bent through the lens.
    for k in -6i32..=6 {
        let a = (k as f32 * 15.0).to_radians();
        let off = a.tan() / (2.0 * half);
        let color = if k == 0 { 0x90FF_FFFF } else { 0x40FF_FFFF };
        // Vertical line: x at `off / aspect` of the width.
        let vx = off / aspect;
        if vx.abs() <= 0.5 {
            let mut prev = None;
            for s in 0..=16 {
                let v = -0.5 + s as f32 / 16.0;
                let p = to_px(model::fisheye_to_screen([vx, v], aspect, strength));
                if let Some(q) = prev {
                    osd::line(d, q, p, 1.0, color);
                }
                prev = Some(p);
            }
        }
        if off.abs() <= 0.5 {
            let mut prev = None;
            for s in 0..=16 {
                let u = -0.5 + s as f32 / 16.0;
                let p = to_px(model::fisheye_to_screen([u, off], aspect, strength));
                if let Some(q) = prev {
                    osd::line(d, q, p, 1.0, color);
                }
                prev = Some(p);
            }
        }
    }
    // The lens's corners darken a little, as the shader's vignette does.
    if strength > 0.0 {
        let edge = osd::fade(0xFF00_0000, strength * 0.5);
        osd::gradient(d, [mx, my, mw, mh * 0.2], edge, 0);
        osd::gradient(d, [mx, my + mh * 0.8, mw, mh * 0.2], 0, edge);
    }
    let _ = d.pop_clip();
    for r in [[mx, my, mw, 1.0], [mx, my + mh - 1.0, mw, 1.0], [mx, my, 1.0, mh], [mx + mw - 1.0, my, 1.0, mh]] {
        d.fill(r, 0x60FF_FFFF);
    }
    let lens_name = ["NO FISHEYE", "LIGHT FISHEYE", "MEDIUM FISHEYE", "STRONG FISHEYE"][lens];
    osd::text(d, &format!("{fov:.0}°"), mx, my + mh + 4.0, WHITE);
    osd::text_right(d, lens_name, mx + mw, my + mh + 4.0, FAINT);

    // The camera from the side: the drone tilts forward to fly fast, and the uptilt keeps the
    // camera looking ahead. It eases between a hover and a cruise every few seconds.
    let sx = mx + mw + 10.0;
    let sw = x + w - 6.0 - sx;
    if sw < 60.0 {
        return;
    }
    let cycle = (time / 3.0) as u32 % 2 == 1;
    let phase = (time / 3.0).fract();
    let blend = ease_out((phase / 0.35).min(1.0));
    let tilt = if cycle { uptilt * blend } else { uptilt * (1.0 - blend) };
    let pivot = [sx + 16.0, y + 6.0 + mh * 0.62];
    // Cruising, the drone pitches nose down (its front is to the right, y grows downward).
    let pitch = tilt.to_radians();
    let _ = d.push_clip([sx - 4.0, y + 1.0, x + w - sx + 4.0, mh + 10.0]);
    let rot = |p: [f32; 2]| {
        let (s, c) = pitch.sin_cos();
        [pivot[0] + p[0] * c - p[1] * s, pivot[1] + p[0] * s + p[1] * c]
    };
    // The view cone, from the camera, up by the uptilt, as wide as the field of view.
    let len = (sw - 22.0).min(120.0);
    let cam = rot([4.0, -2.0]);
    let look = -(uptilt).to_radians() + pitch;
    let spread = (fov / 2.0).to_radians();
    let ray = |a: f32| [cam[0] + a.cos() * len, cam[1] + a.sin() * len];
    let (top, bottom) = (ray(look - spread), ray(look + spread));
    // The cone, filled in thin wedges from the camera.
    let slices = 12;
    for k in 0..slices {
        let a0 = look - spread + 2.0 * spread * k as f32 / slices as f32;
        let a1 = look - spread + 2.0 * spread * (k + 1) as f32 / slices as f32;
        d.quad([cam, ray(a0), ray(a1), cam], 0x1CFF_C040);
    }
    osd::line(d, cam, top, 1.0, 0xA0FF_C040);
    osd::line(d, cam, bottom, 1.0, 0xA0FF_C040);
    osd::line(d, cam, ray(look), 1.0, 0x50FF_FFFF);
    // The drone: frame, props and camera.
    osd::line(d, rot([-12.0, 0.0]), rot([12.0, 0.0]), 2.0, WHITE);
    osd::line(d, rot([-14.0, -3.0]), rot([-6.0, -3.0]), 1.0, DIM);
    osd::line(d, rot([6.0, -3.0]), rot([14.0, -3.0]), 1.0, DIM);
    d.fill([cam[0] - 1.5, cam[1] - 1.5, 3.0, 3.0], AMBER);
    let _ = d.pop_clip();
    let words = if cycle { "CRUISING" } else { "HOVERING" };
    osd::text(d, &format!("UPTILT {uptilt:.0}°"), sx, my + mh + 4.0, FAINT);
    let ww = d.text_width(words);
    d.fill([x + w - 10.0 - ww, y + 3.0, ww + 8.0, 11.0], GLASS);
    osd::text_right(d, words, x + w - 6.0, y + 4.0, DIM);
}

/// The rate curves: degrees a second against stick for the three axes, `focus` the axis whose
/// row is under the pointer. The pad's sticks ride their curves.
fn hero_rates(d: &mut dyn DrawContext, host: &dyn ClientHost, live: &Live, card: [f32; 4], focus: Option<crate::drone::ControlAxis>) {
    use crate::drone::ControlAxis;
    let [x, y, w, h] = card;
    let rates = |axis: ControlAxis| {
        let a = axis.key();
        let v = |k: &str| model::index(&format!("rate_{a}_{k}")).map_or(0.0, |i| range(host, i));
        (v("rc"), v("super"), v("expo"))
    };
    let colors = [WHITE, AMBER, HINT];
    let maxes: Vec<f32> = model::RATE_AXES.iter().map(|a| {
        let (rc, s, e) = rates(*a);
        model::rate_at(rc, s, e, 1.0)
    }).collect();
    let top_rate = maxes.iter().copied().fold(0.0, f32::max);
    let scale = ((top_rate / 200.0).ceil() * 200.0).max(400.0);

    let gx = x + 30.0;
    let gy = y + 8.0;
    let gw = w - 38.0;
    let gh = h - 30.0;
    // Grid: every 200 degrees a second, and quarter sticks.
    let mut level = 0.0;
    while level <= scale + 1.0 {
        let ly = gy + gh - gh * level / scale;
        d.fill([gx, ly, gw, 1.0], if level == 0.0 { 0x50FF_FFFF } else { 0x1CFF_FFFF });
        osd::text_right(d, &format!("{level:.0}"), gx - 4.0, ly - 4.0, FAINT);
        level += 200.0;
    }
    for q in 1..=4 {
        let lx = gx + gw * q as f32 / 4.0;
        d.fill([lx, gy, 1.0, gh], 0x14FF_FFFF);
    }
    let sticks = live.sticks.map(|s| [s.roll, s.pitch, s.yaw]);
    for (n, axis) in model::RATE_AXES.iter().enumerate() {
        let (rc, s, e) = rates(*axis);
        let lit = focus.is_none_or(|f| f == *axis);
        let color = osd::fade(colors[n], if lit { 1.0 } else { 0.3 });
        let width = if focus == Some(*axis) { 2.0 } else { 1.0 };
        let point = |stick: f32| [gx + gw * stick, gy + gh - gh * (model::rate_at(rc, s, e, stick) / scale).min(1.05)];
        let mut prev = point(0.0);
        for k in 1..=32 {
            let p = point(k as f32 / 32.0);
            osd::line(d, prev, p, width, color);
            prev = p;
        }
        if let Some(st) = sticks {
            let v = st[n].abs().min(1.0);
            let p = point(v);
            d.fill([p[0] - 2.0, p[1] - 2.0, 4.0, 4.0], colors[n]);
        }
    }
    // The legend: each axis's top rate.
    let ly = y + h - 12.0;
    let cell = w / 3.0;
    for (n, axis) in model::RATE_AXES.iter().enumerate() {
        let lit = focus.is_none_or(|f| f == *axis);
        let cx = x + cell * (n as f32 + 0.5);
        let label = format!("{} {:.0}°/s", model::rate_axis_name(*axis).to_uppercase(), maxes[n]);
        let lw = d.text_width(&label) + 7.0;
        d.fill([cx - lw / 2.0, ly + 3.0, 4.0, 2.0], colors[n]);
        osd::text(d, &label, cx - lw / 2.0 + 7.0, ly, if lit { WHITE } else { FAINT });
    }
}

/// The sticks, live, Mode 2: throttle and yaw on the left, pitch and roll on the right, with
/// the deadzone where a stick does nothing.
fn hero_sticks(d: &mut dyn DrawContext, host: &dyn ClientHost, live: &Live, card: [f32; 4]) {
    let [x, y, w, h] = card;
    let dz = match key_value(host, "deadzone") {
        Value::Range(v) => v as f32,
        _ => 0.05,
    };
    let size = (h - 38.0).min(64.0);
    let gap = (w - 2.0 * size) / 3.0;
    let sticks = live.sticks;
    let boxes = [
        ("THROTTLE · YAW", sticks.map_or([0.0, 0.0], |s| [s.yaw, s.throttle])),
        ("PITCH · ROLL", sticks.map_or([0.0, 0.0], |s| [s.roll, s.pitch])),
    ];
    for (n, (label, [sx, sy])) in boxes.iter().enumerate() {
        let bx = x + gap + n as f32 * (size + gap);
        let by = y + 6.0;
        let half = size / 2.0;
        let (cx, cy) = (bx + half, by + half);
        d.fill([bx, by, size, size], 0x50000000);
        for r in [[bx, by, size, 1.0], [bx, by + size - 1.0, size, 1.0], [bx, by, 1.0, size], [bx + size - 1.0, by, 1.0, size]] {
            d.fill(r, 0x40FF_FFFF);
        }
        // The deadzone: a band along each axis where that axis reads nothing.
        let band = (dz * half).max(0.5);
        d.fill([cx - band, by + 1.0, band * 2.0, size - 2.0], 0x30FF_C040);
        d.fill([bx + 1.0, cy - band, size - 2.0, band * 2.0], 0x30FF_C040);
        d.fill([cx - 0.5, by + 3.0, 1.0, size - 6.0], 0x30FF_FFFF);
        d.fill([bx + 3.0, cy - 0.5, size - 6.0, 1.0], 0x30FF_FFFF);
        let (px, py) = (cx + sx.clamp(-1.0, 1.0) * (half - 3.0), cy - sy.clamp(-1.0, 1.0) * (half - 3.0));
        let dead = sx.abs() < dz && sy.abs() < dz;
        d.fill([px - 2.5, py - 2.5, 5.0, 5.0], INK);
        d.fill([px - 2.0, py - 2.0, 4.0, 4.0], if dead { AMBER } else { WHITE });
        osd::text_centre(d, label, cx, by + size + 4.0, FAINT);
    }
    if sticks.is_none() {
        osd::text_centre(d, "Move a gamepad stick to try the deadzone", x + w / 2.0, y + h - 11.0, 0x70FF_FFFF);
    } else {
        osd::text_centre(d, "Inside the amber band a stick reads nothing", x + w / 2.0, y + h - 11.0, 0x70FF_FFFF);
        osd::text_right(d, "LIVE", x + w - 5.0, y + 4.0, AMBER);
    }
}

/// The video link against distance: full, fading, LOW SIGNAL, TURN BACK and lost, with the
/// drone riding along it (yours while it is out, else a demonstration flight), and the badge
/// the OSD would show there.
fn hero_link(d: &mut dyn DrawContext, host: &dyn ClientHost, live: &Live, card: [f32; 4], time: f32) {
    use model::{distance_at_quality, link_quality, LINK_CRITICAL, LINK_LOW};
    let [x, y, w, h] = card;
    let cx = x + w / 2.0;
    let range = match key_value(host, "range") {
        Value::Range(v) => v,
        _ => 160.0,
    };
    if model::range_unlimited(range) {
        // No range: the badge the OSD would show is always the calm one.
        let words = if live.drone_out { format!("{:.0}m from you", live.distance) } else { "The link never fades".to_string() };
        let title = "NO RANGE LIMIT";
        let tw = d.text_width(title) + 18.0;
        let pw = tw.max(d.text_width(&words)) + 16.0;
        let (px, by, ph) = (cx - pw / 2.0, y + 6.0, 27.0);
        d.fill([px, by, pw, ph], GLASS);
        d.fill([px, by, pw, 1.0], AMBER);
        d.fill([px, by + ph - 2.0, pw, 2.0], AMBER);
        osd::signal_bars(d, cx - tw / 2.0, by + 12.0, 100.0, WHITE);
        osd::text(d, title, cx - tw / 2.0 + 18.0, by + 4.0, WHITE);
        osd::text_centre(d, &words, cx, by + 14.0, DIM);
        let note = "Fly as far as you like: the picture never breaks up and the drone never fails safe. Drag the range left for a real video link.";
        for (n, l) in clipped_lines(d, note, w - 16.0, 3).iter().enumerate() {
            osd::text_centre(d, l, cx, y + 44.0 + n as f32 * 10.0, FAINT);
        }
        osd::text_right(d, if live.drone_out { "YOUR DRONE" } else { "PREVIEW" }, x + w - 5.0, y + 4.0, 0x60FF_FFFF);
        return;
    }
    // The track runs to a little past the range, where the link is lost.
    let reach = range * 1.1;
    let (demo, distance) = if live.drone_out {
        (false, live.distance)
    } else {
        // From halfway out to a little past the range and back, over ten seconds: where the
        // link changes.
        let t = (time / 10.0).fract();
        let tri = if t < 0.5 { t * 2.0 } else { 2.0 - t * 2.0 };
        (true, range * (0.5 + 0.58 * f64::from(ease_in_out(tri))))
    };
    let link = link_quality(distance, range);

    // The badge, as the OSD shows it there.
    let by = y + 6.0;
    if link > 0.0 && link < LINK_LOW {
        osd::low_signal_badge(d, link, distance, range, time, cx, by);
    } else {
        let (title, words, color) = if link <= 0.0 {
            ("FAILSAFE", "Link lost: the drone disarms".to_string(), WARN)
        } else {
            ("LINK GOOD", format!("{:.0}m from you", distance), WHITE)
        };
        let tw = d.text_width(title) + 18.0;
        let pw = tw.max(d.text_width(&words)) + 16.0;
        let ph = 27.0;
        let px = cx - pw / 2.0;
        d.fill([px, by, pw, ph], GLASS);
        d.fill([px, by, pw, 1.0], color);
        d.fill([px, by + ph - 2.0, pw, 2.0], 0x40FF_FFFF);
        d.fill([px, by + ph - 2.0, pw * (link / 100.0).clamp(0.0, 1.0), 2.0], color);
        let row_x = cx - tw / 2.0;
        osd::signal_bars(d, row_x, by + 12.0, link, color);
        osd::text(d, title, row_x + 18.0, by + 4.0, color);
        osd::text_centre(d, &words, cx, by + 14.0, DIM);
    }

    // The track: you on the left, the range on the right.
    let tx0 = x + 10.0;
    let tx1 = x + w - 10.0;
    let ty = y + 50.0;
    let at = |dist: f64| tx0 + (tx1 - tx0) * (dist / reach) as f32;
    let low = distance_at_quality(LINK_LOW, range);
    let critical = distance_at_quality(LINK_CRITICAL, range);
    let segments = 48;
    for k in 0..segments {
        let d0 = reach * k as f64 / segments as f64;
        let d1 = reach * (k + 1) as f64 / segments as f64;
        let mid = (d0 + d1) / 2.0;
        let q = link_quality(mid, range);
        let color = if q <= 0.0 {
            0x50FF_5050
        } else if q < LINK_CRITICAL {
            WARN
        } else if q < LINK_LOW {
            AMBER
        } else {
            osd::fade(WHITE, 0.35 + 0.5 * q / 100.0)
        };
        d.fill([at(d0), ty, at(d1) - at(d0) + 0.5, 3.0], color);
    }
    // The marks' distances under the track, a second line down where two would touch.
    let you_end = tx0 - 2.0 + d.text_width("YOU") + 4.0;
    osd::text(d, "YOU", tx0 - 2.0, ty + 8.0, FAINT);
    let mut ends = [you_end, f32::MIN];
    for (dist, color) in [(low, AMBER), (critical, WARN), (range, WHITE)] {
        let mx = at(dist);
        let label = format!("{dist:.0}m");
        let lw = d.text_width(&label);
        let line = if mx - lw / 2.0 > ends[0] + 3.0 { 0 } else { 1 };
        d.fill([mx - 0.5, ty - 3.0, 1.0, 9.0 + line as f32 * 10.0], color);
        osd::text_centre(d, &label, mx, ty + 8.0 + line as f32 * 10.0, osd::fade(color, 0.9));
        ends[line] = mx + lw / 2.0;
    }
    // What the colours mean.
    let legend = [("LOW SIGNAL", AMBER), ("TURN BACK", WARN), ("LOST", 0xFFA0_3030)];
    let gap = 12.0;
    let total: f32 = legend.iter().map(|(s, _)| d.text_width(s) + 6.0).sum::<f32>() + gap * (legend.len() as f32 - 1.0);
    let mut lx = x + w / 2.0 - total / 2.0;
    let ly = y + h - 12.0;
    for (s, c) in legend {
        d.fill([lx, ly + 2.0, 3.0, 3.0], c);
        osd::text(d, s, lx + 6.0, ly, FAINT);
        lx += d.text_width(s) + 6.0 + gap;
    }
    // The drone on the track.
    let dx = at(distance.min(reach));
    d.quad([[dx, ty - 7.0], [dx + 3.0, ty - 4.0], [dx, ty - 1.0], [dx - 3.0, ty - 4.0]], WHITE);
    let tag = if demo { "PREVIEW" } else { "YOUR DRONE" };
    osd::text_right(d, tag, x + w - 5.0, y + 4.0, 0x60FF_FFFF);
}

fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The keys and pad buttons, for reference.
fn draw_keys(d: &mut dyn DrawContext, host: &dyn ClientHost, live: &Live, [x, y, w]: [f32; 3]) {
    let setup = settings::setup_key_name(host);
    let rows: [(&str, String, Vec<Hint>); 9] = [
        ("Drone out / land", "H".into(), vec![Hint::Pad(PadButton::L1), Hint::Text("+"), Hint::Pad(PadButton::R1)]),
        ("Arm / disarm", "R".into(), vec![Hint::Pad(PadButton::North)]),
        ("Reset to launch", "Backspace".into(), vec![Hint::Pad(PadButton::West)]),
        ("Switch view", "V".into(), vec![Hint::Pad(PadButton::Up)]),
        ("Flight mode", "M".into(), vec![Hint::Pad(PadButton::L1)]),
        ("Change drone", "[  ]".into(), vec![Hint::Pad(PadButton::Left), Hint::Pad(PadButton::Right)]),
        ("Throttle, yaw", "W S A D".into(), vec![Hint::Text("left stick")]),
        ("Pitch, roll", "Mouse".into(), vec![Hint::Text("right stick")]),
        ("This screen", setup, vec![Hint::Pad(PadButton::Select)]),
    ];
    let family = live.pad.unwrap_or(PadFamily::Xbox);
    let col_key = x + w * 0.48;
    let col_pad = x + w * 0.76;
    for (n, (action, key, pad)) in rows.iter().enumerate() {
        let ry = y + 4.0 + n as f32 * 11.0;
        osd::text(d, action, x, ry, DIM);
        osd::text_centre(d, key, col_key, ry, WHITE);
        let hw = osd::hint_width(d, pad, family);
        osd::hint_centre(d, pad, family, col_pad + hw / 2.0 - 8.0, ry, FAINT);
    }
}
