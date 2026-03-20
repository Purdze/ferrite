use std::collections::HashMap;

use azalea_registry::builtin::ItemKind;

use super::common::{self, WHITE};
use crate::player::inventory::{item_resource_name, Inventory};
use crate::renderer::pipelines::menu_overlay::{MenuElement, SpriteId};

const IMAGE_W: f32 = 195.0;
const IMAGE_H: f32 = 136.0;
const COLS: usize = 9;
const ROWS: usize = 5;
const SLOT_STRIDE: f32 = 18.0;
const SLOT_SIZE: f32 = 16.0;
const GRID_X: f32 = 9.0;
const GRID_Y: f32 = 18.0;
const HOTBAR_Y: f32 = 112.0;
const SCROLLER_X: f32 = 175.0;
const SCROLLER_Y: f32 = 18.0;
const SCROLLER_TRACK_H: f32 = 112.0;
const SCROLLER_W: f32 = 12.0;
const SCROLLER_H: f32 = 15.0;
const LABEL_COLOR: [f32; 4] = [0.25, 0.25, 0.25, 1.0];
const COUNT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const HIGHLIGHT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 0.5];
const TAB_W: f32 = 26.0;
const TAB_H: f32 = 32.0;

const TOP_SEL: [SpriteId; 7] = [
    SpriteId::CreativeTabTopSel1,
    SpriteId::CreativeTabTopSel2,
    SpriteId::CreativeTabTopSel3,
    SpriteId::CreativeTabTopSel4,
    SpriteId::CreativeTabTopSel5,
    SpriteId::CreativeTabTopSel6,
    SpriteId::CreativeTabTopSel7,
];
const TOP_UNSEL: [SpriteId; 7] = [
    SpriteId::CreativeTabTopUnsel1,
    SpriteId::CreativeTabTopUnsel2,
    SpriteId::CreativeTabTopUnsel3,
    SpriteId::CreativeTabTopUnsel4,
    SpriteId::CreativeTabTopUnsel5,
    SpriteId::CreativeTabTopUnsel6,
    SpriteId::CreativeTabTopUnsel7,
];

const TAB_DATA: &str = include_str!("creative_tabs.json");

const TAB_NAMES: &[&str] = &[
    "building_blocks",
    "colored_blocks",
    "natural_blocks",
    "functional_blocks",
    "redstone_blocks",
    "tools_and_utilities",
    "combat",
    "food_and_drinks",
    "ingredients",
    "spawn_eggs",
];

const TAB_LABELS: &[&str] = &[
    "Building", "Colors", "Nature", "Function", "Redstone", "Tools", "Combat", "Food", "Ingredi.",
    "Eggs",
];

pub struct CreativeState {
    pub scroll_offset: f32,
    pub tab: usize,
    items_cache: Vec<Vec<ItemKind>>,
    tab_count: usize,
}

impl CreativeState {
    pub fn new() -> Self {
        let item_map = build_item_kind_map();
        let tab_json: HashMap<String, Vec<String>> =
            serde_json::from_str(TAB_DATA).unwrap_or_default();

        let mut items_cache = Vec::new();
        for &tab_key in TAB_NAMES {
            let tab_names = tab_json.get(tab_key).cloned().unwrap_or_default();
            let total = tab_names.len();
            let items: Vec<ItemKind> = tab_names
                .iter()
                .filter_map(|n| item_map.get(n.as_str()).copied())
                .collect();
            let resolved = items.len();
            if resolved < total {
                log::debug!(
                    "Creative tab {tab_key}: {resolved}/{total} items resolved ({} missing)",
                    total - resolved
                );
            }
            items_cache.push(items);
        }

        let tab_count = items_cache.len();
        Self {
            scroll_offset: 0.0,
            tab: 0,
            items_cache,
            tab_count,
        }
    }

    fn items(&self) -> &[ItemKind] {
        &self.items_cache[self.tab.min(self.items_cache.len() - 1)]
    }

    fn max_scroll(&self) -> f32 {
        let total_rows = self.items().len().div_ceil(COLS);
        (total_rows as f32 - ROWS as f32).max(0.0)
    }
}

