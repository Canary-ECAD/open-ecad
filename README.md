# open-ecad

CAD eléctrico open-source, construido en Rust. Inspirado en el flujo de trabajo de
EPLAN Electric P8, sin su ruido: menos diálogos, menos capas de configuración, el mismo
poder de ingeniería.

**Enfoque clean-room**: el dominio se deriva exclusivamente de documentación pública y
estándares abiertos (IEC 81346, IEC 60617). Sin ingeniería inversa, sin código ni
assets de terceros.

## Estado

Fase de diseño. El modelo de dominio está mapeado en [MAPA-DOMINIO.md](MAPA-DOMINIO.md):

- **v1** — esquemático 2D: funciones + símbolos, motor de conexiones derivadas de la
  topología, potenciales y referencias cruzadas entre páginas, librería de símbolos
  abierta (SVG + JSON), macros con placeholders, undo transaccional, export PDF/SVG.
- **v2** — base de artículos, verificaciones eléctricas, renumeración, reportes, DXF/DWG.
- **v3** — AML/AutomationML, multiusuario, revisiones.

## Stack previsto

Rust + egui (canvas inmediato, docking, multiplataforma + WASM), con el modelo de
dominio como crate independiente del GUI.

## Licencia

Pendiente de definir (previsiblemente dual MIT / Apache-2.0, convención del ecosistema Rust).
