//! Aspecto visual: tema oscuro sobrio, al estilo de los CAD de escritorio.
//!
//! En Windows se usan Segoe UI y la fuente de iconos del sistema, cargadas en tiempo de
//! ejecución (no se distribuyen con open-ecad). En otros sistemas, fuentes por defecto de
//! egui y cinta sin iconos.

use std::sync::Arc;

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Visuals};

pub const ICON_FAMILY: &str = "iconos";

pub const ACCENT: Color32 = Color32::from_rgb(0x2D, 0x7D, 0xD2);
pub const ACTIVE_BG: Color32 = Color32::from_rgb(0x26, 0x3F, 0x5C);
pub const HOVER_BG: Color32 = Color32::from_rgb(0x3A, 0x3D, 0x41);
pub const TITLE_BG: Color32 = Color32::from_rgb(0x1E, 0x1F, 0x22);
pub const RIBBON_BG: Color32 = Color32::from_rgb(0x2B, 0x2D, 0x30);
pub const PANEL_BG: Color32 = Color32::from_rgb(0x25, 0x27, 0x2A);
pub const FIELD_BG: Color32 = Color32::from_rgb(0x1B, 0x1C, 0x1E);
pub const CANVAS_BG: Color32 = Color32::from_rgb(0x3B, 0x3E, 0x43);
pub const SEPARATOR: Color32 = Color32::from_rgb(0x44, 0x47, 0x4C);
pub const TEXT: Color32 = Color32::from_rgb(0xD4, 0xD6, 0xD9);
pub const TEXT_STRONG: Color32 = Color32::WHITE;
pub const TEXT_WEAK: Color32 = Color32::from_rgb(0x82, 0x86, 0x8C);
pub const ICON: Color32 = Color32::from_rgb(0x8C, 0xC4, 0xFF);

/// Aplica fuentes y colores. Devuelve `true` si hay fuente de iconos.
pub fn apply(ctx: &egui::Context) -> bool {
    let icons = load_fonts(ctx);
    ctx.set_theme(egui::Theme::Dark);
    ctx.style_mut_of(egui::Theme::Dark, |style| {
        style.visuals = visuals();
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(6.0, 2.0);
        style.spacing.interact_size.y = 20.0;
    });
    icons
}

fn visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.panel_fill = PANEL_BG;
    v.window_fill = RIBBON_BG;
    v.extreme_bg_color = FIELD_BG;
    v.faint_bg_color = Color32::from_rgb(0x2C, 0x2E, 0x32);
    v.window_stroke = Stroke::new(1.0, SEPARATOR);
    v.window_corner_radius = CornerRadius::same(3);
    v.menu_corner_radius = CornerRadius::same(2);
    v.selection.bg_fill = ACTIVE_BG;
    v.selection.stroke = Stroke::new(1.0, ICON);
    v.hyperlink_color = ICON;
    for w in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(2);
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, SEPARATOR);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x33, 0x36, 0x3A);
    v.widgets.inactive.bg_fill = Color32::from_rgb(0x33, 0x36, 0x3A);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    v.widgets.hovered.weak_bg_fill = HOVER_BG;
    v.widgets.hovered.bg_fill = HOVER_BG;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x55, 0x59, 0x5F));
    v.widgets.active.weak_bg_fill = ACTIVE_BG;
    v.widgets.active.bg_fill = ACTIVE_BG;
    v
}

/// Estilo del dock a juego con el tema.
pub fn dock_style(egui_style: &egui::Style) -> egui_dock::Style {
    let mut s = egui_dock::Style::from_egui(egui_style);
    s.main_surface_border_stroke = Stroke::NONE;
    s.separator.width = 3.0;
    s.separator.color_idle = TITLE_BG;
    s.separator.color_hovered = ACCENT;
    s.separator.color_dragged = ACCENT;
    s.tab_bar.bg_fill = TITLE_BG;
    s.tab_bar.hline_color = SEPARATOR;
    s.tab_bar.height = 26.0;
    s.tab.tab_body.bg_fill = PANEL_BG;
    s.tab.tab_body.stroke = Stroke::NONE;
    s.tab.hline_below_active_tab_name = true;
    for t in [&mut s.tab.inactive, &mut s.tab.inactive_with_kb_focus] {
        t.bg_fill = TITLE_BG;
        t.text_color = TEXT_WEAK;
        t.outline_color = Color32::TRANSPARENT;
    }
    for t in [&mut s.tab.active, &mut s.tab.active_with_kb_focus] {
        t.bg_fill = PANEL_BG;
        t.text_color = TEXT;
        t.outline_color = Color32::TRANSPARENT;
    }
    for t in [&mut s.tab.focused, &mut s.tab.focused_with_kb_focus] {
        t.bg_fill = PANEL_BG;
        t.text_color = TEXT_STRONG;
        t.outline_color = ACCENT;
    }
    s.tab.hovered.bg_fill = HOVER_BG;
    s.tab.hovered.text_color = TEXT;
    for corner in [&mut s.tab.active.corner_radius, &mut s.tab.focused.corner_radius, &mut s.tab.inactive.corner_radius] {
        *corner = CornerRadius::ZERO;
    }
    s.overlay.selection_color = ACCENT.gamma_multiply(0.4);
    s
}

fn load_fonts(ctx: &egui::Context) -> bool {
    let mut fonts = FontDefinitions::default();
    let read = |name: &str| std::fs::read(system_fonts_dir()?.join(name)).ok();

    if let Some(bytes) = read("segoeui.ttf") {
        fonts.font_data.insert("segoe".into(), Arc::new(FontData::from_owned(bytes)));
        fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "segoe".into());
    }
    let icons = ["SegoeIcons.ttf", "segmdl2.ttf"].into_iter().find_map(read);
    let has_icons = icons.is_some();
    if let Some(bytes) = icons {
        fonts.font_data.insert(ICON_FAMILY.into(), Arc::new(FontData::from_owned(bytes)));
        fonts.families.insert(FontFamily::Name(ICON_FAMILY.into()), vec![ICON_FAMILY.into()]);
    }
    ctx.set_fonts(fonts);
    has_icons
}

fn system_fonts_dir() -> Option<std::path::PathBuf> {
    let windir = std::env::var_os("WINDIR")?;
    Some(std::path::PathBuf::from(windir).join("Fonts"))
}
