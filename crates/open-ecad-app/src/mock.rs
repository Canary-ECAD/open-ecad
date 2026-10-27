//! Datos ficticios para el prototipo de ventana. Se sustituirán por `open-ecad-model`.
//! Nombres genéricos a propósito: nada aquí procede de proyectos reales.

use serde::{Deserialize, Serialize};

pub type PageId = u32;

#[derive(Clone, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    /// Identificador de estructura heredado por la página (`=` planta, `+` ubicación).
    pub structure: String,
    pub counter: u32,
    pub description: String,
    pub kind: String,
}

impl Page {
    pub fn name(&self) -> String {
        format!("{}/{}", self.structure, self.counter)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub pages: Vec<Page>,
    next_id: PageId,
}

impl Default for Project {
    fn default() -> Self {
        let rows: &[(&str, &str, &'static str)] = &[
            ("=A1+CAB", "Portada", "Portada"),
            ("=A1+CAB", "Índice de páginas", "Índice"),
            ("=A1+CAB", "Alimentación general", "Esquema multifilar"),
            ("=A1+CAB", "Fuente 24 V DC", "Esquema multifilar"),
            ("=A1+CAB", "Motor bomba 1", "Esquema multifilar"),
            ("=A1+CAB", "Motor bomba 2", "Esquema multifilar"),
            ("=A1+CAB", "Mando y señalización", "Esquema multifilar"),
            ("=A2+PUP", "Pupitre de mando", "Esquema multifilar"),
            ("=A2+PUP", "Paradas de emergencia", "Esquema multifilar"),
            ("=A1+CAB", "Disposición del armario", "Montaje de armario 2D"),
        ];
        let mut counters = std::collections::HashMap::<&str, u32>::new();
        let pages = rows
            .iter()
            .enumerate()
            .map(|(i, (structure, description, kind))| {
                let counter = counters.entry(structure).or_default();
                *counter += 1;
                Page {
                    id: i as PageId + 1,
                    structure: structure.to_string(),
                    counter: *counter,
                    description: description.to_string(),
                    kind: kind.to_string(),
                }
            })
            .collect::<Vec<_>>();
        Self { name: "Proyecto de ejemplo".into(), next_id: pages.len() as PageId + 1, pages }
    }
}

impl Project {
    pub fn page(&self, id: PageId) -> Option<&Page> {
        self.pages.iter().find(|p| p.id == id)
    }

    pub fn page_mut(&mut self, id: PageId) -> Option<&mut Page> {
        self.pages.iter_mut().find(|p| p.id == id)
    }

    pub fn add_page(&mut self, structure: &str) -> PageId {
        let counter = self
            .pages
            .iter()
            .filter(|p| p.structure == structure)
            .map(|p| p.counter)
            .max()
            .unwrap_or(0)
            + 1;
        let id = self.next_id;
        self.next_id += 1;
        self.pages.push(Page {
            id,
            structure: structure.to_string(),
            counter,
            description: "Página nueva".into(),
            kind: "Esquema multifilar".into(),
        });
        id
    }
}

pub struct Device {
    pub name: &'static str,
    pub description: &'static str,
    pub functions: &'static [&'static str],
}

pub const DEVICES: &[Device] = &[
    Device { name: "-Q1", description: "Interruptor general", functions: &["1-2", "3-4", "5-6"] },
    Device { name: "-F1", description: "Guardamotor bomba 1", functions: &["1-2", "3-4", "5-6", "13-14"] },
    Device {
        name: "-K1",
        description: "Contactor bomba 1",
        functions: &["A1-A2 bobina", "1-2", "3-4", "5-6", "13-14", "21-22"],
    },
    Device { name: "-M1", description: "Bomba 1", functions: &["U1 V1 W1 PE"] },
    Device { name: "-T1", description: "Fuente 24 V DC", functions: &["L N PE", "+ -"] },
    Device { name: "-S1", description: "Pulsador marcha", functions: &["13-14"] },
    Device { name: "-S2", description: "Pulsador paro", functions: &["11-12"] },
];

/// (nombre, página de origen, páginas de destino)
pub const INTERRUPTIONS: &[(&str, &str, &str)] = &[
    ("L1", "=A1+CAB/3", "=A1+CAB/5, =A1+CAB/6"),
    ("L2", "=A1+CAB/3", "=A1+CAB/5, =A1+CAB/6"),
    ("L3", "=A1+CAB/3", "=A1+CAB/5, =A1+CAB/6"),
    ("PE", "=A1+CAB/3", "=A1+CAB/4, =A1+CAB/5"),
    ("24V", "=A1+CAB/4", "=A1+CAB/7, =A2+PUP/1"),
    ("0V", "=A1+CAB/4", "=A1+CAB/7, =A2+PUP/1"),
];

/// Árbol de la biblioteca de símbolos de ejemplo: (grupo, símbolos).
pub const SYMBOL_LIBRARY: &[(&str, &[&str])] = &[
    ("Contactos", &["Contacto NA", "Contacto NC", "Contacto conmutado"]),
    ("Bobinas", &["Bobina de relé", "Bobina de contactor"]),
    ("Protección", &["Fusible", "Magnetotérmico", "Guardamotor", "Diferencial"]),
    ("Motores", &["Motor trifásico", "Motor monofásico"]),
    ("Bornes", &["Borne", "Borne PE", "Borne seccionable"]),
    ("Mando", &["Pulsador NA", "Pulsador NC", "Seta de emergencia", "Piloto"]),
    ("Conexión", &["Esquina", "Nodo T", "Cruce", "Punto de interrupción"]),
];

pub const MACROS: &[(&str, &str)] = &[
    ("Arranque directo", "Ventana · multifilar"),
    ("Estrella-triángulo", "Ventana · multifilar"),
    ("Fuente 24 V con protección", "Ventana · multifilar"),
    ("Parada de emergencia doble canal", "Ventana · multifilar"),
    ("Portada", "Página"),
];
