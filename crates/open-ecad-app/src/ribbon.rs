//! Cinta (ribbon) al estilo de los CAD de escritorio: pestañas por menú, grupos con nombre,
//! un botón grande por grupo y el resto en columnas de botones pequeños. Se construye a
//! partir del registro de comandos, igual que antes los menús.

use egui::{
    Align2, Color32, CursorIcon, FontFamily, FontId, Popup, Rect, Response, Sense, Stroke, Ui,
    Vec2, pos2, vec2,
};

use crate::commands::{self, COMMANDS, Command, CommandId, Menu};
use crate::theme;

const LARGE_HEIGHT: f32 = 66.0;
const SMALL_HEIGHT: f32 = 22.0;
const SMALL_ROWS: usize = 3;
const GROUP_LABEL_HEIGHT: f32 = 16.0;

/// Una entrada de un grupo: un comando suelto o un desplegable (submenú del registro).
enum Entry {
    Command(&'static Command),
    Dropdown(&'static str, Vec<&'static Command>),
}

impl Entry {
    fn first(&self) -> &'static Command {
        match self {
            Entry::Command(c) => c,
            Entry::Dropdown(_, items) => items[0],
        }
    }
}

pub struct RibbonState {
    pub tab: Menu,
    pub collapsed: bool,
}

impl Default for RibbonState {
    fn default() -> Self {
        Self {
            tab: Menu::Editar,
            collapsed: false,
        }
    }
}

/// Contexto que la cinta necesita para pintar estados (herramienta activa, conmutadores).
pub struct Ctx<'a> {
    pub icons: bool,
    pub is_active: &'a dyn Fn(CommandId) -> bool,
}

/// Pestañas de la cinta. «Archivo» se muestra aparte, como desplegable destacado.
pub const TABS: [Menu; 7] = [
    Menu::Editar,
    Menu::Ver,
    Menu::Insertar,
    Menu::Proyecto,
    Menu::Herramientas,
    Menu::Datos,
    Menu::Ayuda,
];

pub fn tabs_row(ui: &mut Ui, state: &mut RibbonState, cx: &Ctx, clicked: &mut Vec<CommandId>) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        // Barra de acceso rápido.
        for id in [CommandId::Guardar, CommandId::Deshacer, CommandId::Rehacer] {
            let c = commands::get(id);
            if icon_button(ui, cx, c).clicked() {
                clicked.push(id);
            }
        }
        ui.add_space(8.0);

        let file = ui.add(
            egui::Button::new(egui::RichText::new("  Archivo  ").color(Color32::WHITE))
                .fill(theme::ACCENT)
                .corner_radius(0.0),
        );
        Popup::menu(&file).show(|ui| {
            ui.set_min_width(220.0);
            menu_list(ui, Menu::Archivo, clicked);
        });

        for menu in TABS {
            let selected = state.tab == menu && !state.collapsed;
            let r = tab_button(ui, menu.label(), selected);
            if r.clicked() {
                if state.tab == menu {
                    state.collapsed = !state.collapsed;
                } else {
                    state.tab = menu;
                    state.collapsed = false;
                }
            }
            if r.double_clicked() {
                state.collapsed = !state.collapsed;
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let r = ui.add(
                egui::Button::new(egui::RichText::new("Buscar comando…   Ctrl+K").weak())
                    .min_size(vec2(220.0, 0.0)),
            );
            if r.clicked() {
                clicked.push(CommandId::PaletaComandos);
            }
        });
    });
}

pub fn body(ui: &mut Ui, state: &RibbonState, cx: &Ctx, clicked: &mut Vec<CommandId>) {
    if state.collapsed {
        return;
    }
    let groups = groups(state.tab);
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing = vec2(2.0, 0.0);
        for (i, (group, entries)) in groups.iter().enumerate() {
            if i > 0 {
                let x = ui.cursor().left() + 3.0;
                let top = ui.cursor().top() + 4.0;
                ui.painter().line_segment(
                    [
                        pos2(x, top),
                        pos2(x, top + LARGE_HEIGHT + GROUP_LABEL_HEIGHT - 6.0),
                    ],
                    Stroke::new(1.0, theme::SEPARATOR),
                );
                ui.add_space(8.0);
            }
            group_ui(ui, state.tab, *group, entries, cx, clicked);
        }
    });
}

