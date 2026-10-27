//! Registro único de comandos (MAPA-UI.md §5).
//!
//! Menús, barra de herramientas, atajos y paleta leen de aquí: un comando existe una vez y
//! aparece en tantos sitios como haga falta.

use egui::{Key, KeyboardShortcut, Modifiers};

use crate::panels::PanelKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Menu {
    Archivo,
    Editar,
    Ver,
    Insertar,
    Proyecto,
    Herramientas,
    Datos,
    Ayuda,
}

impl Menu {
    pub const ALL: [Menu; 8] = [
        Menu::Archivo,
        Menu::Editar,
        Menu::Ver,
        Menu::Insertar,
        Menu::Proyecto,
        Menu::Herramientas,
        Menu::Datos,
        Menu::Ayuda,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Menu::Archivo => "Archivo",
            Menu::Editar => "Editar",
            Menu::Ver => "Ver",
            Menu::Insertar => "Insertar",
            Menu::Proyecto => "Proyecto",
            Menu::Herramientas => "Herramientas",
            Menu::Datos => "Datos",
            Menu::Ayuda => "Ayuda",
        }
    }
}

/// Herramienta activa del canvas. En el prototipo solo se marca; la máquina de
/// interacciones (MAPA-DOMINIO.md §15) llegará con el dominio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Seleccion,
    Linea,
    Rectangulo,
    Circulo,
    Texto,
    Simbolo,
    Macro,
}

impl Tool {
    pub fn label(self) -> &'static str {
        match self {
            Tool::Seleccion => "Selección",
            Tool::Linea => "Línea",
            Tool::Rectangulo => "Rectángulo",
            Tool::Circulo => "Círculo",
            Tool::Texto => "Texto",
            Tool::Simbolo => "Símbolo",
            Tool::Macro => "Macro",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandId {
    // Archivo
    NuevoProyecto,
    AbrirProyecto,
    Guardar,
    GuardarComo,
    CerrarProyecto,
    PropiedadesProyecto,
    Imprimir,
    ExportarPdf,
    Ajustes,
    Salir,
    // Editar
    Deshacer,
    Rehacer,
    Cortar,
    Copiar,
    Pegar,
    Duplicar,
    Borrar,
    Mover,
    Girar,
    Reflejar,
    Estirar,
    Agrupar,
    Desagrupar,
    AlinearRejilla,
    TraerDelante,
    EnviarDetras,
    SeleccionarMismoTipo,
    // Ver
    MostrarPanel(PanelKind),
    AjustarPagina,
    Zoom100,
    Rejilla,
    Snap,
    Orto,
    PuntosInsercion,
    ElementosInvisibles,
    SeguimientoRed,
    SeguimientoPotencial,
    RestablecerDisposicion,
    // Insertar
    Herramienta(Tool),
    Imagen,
    Acotacion,
    PuntoInterrupcion,
    DefinicionPotencial,
    Esquina,
    NodoT,
    Cruce,
    CorteAutoconexion,
    // Proyecto
    NuevaPagina,
    PropiedadesPagina,
    Estructura,
    Capas,
    RegenerarConexiones,
    // Herramientas
    Comprobar,
    // Datos
    BibliotecaSimbolos,
    EditarSimbolo,
    CrearMacro,
    Cajetines,
    // Ayuda
    PaletaComandos,
    AcercaDe,
}

pub struct Command {
    pub id: CommandId,
    pub menu: Menu,
    /// Submenú dentro del menú (p. ej. "Paneles"), si lo hay.
    pub submenu: Option<&'static str>,
    pub label: &'static str,
    pub shortcut: Option<KeyboardShortcut>,
    /// Los comandos con distinto grupo dentro de un menú se separan con una línea.
    pub group: u8,
    /// `false` mientras el comando solo tiene sitio en la UI y no hace nada todavía.
    pub implemented: bool,
}

const fn cmd(id: CommandId, menu: Menu, group: u8, label: &'static str) -> Command {
    Command { id, menu, submenu: None, label, shortcut: None, group, implemented: false }
}

const fn ctrl(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::COMMAND, key)
}

const fn ctrl_shift(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::COMMAND.plus(Modifiers::SHIFT), key)
}

const fn key(key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(Modifiers::NONE, key)
}

impl Command {
    const fn sc(mut self, shortcut: KeyboardShortcut) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    const fn sub(mut self, submenu: &'static str) -> Self {
        self.submenu = Some(submenu);
        self
    }

    const fn ok(mut self) -> Self {
        self.implemented = true;
        self
    }
}

