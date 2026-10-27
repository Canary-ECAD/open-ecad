//! Ventana principal: cinta, dock, barra de estado y paleta de comandos (MAPA-UI.md §4.1).

use std::collections::HashMap;

use egui::{Button, Key, Modifiers, RichText, Ui, ViewportCommand};
use egui_dock::{DockArea, DockState, NodeIndex};
use serde::{Deserialize, Serialize};

use crate::canvas::{CanvasOutput, PageView, ViewSettings};
use crate::commands::{self, COMMANDS, CommandId, Tool};
use crate::mock::{PageId, Project};
use crate::panels::{Action, PanelKind, PanelsState, Tab, Viewer};
use crate::ribbon::{self, RibbonState};
use crate::theme;

/// Cambiar la clave invalida disposiciones guardadas incompatibles.
const STORAGE_KEY: &str = "open-ecad/ventana/v1";

#[derive(Serialize, Deserialize)]
struct Persisted {
    dock: DockState<Tab>,
    project: Project,
}

pub struct OpenEcadApp {
    dock: DockState<Tab>,
    project: Project,
    views: HashMap<PageId, PageView>,
    settings: ViewSettings,
    panels: PanelsState,
    tool: Tool,
    status: String,
    canvas: Option<CanvasOutput>,
    palette: Option<Palette>,
    about_open: bool,
    ribbon: RibbonState,
    icons: bool,
}

#[derive(Default)]
struct Palette {
    query: String,
    selected: usize,
}