fn groups(menu: Menu) -> Vec<(u8, Vec<Entry>)> {
    let mut out: Vec<(u8, Vec<Entry>)> = Vec::new();
    for c in COMMANDS.iter().filter(|c| c.menu == menu) {
        if out.last().is_none_or(|(g, _)| *g != c.group) {
            out.push((c.group, Vec::new()));
        }
        let entries = &mut out.last_mut().expect("grupo recién creado").1;
        match c.submenu {
            None => entries.push(Entry::Command(c)),
            Some(sub) => match entries
                .iter_mut()
                .find(|e| matches!(e, Entry::Dropdown(s, _) if *s == sub))
            {
                Some(Entry::Dropdown(_, items)) => items.push(c),
                _ => entries.push(Entry::Dropdown(sub, vec![c])),
            },
        }
    }
    out
}

fn group_ui(
    ui: &mut Ui,
    menu: Menu,
    group: u8,
    entries: &[Entry],
    cx: &Ctx,
    clicked: &mut Vec<CommandId>,
) {
    ui.vertical(|ui| {
        let top = ui.cursor().top();
        ui.horizontal_top(|ui| {
            let (large, small) = entries.split_first().expect("grupo no vacío");
            entry_ui(ui, large, true, cx, clicked);
            for column in small.chunks(SMALL_ROWS) {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for e in column {
                        entry_ui(ui, e, false, cx, clicked);
                    }
                });
            }
        });
        let width = ui.min_rect().width().max(40.0);
        let label_rect = Rect::from_min_size(
            pos2(ui.min_rect().left(), top + LARGE_HEIGHT),
            vec2(width, GROUP_LABEL_HEIGHT),
        );
        ui.painter().text(
            label_rect.center(),
            Align2::CENTER_CENTER,
            commands::group_name(menu, group),
            FontId::proportional(11.0),
            theme::TEXT_WEAK,
        );
        ui.allocate_rect(label_rect, Sense::hover());
    });
}

fn entry_ui(ui: &mut Ui, entry: &Entry, large: bool, cx: &Ctx, clicked: &mut Vec<CommandId>) {
    match entry {
        Entry::Command(c) => {
            let r = command_button(ui, cx, c, large, None);
            if r.clicked() {
                clicked.push(c.id);
            }
        }
        Entry::Dropdown(name, items) => {
            let any_active = items.iter().any(|c| (cx.is_active)(c.id));
            let fake = Command {
                label: name,
                implemented: items.iter().any(|c| c.implemented),
                ..*entry.first()
            };
            let r = command_button(ui, cx, &fake, large, Some(any_active));
            Popup::menu(&r).show(|ui| {
                for c in items {
                    if menu_item(ui, c) {
                        clicked.push(c.id);
                    }
                }
            });
        }
    }
}