pub struct CreativeResult {
    pub picked: Option<(u8, ItemKind)>,
    pub close: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn build_creative(
    elements: &mut Vec<MenuElement>,
    state: &mut CreativeState,
    screen_w: f32,
    screen_h: f32,
    cursor: (f32, f32),
    clicked: bool,
    scroll_delta: f32,
    selected_slot: u8,
    gs: f32,
    inventory: &Inventory,
) -> CreativeResult {
    let scale = gs
        .min(screen_w / IMAGE_W)
        .min(screen_h / (IMAGE_H + TAB_H * 2.0));
    let inv_w = IMAGE_W * scale;
    let inv_h = IMAGE_H * scale;
    let ox = (screen_w - inv_w) / 2.0;
    let oy = (screen_h - inv_h) / 2.0;

    let mut picked = None;
    let mut close = false;

    common::push_overlay(elements, screen_w, screen_h, 0.5);

    // Tabs (top row) - vanilla positions: x = 27 * column, y = topPos - 28
    for i in 0..state.tab_count.min(7) {
        let tx = ox + (27.0 * i as f32) * scale;
        let ty = oy - 28.0 * scale;
        let tw = TAB_W * scale;
        let th = TAB_H * scale;
        let tab_rect = [tx, ty, tw, th];
        let hovered = common::hit_test(cursor, tab_rect);

        let sprite = if i == state.tab {
            TOP_SEL[i]
        } else {
            TOP_UNSEL[i]
        };
        elements.push(MenuElement::Image {
            x: tx,
            y: ty,
            w: tw,
            h: th,
            sprite,
            tint: WHITE,
        });

        if clicked && hovered {
            state.tab = i;
            state.scroll_offset = 0.0;
        }
    }

    // Bottom row tabs (tabs 7+)
    for i in 7..state.tab_count.min(14) {
        let col = i - 7;
        let tx = ox + (27.0 * col as f32) * scale;
        let ty = oy + inv_h - 4.0 * scale;
        let tw = TAB_W * scale;
        let th = TAB_H * scale;
        let tab_rect = [tx, ty, tw, th];
        let hovered = common::hit_test(cursor, tab_rect);

        let sprite = if i == state.tab {
            SpriteId::CreativeTabBotSel1
        } else {
            SpriteId::CreativeTabBotUnsel1
        };
        elements.push(MenuElement::Image {
            x: tx,
            y: ty,
            w: tw,
            h: th,
            sprite,
            tint: WHITE,
        });

        if clicked && hovered {
            state.tab = i;
            state.scroll_offset = 0.0;
        }
    }

    // Background
    elements.push(MenuElement::Image {
        x: ox,
        y: oy,
        w: inv_w,
        h: inv_h,
        sprite: SpriteId::CreativeTabItems,
        tint: WHITE,
    });

    // Tab label
    let fs = 6.0 * scale;
    elements.push(MenuElement::Text {
        x: ox + 8.0 * scale,
        y: oy + 6.0 * scale,
        text: TAB_LABELS[state.tab.min(TAB_LABELS.len() - 1)].into(),
        scale: fs,
        color: LABEL_COLOR,
        centered: false,
    });

    // Grid items
    let items = state.items().to_vec();
    let max_scroll = state.max_scroll();

    let grid_px_x = ox + GRID_X * scale;
    let grid_px_y = oy + GRID_Y * scale;
    let grid_w = COLS as f32 * SLOT_STRIDE * scale;
    let grid_h = ROWS as f32 * SLOT_STRIDE * scale;
    let grid_rect = [grid_px_x, grid_px_y, grid_w, grid_h];
    let in_grid = common::hit_test(cursor, grid_rect);

    if in_grid && scroll_delta.abs() > 0.01 {
        state.scroll_offset = (state.scroll_offset - scroll_delta).clamp(0.0, max_scroll);
    }

    let row_offset = state.scroll_offset.floor() as usize;
    let start_idx = row_offset * COLS;

    for vis_row in 0..ROWS {
        for col in 0..COLS {
            let idx = start_idx + vis_row * COLS + col;
            if idx >= items.len() {
                continue;
            }

            let sx = grid_px_x + col as f32 * SLOT_STRIDE * scale;
            let sy = grid_px_y + vis_row as f32 * SLOT_STRIDE * scale;
            let sz = SLOT_SIZE * scale;
            let slot_rect = [sx, sy, sz, sz];
            let hovered = common::hit_test(cursor, slot_rect) && in_grid;

            if hovered {
                elements.push(MenuElement::Rect {
                    x: sx,
                    y: sy,
                    w: sz,
                    h: sz,
                    corner_radius: 0.0,
                    color: HIGHLIGHT_COLOR,
                });
            }

            let name = item_resource_name(items[idx]);
            elements.push(MenuElement::BlockIcon {
                x: sx,
                y: sy,
                size: sz,
                block_name: name.clone(),
            });

            if clicked && hovered {
                picked = Some((selected_slot, items[idx]));
            }

            if hovered {
                let tooltip = name.replace('_', " ");
                let tip_fs = 6.0 * scale;
                let tip_w = tooltip.len() as f32 * 4.0 * scale + 8.0 * scale;
                let tip_x = (cursor.0 + 8.0 * scale).min(screen_w - tip_w);
                let tip_y = cursor.1 - tip_fs - 6.0 * scale;
                elements.push(MenuElement::Rect {
                    x: tip_x,
                    y: tip_y,
                    w: tip_w,
                    h: tip_fs + 4.0 * scale,
                    corner_radius: 2.0 * scale,
                    color: [0.1, 0.1, 0.1, 0.95],
                });
                elements.push(MenuElement::Text {
                    x: tip_x + 4.0 * scale,
                    y: tip_y + 2.0 * scale,
                    text: tooltip,
                    scale: tip_fs,
                    color: COUNT_COLOR,
                    centered: false,
                });
            }
        }
    }

    // Scrollbar - vanilla: scroller at (175, 18), track height 112
    let track_x = ox + SCROLLER_X * scale;
    let track_y = oy + SCROLLER_Y * scale;
    let track_h = SCROLLER_TRACK_H * scale;
    let thumb_w = SCROLLER_W * scale;
    let thumb_h = SCROLLER_H * scale;
    if max_scroll > 0.0 {
        let thumb_y = track_y + (track_h - thumb_h) * (state.scroll_offset / max_scroll);
        elements.push(MenuElement::Image {
            x: track_x,
            y: thumb_y,
            w: thumb_w,
            h: thumb_h,
            sprite: SpriteId::CreativeScroller,
            tint: WHITE,
        });
    } else {
        elements.push(MenuElement::Image {
            x: track_x,
            y: track_y,
            w: thumb_w,
            h: thumb_h,
            sprite: SpriteId::CreativeScrollerDisabled,
            tint: WHITE,
        });
    }

    // Hotbar (bottom of creative inventory)
    let hotbar = inventory.hotbar_slots();
    for (col, hotbar_item) in hotbar.iter().enumerate().take(9) {
        let sx = ox + GRID_X * scale + col as f32 * SLOT_STRIDE * scale;
        let sy = oy + HOTBAR_Y * scale;
        let sz = SLOT_SIZE * scale;
        let slot_rect = [sx, sy, sz, sz];
        let hovered = common::hit_test(cursor, slot_rect);

        if hovered {
            elements.push(MenuElement::Rect {
                x: sx,
                y: sy,
                w: sz,
                h: sz,
                corner_radius: 0.0,
                color: HIGHLIGHT_COLOR,
            });
        }

        if let azalea_inventory::ItemStack::Present(data) = hotbar_item {
            let name = item_resource_name(data.kind);
            elements.push(MenuElement::ItemIcon {
                x: sx,
                y: sy,
                w: sz,
                h: sz,
                item_name: name,
                tint: WHITE,
            });
            if data.count > 1 {
                elements.push(MenuElement::Text {
                    x: sx + sz - 1.0 * scale,
                    y: sy + sz - fs - 1.0 * scale,
                    text: data.count.to_string(),
                    scale: fs * 0.85,
                    color: COUNT_COLOR,
                    centered: false,
                });
            }
        }
    }

    // Close on click outside
    let outside = cursor.0 < ox
        || cursor.0 > ox + inv_w
        || cursor.1 < oy - TAB_H * scale
        || cursor.1 > oy + inv_h;
    if clicked && outside {
        close = true;
    }

    CreativeResult { picked, close }
}

fn build_item_kind_map() -> HashMap<String, ItemKind> {
    let mut map = HashMap::new();
    let mut id = 0u32;
    while let Ok(kind) = ItemKind::try_from(id) {
        let name = kind
            .to_string()
            .strip_prefix("minecraft:")
            .unwrap_or("air")
            .to_string();
        map.insert(name, kind);
        id += 1;
    }
    map
}
