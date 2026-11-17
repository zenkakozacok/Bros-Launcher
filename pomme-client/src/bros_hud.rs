//! Bros HUD, built in (it was a Fabric mod): armor left of the hotbar, the
//! held item's total count right of it, a smaller sidebar. Settings live in
//! `profiles/bros_hud.txt`; Options → Bros HUD toggles them.

use std::sync::RwLock;

use pomme_item::ItemStack;

use crate::renderer::pipelines::menu_overlay::MenuElement;
use crate::ui::common;

#[derive(Clone, Copy)]
pub struct Settings {
    pub enabled: bool,
    pub armor: bool,
    pub held_item: bool,
    pub small_sidebar: bool,
    pub join_hint: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: true,
            armor: true,
            held_item: true,
            small_sidebar: true,
            join_hint: true,
        }
    }
}

static SETTINGS: RwLock<Option<Settings>> = RwLock::new(None);

fn file() -> std::path::PathBuf {
    pomme_dirs::layout().profiles().join("bros_hud.txt")
}

pub fn get() -> Settings {
    if let Some(s) = *SETTINGS.read().unwrap_or_else(|e| e.into_inner()) {
        return s;
    }
    let mut s = Settings::default();
    if let Ok(text) = std::fs::read_to_string(file()) {
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let on = v.trim() == "true";
            match k.trim() {
                "enabled" => s.enabled = on,
                "armor" => s.armor = on,
                "held-item" => s.held_item = on,
                "small-sidebar" => s.small_sidebar = on,
                "join-hint" => s.join_hint = on,
                _ => {}
            }
        }
    }
    *SETTINGS.write().unwrap_or_else(|e| e.into_inner()) = Some(s);
    s
}

pub fn set(s: Settings) {
    *SETTINGS.write().unwrap_or_else(|e| e.into_inner()) = Some(s);
    let text = format!(
        "enabled={}\narmor={}\nheld-item={}\nsmall-sidebar={}\njoin-hint={}\n",
        s.enabled, s.armor, s.held_item, s.small_sidebar, s.join_hint
    );
    let path = file();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, text);
}

/// The scoreboard's GUI scale factor.
pub fn sidebar_scale() -> f32 {
    let s = get();
    if s.enabled && s.small_sidebar { 0.85 } else { 1.0 }
}

/// Draws the armor column and the held-item counter around the hotbar.
#[allow(clippy::too_many_arguments)]
pub fn build(
    elements: &mut Vec<MenuElement>,
    screen_w: f32,
    screen_h: f32,
    gs: f32,
    slots: &[ItemStack],
    armor: &[ItemStack],
    held: &ItemStack,
    main_hand_right: bool,
    offhand_shown: bool,
) {
    let s = get();
    if !s.enabled {
        return;
    }
    let cx = common::gui_half_width(screen_w, gs);
    let size = 16.0 * gs;
    let y = common::gui_from_bottom(screen_h, 19.0, gs);
    // The offhand slot sits on the off-hand side; keep clear of it.
    let left_shift = if offhand_shown && main_hand_right { 29.0 } else { 0.0 };
    let right_shift = if offhand_shown && !main_hand_right { 29.0 } else { 0.0 };

    if s.armor {
        let present: Vec<_> = armor
            .iter()
            .filter_map(|stack| match stack {
                ItemStack::Present(data) => Some(data),
                ItemStack::Empty => None,
            })
            .collect();
        let n = present.len() as f32;
        for (i, data) in present.into_iter().enumerate() {
            let x = (cx - (91.0 + 6.0 + left_shift + (n - i as f32) * 17.0) * gs).round();
            common::push_item_icon(elements, x, y, size, gs, data);
            common::push_item_decorations(elements, x, y, size, gs, data);
        }
    }

    if s.held_item
        && let ItemStack::Present(data) = held
    {
        let total: i32 = slots
            .iter()
            .filter_map(|stack| match stack {
                ItemStack::Present(d) if d.kind == data.kind => Some(d.count),
                _ => None,
            })
            .sum();
        let x = (cx + (91.0 + 6.0 + right_shift) * gs).round();
        common::push_item_icon(elements, x, y, size, gs, data);
        if total > 1 || data.count > 1 || is_stackable_tracked(data) {
            let text = total.to_string();
            let w = text.len() as f32 * 6.0 * gs;
            let color = if total <= 1 { [1.0, 0.33, 0.33, 1.0] } else { [1.0, 1.0, 1.0, 1.0] };
            elements.push(MenuElement::Text {
                x: x + size + gs - w,
                y: y + 9.0 * gs,
                text,
                scale: 8.0 * gs,
                color,
                centered: false,
            });
        }
    }
}

/// Items whose last copy is worth a red warning even at count 1.
fn is_stackable_tracked(data: &pomme_item::ItemStackData) -> bool {
    use pomme_registry::builtin::ItemKind as K;
    matches!(
        data.kind,
        K::TotemOfUndying | K::EndCrystal | K::EnderPearl | K::GoldenApple | K::EnchantedGoldenApple
    )
}

/// The chat line shown once per join while the HUD is on.
pub fn join_hint() -> Option<&'static str> {
    let s = get();
    (s.enabled && s.join_hint)
        .then_some("§6[Bros HUD]§7 включён. Выключить или настроить: §fНастройки → Bros HUD")
}
