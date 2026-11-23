// Support module in the game's client_tick_post module. Before the mod's own client_tick_post
// hook runs, it reads what the mod API has no read for: the gamepad's sticks and buttons, and
// where the player stands and looks. The drone itself lives in the mod's crate.

use crate::game::{Game, Screen};
use sunrise_client::gamepad::{Button, GlyphStyle, MenuJob};
use sunrise_client::pad_glyph::Glyph;

/// The pad buttons the drone and the setup screen use, as bits for edge detection.
const BUTTONS: [(Button, u32); 10] = [
    (Button::LeftBumper, mod_fpvhorizon::PAD_L1),
    (Button::RightBumper, mod_fpvhorizon::PAD_R1),
    (Button::North, mod_fpvhorizon::PAD_ARM),
    (Button::West, mod_fpvhorizon::PAD_RESET),
    (Button::DPadUp, mod_fpvhorizon::PAD_VIEW),
    (Button::DPadLeft, mod_fpvhorizon::PAD_PREV),
    (Button::DPadRight, mod_fpvhorizon::PAD_NEXT),
    (Button::DPadDown, mod_fpvhorizon::PAD_DOWN),
    (Button::South, mod_fpvhorizon::PAD_SOUTH),
    (Button::Select, mod_fpvhorizon::PAD_SELECT),
];

pub fn tick(game: &mut Game<'_>) {
    if !game.native_window() {
        return;
    }
    let mut guard = mod_fpvhorizon::SHARED.lock().unwrap();
    let shared = &mut *guard;

    shared.player = game.player.as_ref().map(|p| mod_fpvhorizon::PlayerView {
        eye: [p.position.x, p.position.y + f64::from(p.dimensions.eye_height), p.position.z],
        feet: [p.position.x, p.position.y, p.position.z],
        yaw: game.yaw,
        alive: !p.health.dead_or_dying(),
    });
    shared.playing = game.title.is_none() && !game.pause && matches!(game.screen, Screen::Playing);

    // Mode 2: left stick throttle and yaw, right stick pitch and roll, while the pad is in use.
    let pad = game.pad.state;
    shared.sticks = game.pad.in_use.then(|| mod_fpvhorizon::drone::Sticks {
        roll: pad.right[0],
        pitch: pad.right[1],
        yaw: pad.left[0],
        throttle: pad.left[1],
    });
    let held = BUTTONS
        .iter()
        .filter(|(button, _)| pad.held(*button))
        .fold(0, |bits, (_, bit)| bits | bit);
    shared.pad_pressed |= held & !shared.pad_held;
    shared.pad_held = held;

    // The glyphs the player sees on Horizon's own pad hints, for the OSD's. Letter mode isn't
    // exposed on its own, so ask Horizon which button plays North: on a Nintendo pad going by
    // its letters that is the Y on the left, not the X on top.
    let glyphs = game.ui.options.pad_glyphs;
    shared.pad_family = match glyphs.family() {
        GlyphStyle::PlayStation => mod_fpvhorizon::PadFamily::PlayStation,
        GlyphStyle::Nintendo if Glyph::menu(MenuJob::Extra, glyphs).label() == "Y" => {
            mod_fpvhorizon::PadFamily::NintendoLetters
        }
        GlyphStyle::Nintendo => mod_fpvhorizon::PadFamily::Nintendo,
        GlyphStyle::Xbox | GlyphStyle::Auto => mod_fpvhorizon::PadFamily::Xbox,
    };
}