use CommandId as C;
use Menu as M;

pub static COMMANDS: &[Command] = &[
    // Archivo
    cmd(C::NuevoProyecto, M::Archivo, 0, "Nuevo proyecto…").sc(ctrl(Key::N)),
    cmd(C::AbrirProyecto, M::Archivo, 0, "Abrir proyecto…").sc(ctrl(Key::O)),
    cmd(C::Guardar, M::Archivo, 1, "Guardar").sc(ctrl(Key::S)),
    cmd(C::GuardarComo, M::Archivo, 1, "Guardar como…").sc(ctrl_shift(Key::S)),
    cmd(C::CerrarProyecto, M::Archivo, 1, "Cerrar proyecto"),
    cmd(C::PropiedadesProyecto, M::Archivo, 2, "Propiedades del proyecto"),
    cmd(C::Imprimir, M::Archivo, 3, "Imprimir…").sc(ctrl(Key::P)),
    cmd(C::ExportarPdf, M::Archivo, 3, "Exportar PDF…"),
    cmd(C::Ajustes, M::Archivo, 4, "Ajustes…"),
    cmd(C::Salir, M::Archivo, 5, "Salir").ok(),
    // Editar
    cmd(C::Deshacer, M::Editar, 0, "Deshacer").sc(ctrl(Key::Z)),
    cmd(C::Rehacer, M::Editar, 0, "Rehacer").sc(ctrl(Key::Y)),
    cmd(C::Cortar, M::Editar, 1, "Cortar").sc(ctrl(Key::X)),
    cmd(C::Copiar, M::Editar, 1, "Copiar").sc(ctrl(Key::C)),
    cmd(C::Pegar, M::Editar, 1, "Pegar").sc(ctrl(Key::V)),
    cmd(C::Duplicar, M::Editar, 1, "Duplicar").sc(key(Key::D)),
    cmd(C::Borrar, M::Editar, 1, "Borrar").sc(key(Key::Delete)),
    cmd(C::Mover, M::Editar, 2, "Mover").sc(key(Key::M)),
    cmd(C::Girar, M::Editar, 2, "Girar").sc(key(Key::R)),
    cmd(C::Reflejar, M::Editar, 2, "Reflejar"),
    cmd(C::Estirar, M::Editar, 2, "Estirar"),
    cmd(C::Agrupar, M::Editar, 3, "Agrupar"),
    cmd(C::Desagrupar, M::Editar, 3, "Desagrupar"),
    cmd(C::AlinearRejilla, M::Editar, 4, "Alinear a la rejilla"),
    cmd(C::TraerDelante, M::Editar, 4, "Traer al frente"),
    cmd(C::EnviarDetras, M::Editar, 4, "Enviar al fondo"),
    cmd(C::SeleccionarMismoTipo, M::Editar, 5, "Seleccionar los del mismo tipo"),
    // Ver
    cmd(C::MostrarPanel(PanelKind::Paginas), M::Ver, 0, "Páginas").sub("Paneles").ok(),
    cmd(C::MostrarPanel(PanelKind::Preview), M::Ver, 0, "Previsualización").sub("Paneles").ok(),
    cmd(C::MostrarPanel(PanelKind::Insercion), M::Ver, 0, "Inserción").sub("Paneles").ok(),
    cmd(C::MostrarPanel(PanelKind::Propiedades), M::Ver, 0, "Propiedades").sub("Paneles").ok(),
    cmd(C::MostrarPanel(PanelKind::Mensajes), M::Ver, 0, "Mensajes").sub("Paneles").ok(),
    cmd(C::MostrarPanel(PanelKind::PuntosInterrupcion), M::Ver, 0, "Puntos de interrupción")
        .sub("Paneles")
        .ok(),
    cmd(C::MostrarPanel(PanelKind::Dispositivos), M::Ver, 0, "Dispositivos").sub("Paneles").ok(),
    cmd(C::MostrarPanel(PanelKind::Macros), M::Ver, 0, "Macros").sub("Paneles").ok(),
    cmd(C::AjustarPagina, M::Ver, 1, "Ajustar página").sc(key(Key::Home)).ok(),
    cmd(C::Zoom100, M::Ver, 1, "Escala 1:1").ok(),
    cmd(C::Rejilla, M::Ver, 2, "Rejilla").sc(key(Key::F7)).ok(),
    cmd(C::Snap, M::Ver, 2, "Ajustar a la rejilla (snap)").sc(key(Key::F9)).ok(),
    cmd(C::Orto, M::Ver, 2, "Modo orto").sc(key(Key::F8)).ok(),
    cmd(C::PuntosInsercion, M::Ver, 3, "Puntos de inserción"),
    cmd(C::ElementosInvisibles, M::Ver, 3, "Elementos invisibles"),
    cmd(C::SeguimientoRed, M::Ver, 4, "Seguimiento de red"),
    cmd(C::SeguimientoPotencial, M::Ver, 4, "Seguimiento de potencial"),
    cmd(C::RestablecerDisposicion, M::Ver, 5, "Restablecer disposición").ok(),
    // Insertar
    cmd(C::Herramienta(Tool::Simbolo), M::Insertar, 0, "Símbolo…").sc(key(Key::Insert)).ok(),
    cmd(C::Herramienta(Tool::Macro), M::Insertar, 0, "Macro…").ok(),
    cmd(C::Herramienta(Tool::Linea), M::Insertar, 1, "Línea").sub("Gráfico").ok(),
    cmd(C::Herramienta(Tool::Rectangulo), M::Insertar, 1, "Rectángulo").sub("Gráfico").ok(),
    cmd(C::Herramienta(Tool::Circulo), M::Insertar, 1, "Círculo").sub("Gráfico").ok(),
    cmd(C::Herramienta(Tool::Texto), M::Insertar, 1, "Texto").sc(key(Key::T)).ok(),
    cmd(C::Imagen, M::Insertar, 1, "Imagen…"),
    cmd(C::Acotacion, M::Insertar, 1, "Acotación lineal"),
    cmd(C::PuntoInterrupcion, M::Insertar, 2, "Punto de interrupción"),
    cmd(C::DefinicionPotencial, M::Insertar, 2, "Definición de potencial"),
    cmd(C::Esquina, M::Insertar, 3, "Esquina").sub("Piezas de conexión"),
    cmd(C::NodoT, M::Insertar, 3, "Nodo T").sub("Piezas de conexión"),
    cmd(C::Cruce, M::Insertar, 3, "Cruce").sub("Piezas de conexión"),
    cmd(C::CorteAutoconexion, M::Insertar, 3, "Corte de autoconexión").sub("Piezas de conexión"),
    // Proyecto
    cmd(C::NuevaPagina, M::Proyecto, 0, "Nueva página").sc(ctrl_shift(Key::N)).ok(),
    cmd(C::PropiedadesPagina, M::Proyecto, 0, "Propiedades de página"),
    cmd(C::Estructura, M::Proyecto, 1, "Estructura (identificadores)…"),
    cmd(C::Capas, M::Proyecto, 1, "Capas…"),
    cmd(C::RegenerarConexiones, M::Proyecto, 2, "Regenerar conexiones"),
    // Herramientas
    cmd(C::MostrarPanel(PanelKind::Mensajes), M::Herramientas, 0, "Mensajes").ok(),
    cmd(C::Comprobar, M::Herramientas, 0, "Comprobar proyecto (v2)"),
    // Datos
    cmd(C::BibliotecaSimbolos, M::Datos, 0, "Biblioteca de símbolos…"),
    cmd(C::EditarSimbolo, M::Datos, 0, "Editar símbolo…"),
    cmd(C::CrearMacro, M::Datos, 1, "Crear macro desde selección"),
    cmd(C::Cajetines, M::Datos, 2, "Cajetines y marcos…"),
    // Ayuda
    cmd(C::PaletaComandos, M::Ayuda, 0, "Paleta de comandos…").sc(ctrl(Key::K)).ok(),
    cmd(C::AcercaDe, M::Ayuda, 1, "Acerca de open-ecad").ok(),
];

pub fn get(id: CommandId) -> &'static Command {
    COMMANDS
        .iter()
        .find(|c| c.id == id)
        .expect("todo CommandId debe estar en el registro")
}

/// Texto para buscar en la paleta: menú, submenú y nombre.
pub fn search_text(c: &Command) -> String {
    let mut s = String::from(c.menu.label());
    if let Some(sub) = c.submenu {
        s.push(' ');
        s.push_str(sub);
    }
    s.push(' ');
    s.push_str(c.label);
    fold(&s)
}

/// Minúsculas y sin tildes, para que "simbolo" encuentre "Símbolo".
pub fn fold(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'Á' => 'a',
            'é' | 'É' => 'e',
            'í' | 'Í' => 'i',
            'ó' | 'Ó' => 'o',
            'ú' | 'Ú' | 'ü' | 'Ü' => 'u',
            'ñ' | 'Ñ' => 'n',
            c => c.to_ascii_lowercase(),
        })
        .collect()
}
