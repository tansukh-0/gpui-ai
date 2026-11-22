//! Input helpers shared by more than one component family.
//!
//! Keyboard activation is synthesised as a down/up pair against the window
//! rather than routed through a keymap, so a family activates whatever the
//! window currently focuses without standing up a binding for it.

use gpui::{KeyDownEvent, KeyUpEvent, Keystroke, VisualTestContext};

pub(crate) fn activate_key(cx: &mut VisualTestContext, key: &str) {
    let os = cx.read(|app| app.platform_info().os);
    let keystroke = Keystroke::parse_with_os(key, os).expect("test key should parse");
    cx.simulate_event(KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    cx.simulate_event(KeyUpEvent { keystroke });
}
