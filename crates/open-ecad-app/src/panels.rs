//! Pestañas del dock: páginas abiertas y paneles (MAPA-UI.md §4.3).

use std::collections::HashMap;

use egui::{CollapsingHeader, Id, RichText, Sense, TextEdit, Ui, WidgetText};
use egui_dock::TabViewer;
use serde::{Deserialize, Serialize};

use crate::canvas::{self, PageView, ViewSettings};
use crate::commands::{CommandId, Tool, fold};
use crate::mock::{self, PageId, Project};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PanelKind {
    Paginas,
    Preview,
    Insercion,
    Propiedades,
    Mensajes,
    PuntosInterrupcion,
    Dispositivos,
    Macros,
}

impl PanelKind {
    pub fn title(self) -> &'static str {
        match self {
            PanelKind::Paginas => "Páginas",
            PanelKind::Preview => "Previsualización",
            PanelKind::Insercion => "Inserción",
            PanelKind::Propiedades => "Propiedades",
            PanelKind::Mensajes => "Mensajes",
            PanelKind::PuntosInterrupcion => "Puntos de interrupción",
            PanelKind::Dispositivos => "Dispositivos",
            PanelKind::Macros => "Macros",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Tab {
    Page(PageId),
    Panel(PanelKind),
}

/// Lo que los paneles piden a la ventana; se aplica después de dibujar el dock.
pub enum Action {
    Command(CommandId),
    OpenPage(PageId),
    Status(String),
}

/// Estado de interfaz de los paneles (filtros, modo árbol/lista...).
#[derive(Default)]
pub struct PanelsState {
    pub selected_page: Option<PageId>,
    navigators: HashMap<PanelKind, NavigatorState>,
    insertion_macros: bool,
    insertion_selected: Option<&'static str>,
}

#[derive(Default)]
struct NavigatorState {
    filter: String,
    list_view: bool,
}

/// Datos que necesitan las pestañas mientras se dibuja el dock.
pub struct Viewer<'a> {
    pub project: &'a mut Project,
    pub views: &'a mut HashMap<PageId, PageView>,
    pub settings: &'a ViewSettings,
    pub state: &'a mut PanelsState,
    pub actions: Vec<Action>,
    pub canvas: Option<canvas::CanvasOutput>,
}

impl TabViewer for Viewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> WidgetText {
        match tab {
            Tab::Page(id) => self.project.page(*id).map(|p| p.name()).unwrap_or_else(|| "?".into()).into(),
            Tab::Panel(kind) => kind.title().into(),
        }
    }

    fn id(&mut self, tab: &mut Tab) -> Id {
        Id::new(*tab)
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Tab) {
        match *tab {
            Tab::Page(id) => self.page_ui(ui, id),
            Tab::Panel(kind) => match kind {
                PanelKind::Paginas => self.pages_ui(ui),
                PanelKind::Preview => self.preview_ui(ui),
                PanelKind::Insercion => self.insertion_ui(ui),
                PanelKind::Propiedades => self.properties_ui(ui),
                PanelKind::Mensajes => messages_ui(ui),
                PanelKind::PuntosInterrupcion => self.interruptions_ui(ui),
                PanelKind::Dispositivos => self.devices_ui(ui),
                PanelKind::Macros => self.macros_ui(ui),
            },
        }
    }

    fn scroll_bars(&self, tab: &Tab) -> [bool; 2] {
        match tab {
            Tab::Page(_) | Tab::Panel(PanelKind::Preview) => [false, false],
            Tab::Panel(_) => [true, true],
        }
    }

    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if let Tab::Page(id) = tab
            && response.clicked()
        {
            self.state.selected_page = Some(*id);
        }
    }
}

