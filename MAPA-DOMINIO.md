# open-ecad — Mapa del dominio (derivado clean-room del wiki API EPLAN P8 2026)

> **Metodología y cobertura.** Este mapa se derivó leyendo el 100% del *User Guide* (105
> documentos conceptuales) e inventariando estructuralmente el *API Reference* completo
> (17.180 documentos: 706 clases en ~26 namespaces, 98 actions, 40 convertidores XML,
> 13 eventos, 5 interacciones). Es una descripción **en palabras propias** de los conceptos
> del dominio del CAD eléctrico; no reproduce código ni texto de la documentación de EPLAN.
> Los docs fuente viven en `_docs-eplan/` (gitignored — no se distribuyen con el proyecto).

---

## 1. El panorama: namespaces y su peso

| Namespace (concepto) | Clases | Veredicto para open-ecad |
|---|---|---|
| `DataModel` — proyecto, páginas, funciones, conexiones, artículos | 274 | **NÚCLEO v1** |
| `DataModel.Graphics` — primitivas gráficas (línea, arco, texto, pen...) | 52 | **NÚCLEO v1** |
| `MasterData` + `DataModel.MasterData` — artículos, símbolos, formularios | 101 | Núcleo parcial (símbolos v1, artículos v2) |
| `EServices` (+`.Ged`) — verificaciones, mensajes, interacciones del editor | 34 | **NÚCLEO v1** (interacciones) / v2 (verificaciones) |
| `HEServices` — reportes, impresión, búsqueda, macros, selección | 49 | v1 parcial (selección, macros) / v2 (reportes) |
| `DataModel.E3D` — Pro Panel: armarios, carriles, ruteo 3D | 65 | **Fuera de alcance** (v3+ si acaso) |
| `DataModel.Planning` — Pre-planning (lazos PCT, segmentos) | 19 | Fuera de alcance |
| `ApplicationFramework`, `Gui`, `Scripting`, `Remoting`, `RemoteClient/Server`, `Starter` | ~60 | **Ruido** — es la capa de extensibilidad de EPLAN, no el dominio |
| `Base` — propiedades, settings, multilang, excepciones | 41 | Conceptos sí, API no |

**La lectura clave:** de las 706 clases, el dominio eléctrico real cabe en ~30 entidades.
El resto es (a) listas de propiedades autogeneradas, (b) 3D, (c) infraestructura de plugins.
Ahí está el "ruido" de EPLAN que open-ecad puede evitar.

---

## 2. Modelo de dominio — las decisiones de diseño que importan

### 2.1 Función ≠ Símbolo ≠ Dispositivo (el concepto más importante)

- La **Función** es la unidad lógica: un contacto, una bobina, un borne, un pin de PLC.
  Lleva la identificación del dispositivo y los puntos de conexión.
- El **Símbolo colocado** (symbol reference) es su representación gráfica en una página.
  Lógico y gráfico van vinculados, pero son objetos distintos.
- El **Dispositivo NO es una entidad almacenada**: emerge del conjunto de funciones que
  comparten el mismo identificador. Un relé = bobina en la pág. 3 + contactos en la pág. 10,
  todos con el tag `-K1`. Las vistas "regletero", "PLC", "conector" son agrupaciones
  derivadas, no tablas.

### 2.2 Identificación estructurada (IEC 81346)

El identificador de dispositivo no es una cadena plana: se compone de
**planta (`=`) + ubicación (`+`) + producto (`-`)**, donde producto = letra de código +
contador (ej. `=AP+PT1-X4`). Toda la numeración, las referencias cruzadas y los reportes
dependen de esta estructura. open-ecad debe modelarla como tipo propio desde el día 1.

### 2.3 Las conexiones se calculan, no se dibujan

El usuario coloca símbolos y líneas de autoconexión; un motor deriva las **conexiones**
(entre puntos de conexión de funciones) y los **potenciales** como datos del modelo.
Los **puntos de interrupción** continúan un potencial entre páginas y generan referencias
cruzadas automáticas. Esto —no el dibujo— es lo que distingue un CAD eléctrico de un
programa de dibujo. Es la pieza de ingeniería central de open-ecad.

### 2.4 Artículos: patrón flyweight con doble base

- **Artículo** (clave: nº de pieza + variante): datos del fabricante. Vive en una base
  central del sistema; al usarse, se copia a la base interna del proyecto (el proyecto es
  autónomo/portable).
- **Referencia de artículo**: cuelga de función, conexión o proyecto. Una pieza usada
  10 veces = 1 artículo + 10 referencias.

### 2.5 Sistema de propiedades genérico

Todo objeto persistible expone un diccionario de **propiedades tipadas** (bool, entero,
real, texto, coordenada, fecha, **texto multilingüe**), indexables (varios valores por
propiedad), con definiciones de tipo y propiedades de usuario. EPLAN identifica cada
propiedad con un ID numérico estable (ej. designación de conexión = 31011) — esos IDs son
la piedra Rosetta para interoperar con sus exports XML sin tocar nada propietario.

### 2.6 Datos maestros vs. proyecto

