//! Ventana principal: cinta, área del plano fija en el centro, zonas de paneles
//! alrededor, barra de estado y paleta de comandos (MAPA-UI.md §4.1).

use std::collections::HashMap;

use egui::{Button, Key, Modifiers, RichText, Ui, ViewportCommand};
use egui_dock::{DockArea, DockState, NodeIndex};
use serde::{Deserialize, Serialize};

use crate::canvas::{CanvasOutput, PageView, ViewSettings};
use crate::commands::{self, COMMANDS, CommandId, Tool};
use crate::mock::{PageId, Project};
use crate::panels::{Action, PanelKind, PanelsState, Region, Viewer};
use crate::ribbon::{self, RibbonState};
use crate::theme;

/// Cambiar la clave invalida disposiciones guardadas incompatibles.
const STORAGE_KEY: &str = "open-ecad/ventana/v2";

/// Un dock por zona. Dentro de cada zona los paneles se apilan, dividen y reordenan
/// libremente; entre zonas se mueven con el menú contextual de la pestaña.
#[derive(Clone, Serialize, Deserialize)]
struct Docks {
    izquierda: DockState<PanelKind>,
    derecha: DockState<PanelKind>,
    abajo: DockState<PanelKind>,
}

impl Docks {
    fn get_mut(&mut self, region: Region) -> &mut DockState<PanelKind> {
        match region {
            Region::Izquierda => &mut self.izquierda,
            Region::Derecha => &mut self.derecha,
            Region::Abajo => &mut self.abajo,
        }
    }