impl OpenEcadApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let icons = theme::apply(&cc.egui_ctx);
        let persisted: Option<Persisted> = cc.storage.and_then(|s| eframe::get_value(s, STORAGE_KEY));
        let (dock, project) = match persisted {
            Some(p) => (p.dock, p.project),
            None => {
                let project = Project::default();
                (default_dock(&project), project)
            }
        };
        let selected_page = dock.iter_all_tabs().find_map(|(_, t)| match t {
            Tab::Page(id) => Some(*id),
            Tab::Panel(_) => None,
        });
        let mut panels = PanelsState::default();
        panels.selected_page = selected_page;
        Self {
            dock,
            project,
            views: HashMap::new(),
            settings: ViewSettings::default(),
            panels,
            tool: Tool::Seleccion,
            status: "Listo".into(),
            canvas: None,
            palette: None,
            about_open: false,
            ribbon: RibbonState::default(),
            icons,
        }
    }

    fn execute(&mut self, ctx: &egui::Context, id: CommandId) {
        let command = commands::get(id);
        match id {
            CommandId::Salir => ctx.send_viewport_cmd(ViewportCommand::Close),
            CommandId::PaletaComandos => self.palette = Some(Palette::default()),
            CommandId::AcercaDe => self.about_open = true,
            CommandId::MostrarPanel(kind) => self.show_panel(kind),
            CommandId::RestablecerDisposicion => {
                self.dock = default_dock(&self.project);
                self.status = "Disposición restablecida".into();
            }
            CommandId::AjustarPagina => self.views.values_mut().for_each(PageView::request_fit),
            CommandId::Zoom100 => {
                let rect = ctx.content_rect();
                self.views.values_mut().for_each(|v| v.set_one_to_one(rect));
            }
            CommandId::Rejilla => toggle(&mut self.settings.grid_visible, "Rejilla", &mut self.status),
            CommandId::Snap => toggle(&mut self.settings.snap, "Snap", &mut self.status),
            CommandId::Orto => toggle(&mut self.settings.ortho, "Orto", &mut self.status),
            CommandId::Herramienta(tool) => {
                self.tool = tool;
                self.status = format!("Herramienta: {}", tool.label());
            }
            CommandId::NuevaPagina => {
                let structure = self
                    .panels
                    .selected_page
                    .and_then(|id| self.project.page(id))
                    .map_or_else(|| "=A1+CAB".to_string(), |p| p.structure.clone());
                let id = self.project.add_page(&structure);
                self.open_page(id);
            }
            _ => self.status = format!("«{}» aún no está implementado", command.label.trim_end_matches('…')),
        }
    }

    fn show_panel(&mut self, kind: PanelKind) {
        let tab = Tab::Panel(kind);
        if let Some(path) = self.dock.find_tab(&tab) {
            let _ = self.dock.set_active_tab(path);
        } else {
            self.dock.push_to_focused_leaf(tab);
        }
    }

    fn open_page(&mut self, id: PageId) {
        self.panels.selected_page = Some(id);
        let tab = Tab::Page(id);
        if let Some(path) = self.dock.find_tab(&tab) {
            let _ = self.dock.set_active_tab(path);
            return;
        }
        // Abrir junto a las demás páginas; si no hay ninguna, donde esté el foco.
        match self.dock.find_tab_from(|t| matches!(t, Tab::Page(_))) {
            Some(path) => {
                self.dock.set_focused_node_and_surface(path.node_path());
                self.dock.push_to_focused_leaf(tab);
            }
            None => self.dock.push_to_focused_leaf(tab),
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let typing = ctx.egui_wants_keyboard_input();
        let pressed: Vec<CommandId> = ctx.input_mut(|i| {
            COMMANDS
                .iter()
                .filter_map(|c| {
                    let sc = c.shortcut?;
                    // Los atajos sin Ctrl no deben robar teclas a un campo de texto.
                    let plain = sc.modifiers == Modifiers::NONE;
                    (!(typing && plain) && i.consume_shortcut(&sc)).then_some(c.id)
                })
                .collect()
        });
        for id in pressed {
            self.execute(ctx, id);
        }
    }

    fn status_bar(&mut self, ui: &mut Ui) -> Vec<CommandId> {
        let mut clicked = Vec::new();
        ui.horizontal(|ui| {
            ui.label(&self.status);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(c) = &self.canvas {
                    ui.label(format!("{:.0} %", c.zoom_percent));
                    ui.separator();
                }
                for (on, label, id) in [
                    (self.settings.ortho, "Orto", CommandId::Orto),
                    (self.settings.snap, "Snap", CommandId::Snap),
                    (self.settings.grid_visible, "Rejilla", CommandId::Rejilla),
                ] {
                    if ui.selectable_label(on, label).clicked() {
                        clicked.push(id);
                    }
                }
                ui.label(format!("Paso {:.2} mm", self.settings.grid_mm));
                ui.separator();
                let coords = match self.canvas.as_ref().and_then(|c| c.cursor_mm) {
                    Some(p) => format!("X {:>7.2}   Y {:>7.2} mm", p.x, p.y),
                    None => "X —   Y —".into(),
                };
                ui.monospace(coords);
            });
        });
        clicked
    }

    fn palette_ui(&mut self, ctx: &egui::Context) -> Option<CommandId> {
        let palette = self.palette.as_mut()?;
        let words: Vec<String> = commands::fold(&palette.query).split_whitespace().map(String::from).collect();
        let matches: Vec<&commands::Command> = COMMANDS
            .iter()
            .filter(|c| {
                let text = commands::search_text(c);
                words.iter().all(|w| text.contains(w.as_str()))
            })
            .collect();
        palette.selected = palette.selected.min(matches.len().saturating_sub(1));

        let (up, down, enter, escape) = ctx.input_mut(|i| {
            (
                i.consume_key(Modifiers::NONE, Key::ArrowUp),
                i.consume_key(Modifiers::NONE, Key::ArrowDown),
                i.consume_key(Modifiers::NONE, Key::Enter),
                i.consume_key(Modifiers::NONE, Key::Escape),
            )
        });
        if up {
            palette.selected = palette.selected.saturating_sub(1);
        }
        if down && palette.selected + 1 < matches.len() {
            palette.selected += 1;
        }

        let mut chosen = None;
        let response = egui::Modal::new(egui::Id::new("paleta")).show(ctx, |ui| {
            ui.set_width(520.0);
            let edit = ui.add(
                egui::TextEdit::singleline(&mut palette.query)
                    .hint_text("Escribe un comando…  (p. ej. «girar», «panel macros», «nueva página»)")
                    .desired_width(f32::INFINITY),
            );
            edit.request_focus();
            ui.separator();
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for (i, c) in matches.iter().enumerate() {
                    let path = match c.submenu {
                        Some(sub) => format!("{} › {} › ", c.menu.label(), sub),
                        None => format!("{} › ", c.menu.label()),
                    };
                    let mut text = RichText::new(format!("{path}{}", c.label));
                    if !c.implemented {
                        text = text.weak();
                    }
                    let mut button = Button::selectable(i == palette.selected, text);
                    if let Some(sc) = &c.shortcut {
                        button = button.shortcut_text(ctx.format_shortcut(sc));
                    }
                    let r = ui.add_sized([ui.available_width(), 22.0], button);
                    if i == palette.selected {
                        r.scroll_to_me(None);
                    }
                    if r.clicked() {
                        chosen = Some(c.id);
                    }
                }
                if matches.is_empty() {
                    ui.weak("Sin resultados");
                }
            });
            ui.separator();
            ui.weak("↑↓ elegir · Intro ejecutar · Esc cerrar · en gris: aún sin implementar");
        });
        if enter {
            chosen = matches.get(palette.selected).map(|c| c.id);
        }
        if chosen.is_some() || escape || response.should_close() {
            self.palette = None;
        }
        chosen
    }
}