/// Botón de la cinta. `dropdown` = `Some(activo)` si abre un desplegable.
fn command_button(
    ui: &mut Ui,
    cx: &Ctx,
    c: &Command,
    large: bool,
    dropdown: Option<bool>,
) -> Response {
    let label = match dropdown {
        Some(_) => format!("{} ▾", c.label.trim_end_matches('…')),
        None => c.label.trim_end_matches('…').to_string(),
    };
    let active = dropdown.unwrap_or_else(|| (cx.is_active)(c.id));
    let icon = if cx.icons { commands::icon(c.id) } else { None };
    let text_font = FontId::proportional(12.5);
    let text_w = ui.fonts_mut(|f| {
        f.layout_no_wrap(label.clone(), text_font.clone(), Color32::WHITE)
            .size()
            .x
    });

    let size = if large {
        vec2((text_w + 12.0).max(52.0), LARGE_HEIGHT)
    } else {
        vec2(
            text_w + if icon.is_some() { 30.0 } else { 10.0 },
            SMALL_HEIGHT,
        )
    };
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    let response = match &c.shortcut {
        Some(sc) => {
            response.on_hover_text(format!("{}  ({})", c.label, ui.ctx().format_shortcut(sc)))
        }
        None => response.on_hover_text(c.label),
    };

    let painter = ui.painter();
    if active {
        painter.rect_filled(rect, 2.0, theme::ACTIVE_BG);
        painter.rect_stroke(
            rect,
            2.0,
            Stroke::new(1.0, theme::ACCENT),
            egui::StrokeKind::Inside,
        );
    } else if response.hovered() {
        painter.rect_filled(rect, 2.0, theme::HOVER_BG);
    }
    let color = if c.implemented {
        theme::TEXT
    } else {
        theme::TEXT_WEAK
    };
    let icon_color = if c.implemented {
        theme::ICON
    } else {
        theme::TEXT_WEAK
    };
    let icon_font = |size: f32| FontId::new(size, FontFamily::Name(theme::ICON_FAMILY.into()));

    if large {
        if let Some(icon) = icon {
            painter.text(
                rect.center_top() + vec2(0.0, 22.0),
                Align2::CENTER_CENTER,
                icon,
                icon_font(26.0),
                icon_color,
            );
        }
        painter.text(
            rect.center_bottom() - vec2(0.0, 12.0),
            Align2::CENTER_CENTER,
            label,
            text_font,
            color,
        );
    } else {
        let mut x = rect.left() + 5.0;
        if let Some(icon) = icon {
            painter.text(
                pos2(x + 8.0, rect.center().y),
                Align2::CENTER_CENTER,
                icon,
                icon_font(15.0),
                icon_color,
            );
            x += 22.0;
        }
        painter.text(
            pos2(x, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            text_font,
            color,
        );
    }
    response
}

/// Botón solo-icono (barra de acceso rápido).
fn icon_button(ui: &mut Ui, cx: &Ctx, c: &Command) -> Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(24.0), Sense::click());
    if response.hovered() {
        ui.painter().rect_filled(rect, 2.0, theme::HOVER_BG);
    }
    let color = if c.implemented {
        theme::ICON
    } else {
        theme::TEXT_WEAK
    };
    match commands::icon(c.id).filter(|_| cx.icons) {
        Some(icon) => {
            let font = FontId::new(15.0, FontFamily::Name(theme::ICON_FAMILY.into()));
            ui.painter()
                .text(rect.center(), Align2::CENTER_CENTER, icon, font, color);
        }
        None => {
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                &c.label[..1],
                FontId::proportional(13.0),
                color,
            );
        }
    }
    response.on_hover_text(c.label)
}

fn tab_button(ui: &mut Ui, label: &str, selected: bool) -> Response {
    let font = FontId::proportional(13.5);
    let w = ui.fonts_mut(|f| {
        f.layout_no_wrap(label.to_string(), font.clone(), Color32::WHITE)
            .size()
            .x
    });
    let (rect, response) = ui.allocate_exact_size(vec2(w + 22.0, 26.0), Sense::click());
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 0.0, theme::RIBBON_BG);
        painter.line_segment(
            [
                rect.left_bottom() + vec2(6.0, -1.5),
                rect.right_bottom() + vec2(-6.0, -1.5),
            ],
            Stroke::new(2.0, theme::ACCENT),
        );
    } else if response.hovered() {
        painter.rect_filled(rect, 0.0, theme::HOVER_BG);
    }
    let color = if selected {
        theme::TEXT_STRONG
    } else {
        theme::TEXT
    };
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, color);
    response
}

/// Lista de comandos de un menú con separadores por grupo (desplegable «Archivo»).
fn menu_list(ui: &mut Ui, menu: Menu, clicked: &mut Vec<CommandId>) {
    let mut last = None;
    for c in COMMANDS.iter().filter(|c| c.menu == menu) {
        if last.is_some_and(|g| g != c.group) {
            ui.separator();
        }
        last = Some(c.group);
        if menu_item(ui, c) {
            clicked.push(c.id);
        }
    }
}

fn menu_item(ui: &mut Ui, c: &Command) -> bool {
    let mut text = egui::RichText::new(c.label);
    if !c.implemented {
        text = text.weak();
    }
    let mut button = egui::Button::new(text);
    if let Some(sc) = &c.shortcut {
        button = button.shortcut_text(ui.ctx().format_shortcut(sc));
    }
    ui.add(button).clicked()
}
