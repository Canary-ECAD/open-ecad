# open-ecad — Mapa de la interfaz (UI)

> **Qué es este documento.** Inventario de la interfaz de EPLAN P8 *tal como la usa un
> proyectista en el día a día*, con el veredicto de qué entra en open-ecad, qué se fusiona y
> qué se descarta. Fuente: capturas y notas de uso real (carpeta local `imagenes/`, no
> versionada). Es **observación de uso**, no copia: open-ecad diseña su propia interfaz; de
> EPLAN tomamos el *flujo de trabajo*, no el aspecto ni la disposición.
>
> Complementa a [MAPA-DOMINIO.md](MAPA-DOMINIO.md): aquel dice *qué es* cada cosa; este dice
> *dónde y cuándo* la toca el usuario.
>
> **Es un documento vivo**: la §6 recoge las dudas y las capturas que faltan.

---

## 1. Lo que dicen las capturas, en cinco ideas

1. **El 80% del trabajo ocurre en tres sitios**: el árbol de páginas, el canvas y la
   inserción de símbolos/macros. Todo lo demás es ocasional.
2. **Hay mucha redundancia.** La pestaña *Inicio* repite cosas de *Editar*, *Insertar* y
   *Ver*; *Insertar → Medios de explotación / Bornes / Conectores / Cables* repite lo que ya
   ofrece el centro de inserción de símbolos; *Importar* y *Extras* se solapan.
3. **Los navegadores son todos el mismo componente**: título, filtro (esquema + valor),
   árbol/lista, tabla de columnas por propiedad. Cambia solo el tipo de objeto. → En
   open-ecad es **un único componente** parametrizado por consulta (§13 del mapa de dominio).
4. **Demasiados modales.** Propiedades, texto, filtro, ajustes, crear proyecto, copiar
   proyecto, "¿actualizar datos básicos?"... Muchos se pueden sustituir por un **panel de
   propiedades** fijo y por avisos no bloqueantes.
5. **Los paneles anclables con memoria son buenos**: se acoplan a cualquier borde, se
   apilan, flotan y recuerdan dónde estaban la última vez. Esto sí se conserva.

---

## 2. Inventario por menú y veredicto

Leyenda: **v1** entra en el esqueleto · **v2/v3** según fases del mapa de dominio ·
**fusionar** se resuelve con otra pieza · **descartar** no se replica.

### 2.1 Fichero (vista "backstage")

| Elemento | Uso real | Veredicto |
|---|---|---|
| Nuevo (nombre, ubicación, plantilla base, fecha, autor) | Sí | **v1** — un diálogo simple; fecha y autor automáticos |
| Abrir | Sí | **v1** + lista de recientes |
| Guardar / Salvaguardar (backup) | Sí | **v1** guardar; backup = copiar archivo (el formato es un fichero) |
| Copiar proyecto | Sí | v2 — "Guardar como" cubre lo básico en v1 |
| Borrar, Cerrar | Sí | **v1** |
| Información → Propiedades del proyecto | Sí | **v1** — en el panel de propiedades, no en backstage |
| Información → Estadística | Ocasional | v2 |
| Información → Estructura (árbol de identificadores de estructura) | Sí | **v1** — ver §4.4 |
| Información → Estado | Ocasional | v2 |
| Gestión de proyectos | Nunca | **descartar** |
| Gestión de revisión | Nunca | **descartar** — git / historial propio (v3) |
| Imprimir (impresora, páginas: actual / marcadas / todo / solo cambiadas, copias) | Sí, el modal | **v1** — diálogo único con preview |
| Exportar PDF | Sí | **v1** |
| Exportar DXF/DWG, imagen | Útil | v2 |
| Exportar propiedades a Excel (editar fuera y reimportar) | Útil | v2 — CSV ida y vuelta |
| Exportar lista de materiales / dispositivos | Útil | v2 |
| Exportar PLC, AutomationML, árbol de cables, fabricación de hilos | Poco | v3 |
| Importar datos de proyecto | Mayoría sin uso | v2, depurado (DXF + CSV) |
| Extras | Repite importar/exportar | **fusionar** en Importar/Exportar |
| Ajustes (árbol Proyectos / Usuario / Estación / Empresa) | Necesario pero denso | **v1 reducido** — dos niveles: *proyecto* y *usuario* |
| Ayuda, Salir | — | **v1** |