impl Viewer<'_> {
    fn page_ui(&mut self, ui: &mut Ui, id: PageId) {
        let Some(page) = self.project.page(id) else {
            ui.label("Página no encontrada");
            return;
        };
        let view = self.views.entry(id).or_default();
        let out = view.ui(ui, page, self.settings);
        // Solo la página bajo el ratón informa a la barra de estado.
        if out.cursor_mm.is_some() || self.canvas.is_none() {
            self.canvas = Some(out);
        }
    }

    fn pages_ui(&mut self, ui: &mut Ui) {
        let nav = self.state.navigators.entry(PanelKind::Paginas).or_default();
        navigator_header(ui, nav);
        let filter = fold(&nav.filter);
        let list_view = nav.list_view;
        let visible: Vec<_> = self
            .project
            .pages
            .iter()
            .filter(|p| filter.is_empty() || fold(&format!("{} {}", p.name(), p.description)).contains(&filter))
            .collect();

        let mut clicked = None;
        let mut opened = None;
        let mut row = |ui: &mut Ui, label: String, id: PageId| {
            let selected = self.state.selected_page == Some(id);
            let r = ui.selectable_label(selected, label);
            if r.clicked() {
                clicked = Some(id);
            }
            if r.double_clicked() {
                opened = Some(id);
            }
        };

        if list_view {
            egui::Grid::new("pages_list").striped(true).num_columns(3).show(ui, |ui| {
                ui.strong("Página");
                ui.strong("Descripción");
                ui.strong("Tipo");
                ui.end_row();
                for p in &visible {
                    row(ui, p.name(), p.id);
                    ui.label(&p.description);
                    ui.label(&p.kind);
                    ui.end_row();
                }
            });
        } else {
            CollapsingHeader::new(&self.project.name).default_open(true).show(ui, |ui| {
                let mut structures: Vec<&str> = visible.iter().map(|p| p.structure.as_str()).collect();
                structures.dedup();
                structures.sort_unstable();
                structures.dedup();
                for s in structures {
                    CollapsingHeader::new(s).default_open(true).show(ui, |ui| {
                        for p in visible.iter().filter(|p| p.structure == s) {
                            row(ui, format!("{}  {}", p.counter, p.description), p.id);
                        }
                    });
                }
            });
        }

        if let Some(id) = clicked {
            self.state.selected_page = Some(id);
        }
        if let Some(id) = opened {
            self.actions.push(Action::OpenPage(id));
        }
        ui.add_space(6.0);
        if ui.small_button("+ Nueva página").clicked() {
            self.actions.push(Action::Command(CommandId::NuevaPagina));
        }
    }

    fn preview_ui(&mut self, ui: &mut Ui) {
        let Some(page) = self.state.selected_page.and_then(|id| self.project.page(id)) else {
            ui.weak("Selecciona una página en el navegador.");
            return;
        };
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        let t = canvas::Transform::fit(rect.shrink(6.0));
        let settings = ViewSettings { grid_visible: false, ..Default::default() };
        canvas::paint_page(&ui.painter_at(rect), &t, page, &settings, rect);
    }

    fn insertion_ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.state.insertion_macros, false, "Símbolos");
            ui.selectable_value(&mut self.state.insertion_macros, true, "Macros");
        });
        let nav = self.state.navigators.entry(PanelKind::Insercion).or_default();
        ui.add(TextEdit::singleline(&mut nav.filter).hint_text("Buscar…").desired_width(f32::INFINITY));
        let filter = fold(&nav.filter);
        ui.separator();

        let mut picked = None;
        if self.state.insertion_macros {
            for (name, kind) in mock::MACROS {
                if filter.is_empty() || fold(name).contains(&filter) {
                    let r = ui.selectable_label(self.state.insertion_selected == Some(*name), *name);
                    if r.on_hover_text(*kind).clicked() {
                        picked = Some((*name, Tool::Macro));
                    }
                }
            }
        } else {
            CollapsingHeader::new("IEC 60617 (ejemplo)").default_open(true).show(ui, |ui| {
                for (group, symbols) in mock::SYMBOL_LIBRARY {
                    let matches: Vec<_> =
                        symbols.iter().filter(|s| filter.is_empty() || fold(s).contains(&filter)).collect();
                    if matches.is_empty() {
                        continue;
                    }
                    CollapsingHeader::new(*group).default_open(!filter.is_empty()).show(ui, |ui| {
                        for s in matches {
                            if ui.selectable_label(self.state.insertion_selected == Some(*s), *s).clicked() {
                                picked = Some((*s, Tool::Simbolo));
                            }
                        }
                    });
                }
            });
        }
        if let Some((name, tool)) = picked {
            self.state.insertion_selected = Some(name);
            self.actions.push(Action::Command(CommandId::Herramienta(tool)));
            self.actions.push(Action::Status(format!("«{name}»: haz clic en la página para colocarlo (pendiente)")));
        }
    }

    fn properties_ui(&mut self, ui: &mut Ui) {
        let Some(page) = self.state.selected_page.and_then(|id| self.project.page_mut(id)) else {
            ui.weak("Nada seleccionado.");
            return;
        };
        ui.label(RichText::new(format!("Página {}", page.name())).strong());
        ui.separator();
        egui::Grid::new("page_props").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            ui.label("Estructura");
            ui.label(&page.structure);
            ui.end_row();
            ui.label("Contador");
            ui.label(page.counter.to_string());
            ui.end_row();
            ui.label("Descripción");
            ui.text_edit_singleline(&mut page.description);
            ui.end_row();
            ui.label("Tipo");
            ui.label(&page.kind);
            ui.end_row();
            ui.label("Marco");
            ui.label("A3 apaisado (ejemplo)");
            ui.end_row();
            ui.label("Escala");
            ui.label("1:1");
            ui.end_row();
        });
    }

    fn interruptions_ui(&mut self, ui: &mut Ui) {
        let nav = self.state.navigators.entry(PanelKind::PuntosInterrupcion).or_default();
        navigator_header(ui, nav);
        let filter = fold(&nav.filter);
        egui::Grid::new("interruptions").striped(true).num_columns(3).show(ui, |ui| {
            ui.strong("Nombre");
            ui.strong("Origen");
            ui.strong("Destinos");
            ui.end_row();
            for (name, from, to) in mock::INTERRUPTIONS {
                if filter.is_empty() || fold(name).contains(&filter) {
                    ui.label(*name);
                    ui.label(*from);
                    ui.label(*to);
                    ui.end_row();
                }
            }
        });
    }

    fn devices_ui(&mut self, ui: &mut Ui) {
        let nav = self.state.navigators.entry(PanelKind::Dispositivos).or_default();
        navigator_header(ui, nav);
        let filter = fold(&nav.filter);
        let list_view = nav.list_view;
        let visible =
            mock::DEVICES.iter().filter(|d| filter.is_empty() || fold(&format!("{} {}", d.name, d.description)).contains(&filter));
        if list_view {
            egui::Grid::new("devices").striped(true).num_columns(3).show(ui, |ui| {
                ui.strong("Identificador");
                ui.strong("Descripción");
                ui.strong("Funciones");
                ui.end_row();
                for d in visible {
                    ui.label(d.name);
                    ui.label(d.description);
                    ui.label(d.functions.len().to_string());
                    ui.end_row();
                }
            });
        } else {
            for d in visible {
                CollapsingHeader::new(format!("{}  {}", d.name, d.description)).show(ui, |ui| {
                    for f in d.functions {
                        ui.label(*f);
                    }
                });
            }
        }
    }

    fn macros_ui(&mut self, ui: &mut Ui) {
        let nav = self.state.navigators.entry(PanelKind::Macros).or_default();
        navigator_header(ui, nav);
        let filter = fold(&nav.filter);
        egui::Grid::new("macros").striped(true).num_columns(2).show(ui, |ui| {
            ui.strong("Macro");
            ui.strong("Tipo");
            ui.end_row();
            for (name, kind) in mock::MACROS {
                if filter.is_empty() || fold(name).contains(&filter) {
                    ui.label(*name);
                    ui.label(*kind);
                    ui.end_row();
                }
            }
        });
    }
}

fn messages_ui(ui: &mut Ui) {
    egui::Grid::new("messages").striped(true).num_columns(4).show(ui, |ui| {
        ui.strong("Clase");
        ui.strong("Región");
        ui.strong("Mensaje");
        ui.strong("Objeto");
        ui.end_row();
        ui.label("Indicación");
        ui.label("—");
        ui.label("Prototipo de interfaz: el proyecto contiene datos de ejemplo.");
        ui.label("—");
        ui.end_row();
    });
}

/// Cabecera común de los navegadores (MAPA-UI.md §3.2): filtro rápido y árbol/lista.
fn navigator_header(ui: &mut Ui, nav: &mut NavigatorState) {
    ui.horizontal(|ui| {
        ui.selectable_value(&mut nav.list_view, false, "Árbol");
        ui.selectable_value(&mut nav.list_view, true, "Lista");
        ui.add(TextEdit::singleline(&mut nav.filter).hint_text("Filtrar…").desired_width(f32::INFINITY));
    });
    ui.separator();
}
