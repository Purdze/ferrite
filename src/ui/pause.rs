use crate::renderer::pipelines::menu_overlay::MenuElement;
use super::common::{self, WHITE, BTN_NORMAL, BTN_HOVER};

const BTN_W: f32 = 200.0;
const BTN_H: f32 = 20.0;
const BTN_GAP: f32 = 4.0;
const FONT_SIZE: f32 = 8.0;

pub enum PauseAction {
    None,
    Resume,
    Disconnect,
    Quit,
}

pub fn build_pause_menu(
    elements: &mut Vec<MenuElement>,
    screen_w: f32,
    screen_h: f32,
    cursor: (f32, f32),
    clicked: bool,
    gs: f32,
) -> PauseAction {
    let mut action = PauseAction::None;
    let fs = FONT_SIZE * gs;

    common::push_overlay(elements, screen_w, screen_h, 0.47);

    let btn_w = BTN_W * gs;
    let btn_h = BTN_H * gs;
    let gap = BTN_GAP * gs;

    let title_y = screen_h / 2.0 - btn_h * 2.5 - gap * 2.0;
    elements.push(MenuElement::Text {
        x: screen_w / 2.0, y: title_y,
        text: "Game Menu".into(), scale: fs * 1.5,
        color: WHITE, centered: true,
    });

    let start_y = title_y + fs * 1.5 + 16.0 * gs;
    let btn_x = (screen_w - btn_w) / 2.0;

    let buttons: [(&str, PauseAction); 3] = [
        ("Back to Game", PauseAction::Resume),
        ("Disconnect", PauseAction::Disconnect),
        ("Quit Game", PauseAction::Quit),
    ];

    for (i, (label, btn_action)) in buttons.into_iter().enumerate() {
        let by = start_y + i as f32 * (btn_h + gap);
        let hovered = common::hit_test(cursor, [btn_x, by, btn_w, btn_h]);

        elements.push(MenuElement::Rect {
            x: btn_x, y: by, w: btn_w, h: btn_h,
            corner_radius: 2.0 * gs,
            color: if hovered { BTN_HOVER } else { BTN_NORMAL },
        });
        elements.push(MenuElement::Text {
            x: screen_w / 2.0, y: by + (btn_h - fs) / 2.0,
            text: label.into(), scale: fs,
            color: WHITE, centered: true,
        });

        if clicked && hovered {
            action = btn_action;
        }
    }

    action
}