impl eframe::App for OpenEcadApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.palette.is_none() {
            self.handle_shortcuts(&ctx);
        }

        let mut clicked = Vec::new();
        let (tool, settings) = (self.tool, &self.settings);
        let is_active = |id: CommandId| match id {
            CommandId::Herramienta(t) => t == tool,
            CommandId::Rejilla => settings.grid_visible,
            CommandId::Snap => settings.snap,
            CommandId::Orto => settings.ortho,
            _ => false,
        };
        let cx = ribbon::Ctx { icons: self.icons, is_active: &is_active };
        egui::Panel::top("pestanas")
            .frame(egui::Frame::new().fill(theme::TITLE_BG).inner_margin(egui::Margin::symmetric(6, 3)))
            .show(ui, |ui| ribbon::tabs_row(ui, &mut self.ribbon, &cx, &mut clicked));
        egui::Panel::top("cinta")
            .frame(egui::Frame::new().fill(theme::RIBBON_BG).inner_margin(egui::Margin::symmetric(6, 4)))
            .show(ui, |ui| ribbon::body(ui, &self.ribbon, &cx, &mut clicked));
        egui::Panel::bottom("estado")
            .frame(egui::Frame::new().fill(theme::TITLE_BG).inner_margin(egui::Margin::symmetric(8, 3)))
            .show(ui, |ui| clicked.extend(self.status_bar(ui)));

        let mut viewer = Viewer {
            project: &mut self.project,
            views: &mut self.views,
            settings: &self.settings,
            state: &mut self.panels,
            actions: Vec::new(),
            canvas: None,
        };
        egui::CentralPanel::no_frame().show(ui, |ui| {
            DockArea::new(&mut self.dock)
                .style(theme::dock_style(ui.style()))
                .show_leaf_collapse_buttons(false)
                .show_inside(ui, &mut viewer);
        });
        let Viewer { actions, canvas, .. } = viewer;
        self.canvas = canvas;

        for action in actions {
            match action {
                Action::Command(id) => self.execute(&ctx, id),
                Action::OpenPage(id) => self.open_page(id),
                Action::Status(s) => self.status = s,
            }
        }
        for id in clicked {
            self.execute(&ctx, id);
        }
        if let Some(id) = self.palette_ui(&ctx) {
            self.execute(&ctx, id);
        }

        if self.about_open {
            let r = egui::Modal::new(egui::Id::new("acerca_de")).show(&ctx, |ui| {
                ui.heading("open-ecad");
                ui.label("CAD eléctrico open-source. Prototipo de la ventana: sin modelo de dominio todavía.");
                ui.add_space(8.0);
                ui.button("Cerrar").clicked()
            });
            if r.inner || r.should_close() {
                self.about_open = false;
            }
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let persisted = Persisted { dock: self.dock.clone(), project: self.project.clone() };
        eframe::set_value(storage, STORAGE_KEY, &persisted);
    }
}

fn toggle(flag: &mut bool, name: &str, status: &mut String) {
    *flag = !*flag;
    *status = format!("{name}: {}", if *flag { "activado" } else { "desactivado" });
}

/// Disposición por defecto (MAPA-UI.md §4.1): páginas en el centro, navegación a la
/// izquierda, inserción y propiedades a la derecha, listas abajo.
fn default_dock(project: &Project) -> DockState<Tab> {
    let first_pages: Vec<Tab> = project.pages.iter().skip(2).take(2).map(|p| Tab::Page(p.id)).collect();
    let mut dock = DockState::new(first_pages);
    let tree = dock.main_surface_mut();
    let [center, left] = tree.split_left(NodeIndex::root(), 0.2, vec![Tab::Panel(PanelKind::Paginas)]);
    tree.split_below(left, 0.65, vec![Tab::Panel(PanelKind::Preview)]);
    let [center, right] = tree.split_right(
        center,
        0.78,
        vec![Tab::Panel(PanelKind::Insercion), Tab::Panel(PanelKind::Macros)],
    );
    tree.split_below(right, 0.55, vec![Tab::Panel(PanelKind::Propiedades)]);
    tree.split_below(
        center,
        0.75,
        vec![
            Tab::Panel(PanelKind::Mensajes),
            Tab::Panel(PanelKind::PuntosInterrupcion),
            Tab::Panel(PanelKind::Dispositivos),
        ],
    );
    dock
}
