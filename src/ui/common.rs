use crate::renderer::pipelines::menu_overlay::MenuElement;

pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const BTN_NORMAL: [f32; 4] = [0.3, 0.3, 0.3, 0.8];
pub const BTN_HOVER: [f32; 4] = [0.45, 0.45, 0.55, 0.9];

pub fn push_overlay(elements: &mut Vec<MenuElement>, screen_w: f32, screen_h: f32, alpha: f32) {
    elements.push(MenuElement::Rect {
        x: 0.0, y: 0.0, w: screen_w, h: screen_h,
        corner_radius: 0.0, color: [0.0, 0.0, 0.0, alpha],
    });
}

pub fn hit_test(cursor: (f32, f32), rect: [f32; 4]) -> bool {
    cursor.0 >= rect[0] && cursor.0 <= rect[0] + rect[2]
        && cursor.1 >= rect[1] && cursor.1 <= rect[1] + rect[3]
}

pub fn push_cursor_blink(
    elements: &mut Vec<MenuElement>,
    cursor_blink: &std::time::Instant,
    x: f32, y: f32,
    gs: f32, fs: f32,
    text_width: f32,
) {
    if cursor_blink.elapsed().as_millis() % 1000 < 500 {
        elements.push(MenuElement::Rect {
            x: x + text_width, y,
            w: 1.0 * gs, h: fs,
            corner_radius: 0.0, color: WHITE,
        });
    }
}