Modales al abrir proyecto ("comprobando datos", "¿actualizar BBDD de artículos?", "¿actualizar
datos básicos?") → **descartar como modales**: open-ecad avisa en la barra de estado o en
Mensajes y deja actualizar cuando el usuario quiera.

### 2.2 Inicio

Es una pestaña de "accesos rápidos" que duplica otras.

| Elemento | Veredicto |
|---|---|
| Portapapeles (cortar, copiar, pegar, borrar, copiar/asignar formato) | **fusionar** en Editar + atajos + menú contextual |
| Página: Navegador, Nueva, Numerar, Macro de página | **v1** Navegador y Nueva; v2 Numerar y macro de página |
| Espacio 3D | **descartar** (fuera de alcance) |
| Previsualización gráfica | **v1** — panel de preview de la página seleccionada |
| Texto: insertar, texto de función del circuito, desplazar | **fusionar** en Insertar (desplazar = mover) |
| Editar: propiedades, resumen de propiedades, en forma de tabla | **v1** propiedades (panel); v2 edición en tabla |
| Buscar, mostrar resultados, sincronizar selección | v2 — en v1 basta la paleta de comandos y el filtro de navegador |

→ **No habrá pestaña "Inicio"**: su papel lo cumple una barra de herramientas corta y la
paleta de comandos (§3.3).

### 2.3 Insertar

| Elemento | Uso real | Veredicto |
|---|---|---|
| Navegador de macros | Útil | **v1** |
| Símbolos (galería: esquinas, nodos T, cruces, interrupciones...) | Útil | **v1** — son las piezas de autoconexión |
| Medios de explotación (caja de aparato, PLC, estructura, definición de artículo...) | Redundante con el centro de inserción | **fusionar** en el centro de inserción |
| Cables / conexiones, Bornes, Conectores | Redundante | **fusionar** en el centro de inserción |
| Texto | Repite Inicio | **v1** una sola vez |
| Galería de gráficos (líneas, polilínea, rectángulo, arcos, círculo, elipse, spline) | Útil | **v1** |
| Acotación | Útil | **v1** lineal; resto v2 |
| Externo, Ver, Construcción de armarios | Redundante | **descartar** de este menú |

### 2.4 Editar

| Grupo | Veredicto |
|---|---|
| Portapapeles | **v1** (único sitio) |
| Gráfico: mover, girar, reflejar, escalar, estirar, longitud, cortar, redondear, achaflanar, duplicar | **v1** mover/girar/reflejar/duplicar/estirar; v2 el resto |
| Agrupar / desagrupar | **v1** |
| Bloque (formar, resolver, convertir) | **descartar** — herencia del mundo DWG |
| Organizar: alinear en retícula, alinear, distribuir, primer/segundo plano | **v1** alinear a rejilla y orden; v2 distribuir |
| Protección | **descartar** (bloqueo de objeto vía propiedad si hace falta) |
| Edición en conjunto (varios objetos) | **v1** — el panel de propiedades admite selección múltiple |
| Opciones: edición directa, coordenadas, ancho de paso, conexión inteligente | **v1** coordenadas y conexión inteligente; descartar "desplazamiento del punto de referencia" y "editar segundo plano" |

### 2.5 Ver

| Elemento | Veredicto |
|---|---|
| Abrir navegador (lista de ~20 navegadores) | **v1** — menú Ver → Paneles |
| Seguimiento de red / potencial / señal | v1 red y potencial (resaltado), señal v2 |
| Puntos de inserción | **v1** |
| Elementos invisibles | **v1** |
| Circuitos, marcas de revisión, destinos externos, textos vacíos, conexiones de cable, designaciones de punto de conexión | v2 si se piden |

### 2.6 Medios de explotación (dispositivos)

| Elemento | Uso real | Veredicto |
|---|---|---|
| Navegador de dispositivos | Sí | **v1** |
| Lista de materiales | Sí | v2 (depende de artículos) |
| Numerar, Extras | Nunca | v2 numerar (otros lo usan) |
| PLC (todo) | Nunca | v3 |
| Bornes (navegador, numerar, regleta) | Nunca (personal) | v2 — mucha gente sí lo usa |
| Conectores | Nunca | v2 |
| Construcción de armarios 2D | **Útil** | v2 — anotar como prioridad alta de v2 |

### 2.7 Conexiones

| Elemento | Uso real | Veredicto |
|---|---|---|
| Navegador de cables | Casi nunca | v2 |
| Numerar cables / designar hilos | Personal no (lo resuelve con macros), muchos sí | v2 |
| Editar / volver a asignar | Nunca | v2 bajo demanda |
| Navegador de conexiones, de potenciales | Nunca | v2 (el *seguimiento* de v1 cubre la consulta) |
| Navegador de puntos de interrupción | **Sí** | **v1** |
| Árbol de cables | Nunca | descartar / v3 |

### 2.8 Herramientas

| Elemento | Veredicto |
|---|---|
| Comprobación: Mensajes, Comprobar | **v1** panel de mensajes; comprobaciones v2 |
| Evaluaciones: Generar, Actualizar | v2 (reportes); resto del grupo descartar |
| Gestión: identificación de estructura, capas | **v1** |
| Gestión: distribuciones de propiedades, propiedades | **descartar** — "demasiado complejo" |
| Traducción (traducir, diccionario, exportar pendientes, importar textos, eliminar) | v3 |
| Especificaciones, Sincronización, Comentarios PDF | **descartar** |

### 2.9 Planificación previa

Nunca usado → **descartar** (coincide con el mapa de dominio).

### 2.10 Datos básicos (datos maestros)

| Elemento | Uso real | Veredicto |
|---|---|---|
| Artículos: todo (gestión, sincronizar, actualizar, autocompletar) | **Sí** | v2 (núcleo de v2) |
| Macros: navegador | Sí | **v1** |
| Macros: crear, colección, generar automáticamente | — | **v1** crear macro desde selección (también en menú contextual) |
| Sincronización: sincronizar/actualizar proyecto + datos básicos del sistema | Útil, pero en un solo sitio | v2, **un solo comando** "Sincronizar datos maestros" |
| Cajetines / formularios (crear, modificar) | **Necesario** | **v1** cajetín/marco; formularios de reporte v2 |
| Biblioteca de símbolos (crear biblioteca) / Símbolo (editar) | **Necesario** | **v1** editor de símbolos simple |
| Contorno, imagen de perforación, resto | Nunca | descartar |

### 2.11 Pestañas propias de la instalación

*Ampliaciones*, las pestañas de herramientas internas de cada empresa y *EPLAN Cloud* →
**descartar**.
Las herramientas de empresa se cubrirán con plugins propios (v3), no con pestañas.

---

## 3. Patrones de interacción que se conservan (y cómo se simplifican)

### 3.1 Paneles anclables
Cualquier panel se acopla a un borde, se apila en pestañas, flota o se fija/oculta
automáticamente, y **recuerda su última posición**. La disposición se guarda por usuario.
(`egui_dock` cubre esto.)

### 3.2 Navegador genérico
Un único componente para páginas, macros, dispositivos, puntos de interrupción, etc.:
- **Vista árbol / lista** (lista = tabla con columnas por propiedad, configurables).
- **Filtro rápido** (texto, siempre visible) + **esquemas de filtro guardados**
  (criterio · operador · valor · negar). Un esquema por defecto razonable: "sin
  evaluaciones" para páginas.
- Doble clic abre/salta; selección sincronizada con el canvas.
- Menú contextual propio del tipo de objeto.

### 3.3 Paleta de comandos
EPLAN ya tiene "Buscar comando". En open-ecad es un complemento de la cinta (`Ctrl+K`):
todo comando existe en la paleta, aunque no esté en la cinta. Eso permite una cinta corta
sin perder funciones.

### 3.3 bis Cinta (decidido 2026-10-27)
Se adopta la **cinta** de los CAD de escritorio, en vez de menús clásicos: pestañas
(*Editar, Ver, Insertar, Proyecto, Herramientas, Datos, Ayuda*), grupos con nombre, un botón
grande por grupo y el resto pequeños en columnas de tres; los submenús pasan a botones
desplegables. *Archivo* es un desplegable destacado, no una pantalla completa. Barra de
acceso rápido (guardar, deshacer, rehacer) a la izquierda de las pestañas. Clic en la pestaña
activa (o doble clic) pliega la cinta. Tema oscuro por defecto.

### 3.4 Menú contextual del canvas
Se conserva el patrón observado, reducido:
- **Sin objeto**: repetir último comando, deshacer/rehacer, insertar (gráfico, acotación,
  texto, imagen), pegar, zoom ventana, encuadre de página.
- **Con objeto**: repetir, deshacer, copiar, pegar, duplicar, mover, borrar, crear macro
  desde selección, *ir a* (pareja / otras representaciones), seleccionar los del mismo tipo,
  propiedades.

### 3.5 Propiedades en panel, no en modal
El diálogo de propiedades (pestañas: datos, formato...) se convierte en un **panel lateral
fijo** que muestra la selección actual (simple o múltiple). Insertar un texto = escribir
directamente en el canvas y ajustar en el panel.

### 3.6 Atajos de una tecla
Se observan atajos de una letra (T texto, D duplicar, V mover, F ir a pareja, Z zoom) y
"repetir último comando". Se adoptan como idea (los atajos concretos los decide open-ecad,
configurables).

---

## 4. Esqueleto de la UI v1 (propuesta)

### 4.1 Ventana

```
┌────────────────────────────────────────────────────────────────────────────┐
│ 💾↶↷ [Archivo] Editar Ver Insertar Proyecto Herram. Datos  [Buscar… Ctrl+K]│  ← pestañas
│ [⟲]  ⎘ Copiar  │ [✥]  ↻ Girar   │ [▦]  ...                                  │  ← cinta
│ Deshacer       │ Mover          │                                           │
│   Historial    │  Transformar   │  Organizar                                │
├──────────────┬──────────────────────────────────────────┬──────────────────┤
│ PROYECTO     │  [=A1+CAB/3 ×] [=A1+CAB/4 ×]             │ INSERCIÓN        │
│ ▸ Páginas    │                                          │  Símbolos │Macros│
│   (árbol/    │                                          │  [buscar…]       │
│    lista)    │              CANVAS                      │  ▸ biblioteca    │
│              │      (página con marco y rejilla)        │    ▸ grupo       │
│──────────────│                                          │──────────────────│
│ PREVIEW      │                                          │ PROPIEDADES      │
│ (miniatura)  │                                          │  (selección)     │
├──────────────┴──────────────────────────────────────────┴──────────────────┤
│ MENSAJES · PUNTOS DE INTERRUPCIÓN · DISPOSITIVOS     (pestañas, plegado)   │
├────────────────────────────────────────────────────────────────────────────┤
│ X 120,00 Y 80,00 mm  │ rejilla 1 mm │ snap ● orto ○ │ 1:1 │ tarea en curso  │  ← estado
└────────────────────────────────────────────────────────────────────────────┘
```

**El área del plano es fija** (decidido 2026-10-27): es el cuerpo del programa, no un panel;
no se mueve ni se cierra, y muestra **una sola página**: un clic en el navegador sustituye
la página mostrada en vez de abrir otra pestaña. Los paneles se acoplan en tres zonas
alrededor (izquierda, derecha, abajo); dentro de cada zona se apilan y reordenan, y se
cambian de zona desde el menú contextual de la pestaña.

### 4.2 Pestañas de la cinta v1

| Pestaña | Contenido |
|---|---|
| **Archivo** | Nuevo · Abrir · Recientes · Guardar · Guardar como · Cerrar · Propiedades del proyecto · Imprimir · Exportar PDF/SVG · Ajustes · Salir |
| **Editar** | Deshacer · Rehacer · Cortar · Copiar · Pegar · Duplicar · Borrar · Mover · Girar · Reflejar · Estirar · Agrupar/Desagrupar · Alinear a rejilla · Orden (delante/detrás) · Seleccionar mismo tipo |
| **Ver** | Paneles ▸ (todos los navegadores) · Puntos de inserción · Elementos invisibles · Rejilla · Seguimiento de red / potencial · Zoom (ventana, página, 1:1) · Restablecer disposición |
| **Insertar** | Símbolo… · Macro… · Gráfico ▸ (línea, polilínea, rectángulo, círculo, arco, elipse) · Texto · Imagen · Acotación lineal · Punto de interrupción · Definición de potencial · Piezas de conexión ▸ (esquina, nodo T, cruce, corte) |
| **Proyecto** | Nueva página · Propiedades de página · Estructura (identificadores) · Capas · Regenerar conexiones |
| **Herramientas** | Mensajes · (v2: Comprobar, Numerar, Generar reportes) |
| **Datos** | Biblioteca de símbolos · Editar símbolo · Crear macro desde selección · Cajetines / marcos · (v2: Artículos, Sincronizar datos maestros) |

### 4.3 Paneles v1

| Panel | Base en el dominio | Notas |
|---|---|---|
| Páginas | Proyecto → páginas, estructura | Árbol por estructura (`=`, `+`, `&`) y lista; nueva página desde aquí |
| Preview | Página seleccionada | Miniatura sin abrir pestaña |
| Inserción | Símbolos + macros | Sustituye a los grupos redundantes de *Insertar*; búsqueda + árbol + preview |
| Propiedades | Sistema de propiedades (§11 del dominio) | Selección simple y múltiple |
| Mensajes | §14 del dominio | Ligados a objetos; clic = saltar |
| Puntos de interrupción | §6.5 del dominio | Navegador genérico |
| Dispositivos | §4.3 del dominio | Navegador genérico, agrupa funciones por identificador |
| Macros | §9.5 del dominio | Navegador genérico (lista de macros del proyecto) |

### 4.4 Diálogos que sobreviven en v1 (los mínimos)

1. **Nuevo proyecto**: nombre, carpeta, plantilla. Autor y fecha automáticos.
2. **Imprimir / Exportar PDF**: alcance (página actual, selección, todo), copias, preview.
3. **Ajustes**: dos niveles (proyecto / usuario) con búsqueda.
4. **Estructura del proyecto**: editar los identificadores de estructura.
5. **Esquemas de filtro** de los navegadores.
6. **Editores de datos maestros** (símbolo, cajetín): son pestañas de documento en el
   canvas, no diálogos.

Todo lo demás va al panel de propiedades o a avisos no bloqueantes.

### 4.5 Estado sin proyecto
Pantalla de inicio útil en vez de imagen de bienvenida: **recientes**, **nuevo**,
**abrir**, y acceso a bibliotecas (editar símbolos/cajetines no requiere un proyecto de
esquema abierto).

---

## 5. Relación con el código previsto

- Crate `open-ecad-model` (dominio) sin dependencias de UI; la UI solo emite **comandos**.
- Crate `open-ecad-ui` (egui + egui_dock):
  - `shell/` — ventana, menús, barra de herramientas, barra de estado, paleta de comandos.
  - `panels/` — navegador genérico + instancias (páginas, dispositivos, macros, puntos de
    interrupción), propiedades, mensajes, inserción, preview.
  - `canvas/` — render de página, rejilla, snaps, máquina de interacciones (§15 del dominio).
  - `dialogs/` — los seis de §4.4.
- **Registro de comandos** único: cada comando tiene id, nombre, atajo opcional y
  disponibilidad (requiere proyecto, requiere selección...). Menús, barra, menú contextual y
  paleta leen de ese registro.

---

## 6. Dudas abiertas y capturas que faltan

Pedidas al usuario para ir completando el mapa. Marcar con ✔ al resolver.

### Capturas que harían falta (por prioridad para v1)

1. **Centro de inserción de símbolos**: el panel o diálogo de selección de símbolo, con su
   árbol de bibliotecas y la preview. Es el panel central de v1.
2. **Propiedades de una función** (doble clic en un contacto o bobina): todas las pestañas.
   Define el panel de propiedades.
3. **Editor de símbolos**: crear o editar un símbolo, incluidos los puntos de conexión y las
   variantes.
4. **Editor de cajetín o marco**: cómo se colocan los textos de propiedad y la rejilla de
   columnas.
5. **Navegador de puntos de interrupción** y **navegador de dispositivos**, en vista lista.
6. **Inserción de una macro**: lo que ocurre al soltarla (numeración, diálogos, valores de
   marcadores).
7. **Propiedades de página** y **diálogo de nueva página**.
8. **Panel de mensajes** con algún mensaje real.
9. **Administración de capas**.
10. **Ajustes de impresión** (botón "Ajustes" del diálogo de imprimir).

### Preguntas

- ✔ **¿Cinta o menús clásicos?** Cinta, con aspecto de CAD de escritorio (§3.3 bis).
- ✔ **¿Tema oscuro por defecto?** Sí.
- **Idioma**: ¿interfaz solo en español al principio, o español e inglés desde v1?
- **Ajustes**: ¿qué ajustes cambias de verdad? En los de usuario y proyecto hay cientos.
  Con una lista de los 10–20 que tocas basta para el v1.
- **Macros como sustituto de numerar cables**: ¿cómo lo haces exactamente? Puede ser un
  patrón que open-ecad debería facilitar (marcadores con valores).
- **Construcción de armarios 2D**: lo marcas como útil. ¿Lo usas para el layout del armario
  con artículos reales? ¿Debería subir a v1 tardío?
- **Atajos**: ¿cuáles usas a diario? Así sabemos cuáles mantener por memoria muscular.