    fn remove(&mut self, kind: PanelKind) {
        for region in Region::ALL {
            let dock = self.get_mut(region);
            if let Some(path) = dock.find_tab(&kind) {
                dock.remove_tab(path);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Persisted {
    docks: Docks,
    project: Project,
    current_page: Option<PageId>,
}

pub struct OpenEcadApp {
    docks: Docks,
    project: Project,
    current_page: Option<PageId>,
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
        let (docks, project, current_page) = match persisted {
            Some(p) => (p.docks, p.project, p.current_page),
            None => {
                let project = Project::default();
                let first = project.pages.get(2).map(|p| p.id);
                (default_docks(), project, first)
            }
        };
        Self {
            docks,
            project,
            current_page,
            views: HashMap::new(),
            settings: ViewSettings::default(),
            panels: PanelsState::default(),
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
                self.docks = default_docks();
                self.status = "Disposición restablecida".into();
            }
            CommandId::AjustarPagina => self.views.values_mut().for_each(PageView::request_fit),
            CommandId::PaginaAnterior => self.step_page(-1),
            CommandId::PaginaSiguiente => self.step_page(1),
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
                    .current_page
                    .and_then(|id| self.project.page(id))
                    .map_or_else(|| "=A1+CAB".to_string(), |p| p.structure.clone());
                self.current_page = Some(self.project.add_page(&structure));
            }
            _ => self.status = format!("«{}» aún no está implementado", command.label.trim_end_matches('…')),
        }
    }

    fn show_panel(&mut self, kind: PanelKind) {
        for region in Region::ALL {
            let dock = self.docks.get_mut(region);
            if let Some(path) = dock.find_tab(&kind) {
                let _ = dock.set_active_tab(path);
                return;
            }
        }
        self.move_panel(kind, kind.default_region());
    }

    fn move_panel(&mut self, kind: PanelKind, region: Region) {
        self.docks.remove(kind);
        let dock = self.docks.get_mut(region);
        if dock.iter_all_tabs().next().is_none() {
            *dock = DockState::new(vec![kind]);
        } else {
            dock.push_to_first_leaf(kind);
        }
    }

    /// Página anterior/siguiente en el orden del proyecto (Re Pág / Av Pág).
    fn step_page(&mut self, delta: isize) {
        let pages = &self.project.pages;
        let pos = self.current_page.and_then(|id| pages.iter().position(|p| p.id == id)).unwrap_or(0);
        let next = (pos as isize + delta).clamp(0, pages.len() as isize - 1) as usize;
        self.current_page = pages.get(next).map(|p| p.id);
    }

    /// Área del plano: fija, siempre en el centro, una sola página a la vez.
    fn document_ui(&mut self, ui: &mut Ui, clicked: &mut Vec<CommandId>) {
        let Some(page) = self.current_page.and_then(|id| self.project.page(id)) else {
            ui.centered_and_justified(|ui| ui.weak("Selecciona una página en el navegador"));
            self.canvas = None;
            return;
        };
        egui::Frame::new().fill(theme::TITLE_BG).inner_margin(egui::Margin::symmetric(8, 3)).show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.small_button("◀").on_hover_text("Página anterior (Re Pág)").clicked() {
                    clicked.push(CommandId::PaginaAnterior);
                }
                if ui.small_button("▶").on_hover_text("Página siguiente (Av Pág)").clicked() {
                    clicked.push(CommandId::PaginaSiguiente);
                }
                ui.label(RichText::new(page.name()).strong().color(theme::TEXT_STRONG));
                ui.label(RichText::new(&page.description).color(theme::TEXT_WEAK));
            });
        });
        let view = self.views.entry(page.id).or_default();
        self.canvas = Some(view.ui(ui, page, &self.settings));
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
            state: &mut self.panels,
            current_page: &mut self.current_page,
            region: Region::Abajo,
            actions: Vec::new(),
        };
        let dock_style = theme::dock_style(ui.style());
        let mut show_region = |ui: &mut Ui, region: Region, dock: &mut DockState<PanelKind>| {
            viewer.region = region;
            DockArea::new(dock)
                .id(egui::Id::new(("zona", region)))
                .style(dock_style.clone())
                .show_leaf_collapse_buttons(false)
                .show_inside(ui, &mut viewer);
        };
        let Docks { izquierda, derecha, abajo } = &mut self.docks;
        let has_tabs = |d: &DockState<PanelKind>| d.iter_all_tabs().next().is_some();
        if has_tabs(abajo) {
            egui::Panel::bottom("zona_abajo")
                .resizable(true)
                .default_size(190.0)
                .frame(egui::Frame::NONE)
                .show(ui, |ui| show_region(ui, Region::Abajo, abajo));
        }
        if has_tabs(izquierda) {
            egui::Panel::left("zona_izquierda")
                .resizable(true)
                .default_size(270.0)
                .frame(egui::Frame::NONE)
                .show(ui, |ui| show_region(ui, Region::Izquierda, izquierda));
        }
        if has_tabs(derecha) {
            egui::Panel::right("zona_derecha")
                .resizable(true)
                .default_size(270.0)
                .frame(egui::Frame::NONE)
                .show(ui, |ui| show_region(ui, Region::Derecha, derecha));
        }
        let actions = viewer.actions;
        egui::CentralPanel::no_frame().show(ui, |ui| self.document_ui(ui, &mut clicked));

        for action in actions {
            match action {
                Action::Command(id) => self.execute(&ctx, id),
                Action::MoveTo(kind, region) => self.move_panel(kind, region),
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
        let persisted =
            Persisted { docks: self.docks.clone(), project: self.project.clone(), current_page: self.current_page };
        eframe::set_value(storage, STORAGE_KEY, &persisted);
    }
}

fn toggle(flag: &mut bool, name: &str, status: &mut String) {
    *flag = !*flag;
    *status = format!("{name}: {}", if *flag { "activado" } else { "desactivado" });
}

/// Disposición por defecto (MAPA-UI.md §4.1): navegación a la izquierda, inserción y
/// propiedades a la derecha, listas abajo. El plano siempre queda en el centro.
fn default_docks() -> Docks {
    let mut izquierda = DockState::new(vec![PanelKind::Paginas]);
    izquierda.main_surface_mut().split_below(NodeIndex::root(), 0.65, vec![PanelKind::Preview]);
    let mut derecha = DockState::new(vec![PanelKind::Insercion, PanelKind::Macros]);
    derecha.main_surface_mut().split_below(NodeIndex::root(), 0.55, vec![PanelKind::Propiedades]);
    let abajo = DockState::new(vec![PanelKind::Mensajes, PanelKind::PuntosInterrupcion, PanelKind::Dispositivos]);
    Docks { izquierda, derecha, abajo }
}