Los **datos maestros** (librerías de símbolos, formularios de reporte, marcos de página,
definiciones de función, macros) viven fuera del proyecto y se copian a su pool al usarse.
- **Símbolo** = (librería, número, letra de código, variante) + puntos de conexión.
- **Marco de página** (plot frame): carátula con rejilla de columnas/filas para referencias cruzadas.
- **Formulario**: plantilla de reporte (lista de bornes, de cables, BOM...).
- **Definición de función** (function template): describe qué pines y categoría tiene un
  tipo de función; conecta el mundo artículo ↔ función.
- **Macro**: fragmento de esquema reutilizable con placeholders y conjuntos de valores
  ("Motor 0,75 kW") — centro de la productividad real del proyectista.

### 2.7 Infraestructura del modelo digna de imitar

- **Transacciones anidables** todo-o-nada sobre el modelo (undo/redo nativo del dominio).
- **Verificación online**: chequeos que corren al confirmar cada cambio, con tres modos
  (avisar / impedir-revertir / bajo demanda) y mensajes Error/Warning/Info ligados a objetos.
- **Consulta sobre el grafo**: un buscador de objetos por tipo + filtros declarativos.
- **Locking** por objeto/página/proyecto (solo relevante si hay multiusuario — v3).

---

## 3. Servicios del flujo de trabajo

| Servicio | Qué hace para el proyectista | open-ecad |
|---|---|---|
| Interacciones del editor | Máquina de estados de herramientas del canvas: insertar símbolo/macro, seleccionar (con filtro por tipo), mover, borrar, capturar puntos | **v1** |
| Motor de conexiones | Deriva conexiones/potenciales de la topología gráfica | **v1** |
| Inserción de macros | Coloca fragmentos con placeholders + value sets | **v1** |
| Mensajería del proyecto | Bandeja de errores/avisos clasificados, ligados a objetos | v1 mínimo |
| Verificaciones | 4 dominios: funciones (tag duplicado, sin función principal), potenciales (conflictos de nombre), puntos de interrupción (emparejamiento), artículos | v2 |
| Reportes | Genera documentos derivados: listas de bornes, cables, conexiones, BOM, índices — sobre formularios | v2 |
| Numeración/renumeración | Device tags, bornes, cables, hilos | v2 |
| Búsqueda global | Texto en propiedades/páginas con esquemas de ajuste | v2 |
| Impresión / export gráfico | PDF, imagen, DXF/DWG | v1 (PDF/SVG simple), v2 (DXF) |
| Traducción | Diccionario multilingüe del proyecto | v3 |
| Backup/restore, revisiones | Empaquetado y control de revisiones | v3 |

---

## 4. El "ruido" de EPLAN que open-ecad NO debe replicar

1. **Toda la capa de extensibilidad**: add-ins .NET, add-ons con install.xml, shadow
   copying, scripting C#/VB en caliente, sistema de actions con ordinales, eventos por
   string, API de ribbon. Son el "cómo extiendo EPLAN", no el dominio. Un clon moderno
   usa una arquitectura de comandos propia y, si algún día hace falta, plugins diseñados
   de cero.
2. **Remoting/COM/cloud**: servidor gRPC embebido, interop COM con Office, IdentityClient,
   Data Portal, firma de assemblies, licencias. Irrelevante.
3. **La granularidad burocrática**: cientos de clases `*PropertyList` autogeneradas
   (Placement3D: 1.130 docs de propiedades). Con un sistema de propiedades genérico
   (§2.5), todo eso colapsa en una sola abstracción.
4. **Pro Panel (3D)** y **Pre-planning**: productos aparte dentro del producto. Diseñar
   el modelo para no cerrarles la puerta (el artículo puede llevar datos físicos), pero
   ni una línea de código v1.
5. **El doble modo API transaction / internal transaction, locking steps obligatorios,
   settings con 10 niveles**: complejidad histórica; un diseño nuevo lo resuelve con una
   sola primitiva de transacción y un árbol de settings plano.

---

## 5. Propuesta de alcance

### v1 — Esquemático 2D funcional
- Modelo: proyecto → páginas → funciones + símbolos colocados + primitivas gráficas;
  identificador IEC 81346; propiedades tipadas/multilingües; transacciones (undo/redo).
- Motor de conexiones: autoconnecting por topología, potenciales, puntos de interrupción
  con referencias cruzadas entre páginas.
- Librería de símbolos propia (formato abierto: SVG + JSON de puntos de conexión) con una
  librería inicial IEC 60617.
- Editor: canvas con rejilla, herramientas insertar/seleccionar/mover/borrar, macros con
  placeholders, marcos de página.
- Persistencia: formato propio legible (JSON/SQLite); export PDF/SVG.

### v2 — Proyectar de verdad
- Base de artículos (SQLite) + referencias; definiciones de función.
- Verificaciones (los 4 dominios) con modo online; renumeración; reportes sobre
  formularios (bornes, cables, BOM); import/export DXF/DWG; búsqueda global.

### v3 — Ecosistema
- AML/AutomationML e intercambio con los XML documentados de EPLAN (vía IDs de propiedad),
  multiusuario (locking), revisiones, traducción, API de plugins propia.

---

*Documento generado a partir de documentación pública (eplan.help) como especificación
funcional, siguiendo un enfoque clean-room: conceptos sí, implementación e interfaz no.*
