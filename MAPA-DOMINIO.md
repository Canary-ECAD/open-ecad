# open-ecad — Mapa del dominio (derivado clean-room de la documentación pública de la API EPLAN P8)

> **Metodología y cobertura.** Este mapa se derivó de dos lecturas de la documentación
> pública de eplan.help:
>
> 1. **v1 del mapa** — 100% del *User Guide* de P8 2026 (105 documentos conceptuales) e
>    inventario estructural del *API Reference* completo (17.180 documentos: 706 clases en
>    ~26 namespaces, 98 actions, 40 convertidores XML, 13 eventos, 5 interacciones).
> 2. **v2 del mapa (esta versión)** — lectura miembro a miembro del *API Reference* de P8 2027
>    (1.299 documentos agrupados por tipo): clases, propiedades, métodos, enumeraciones con
>    sus valores y los IDs numéricos de propiedad. Es la misma superficie pública que muestra
>    el SDK (`Eplan.EplApi.*.dll`) en un IDE; no se ha descompilado código ni se han mirado
>    tipos internos.
>
> Todo está descrito **en palabras propias** como especificación funcional: conceptos,
> relaciones, reglas y vocabulario del dominio. Se citan nombres de clases de EPLAN solo como
> referencia para que quien lea la documentación oficial pueda orientarse; los **IDs numéricos
> de propiedad** se recogen porque son la clave de interoperabilidad con los exports XML.
> Los docs fuente viven en `_docs-eplan/` (gitignored — no se distribuyen con el proyecto).

---

## 1. El panorama: namespaces y su peso

| Namespace (concepto) | Clases | Veredicto para open-ecad |
|---|---|---|
| `DataModel` — proyecto, páginas, funciones, conexiones, artículos | 274 | **NÚCLEO v1** |
| `DataModel.EObjects` — bornes, regleteros, conectores, PLC, cables, pantallas | 14 | **NÚCLEO v1/v2** |
| `DataModel.Graphics` — primitivas gráficas (línea, arco, texto, pen, capas...) | 52 | **NÚCLEO v1** |
| `MasterData` + `DataModel.MasterData` — artículos, símbolos, macros, marcos | 101 | Núcleo parcial (símbolos/macros v1, artículos v2) |
| `EServices` (+`.Ged`) — verificaciones, mensajes, interacciones del editor | 34 | **NÚCLEO v1** (interacciones) / v2 (verificaciones) |
| `HEServices` — generación, numeración, reportes, búsqueda, macros, sincronización | 49 | v1 parcial / v2 |
| `DataModel.Revisions` — comparación de proyectos | 4 | v3 |
| `DataModel.Topology` — segmentos de ruteo de cables | 3 | Fuera de alcance v1 |
| `DataModel.E3D` — Pro Panel: armarios, carriles, ruteo 3D | 65 | **Fuera de alcance** (v3+ si acaso) |
| `DataModel.Planning` — Pre-planning (lazos PCT, segmentos) | 19 | Fuera de alcance |
| `ApplicationFramework`, `Gui`, `Scripting`, `Remoting`, `RemoteClient/Server`, `Starter` | ~60 | **Ruido** — capa de extensibilidad de EPLAN, no dominio |
| `Base` — propiedades, settings, multilang, enumeraciones globales | 41 | Conceptos sí, API no |

**La lectura clave:** de las 706 clases, el dominio eléctrico real cabe en ~30 entidades.
El resto es (a) listas de propiedades autogeneradas, (b) 3D, (c) infraestructura de plugins.

---

## 2. Decisiones de diseño que importan (resumen ejecutivo)

1. **Función ≠ Símbolo ≠ Dispositivo.** La función es la unidad lógica; el símbolo colocado,
   su dibujo; el dispositivo *emerge* de las funciones que comparten identificador (§4).
2. **Una función lógica, muchas representaciones.** El mismo contacto aparece en multifilar,
   unifilar, vista general, montaje en armario... Cada aparición es un objeto colocado; un
   "merge" los agrupa en una sola función lógica (§4.4). Es la decisión estructural más
   cara de cambiar después: hay que modelarla desde v1 aunque v1 solo dibuje multifilar.
3. **Identificación estructurada IEC 81346** con varios niveles (`==`, `=`, `++`, `+`, `#`,
   `&`, `-`) y un sistema de nombre *completo* vs *visible* (§5).
4. **Las conexiones se calculan, no se dibujan.** Topología → conexiones → redes →
   potenciales → señales (§6).
5. **Plantillas de función en el artículo.** Un artículo declara "qué funciones tiene el
   aparato" (bobina + 4 contactos conmutados); colocar un símbolo *cubre* una plantilla. Así
   se sabe qué contactos quedan libres (§9.3).
6. **Sistema de propiedades genérico** con IDs numéricos estables, indexadas desde 1,
   multilingües, con propiedades de usuario (§11).
7. **Transacciones anidables + undo** como primitiva del modelo; el motor de conexiones
   puede suspenderse durante operaciones masivas (§13).

---

## 3. Jerarquía de objetos (cómo está organizado el modelo)

Todo objeto persistente desciende de una raíz común (*objeto almacenable*) que aporta:
identificador estable serializable a texto, proyecto al que pertenece, validez (no borrado),
estado transitorio (creado pero aún no colocado), bloqueo, solo-lectura, lista de propiedades
y **objetos con referencia cruzada** (los que comparten nombre).

```
ObjetoAlmacenable
├── Proyecto
├── Artículo · ReferenciaDeArtículo (→ AccesorioDeRegletero) · ReferenciaFusionada
├── Conexión (→ ConexiónRuteada [topología], Conexión3D)
├── DefiniciónDeFunción · BibliotecaDeDefiniciones
├── Símbolo · BibliotecaDeSímbolos
├── Ubicación (nodo del árbol de estructura: +A, +A.B ...)
├── FunciónFusionada · ConexiónFusionada        ← vistas agregadas, no persistentes
├── Opción · GrupoDeOpciones · FragmentoDeOpción  ← variantes de proyecto
├── BloqueDeReporte · EntradaDeListaDeDispositivos · EntradaPLC-IO
├── PatrónDePuntosDeConexión
└── Colocación  (= todo lo que se puede poner en una página)
    ├── Grupo
    │   ├── DocumentoBase → Página, MarcoDePágina
    │   ├── VarianteDeSímbolo        (un símbolo es, internamente, un grupo de gráficos)
    │   ├── GrupoDeDispositivo · GrupoProtegido · GrupoDeCotas · ReferenciaDeReporte
    ├── Bloque (→ ParteDeVista)
    ├── EntradaDeMacro
    ├── ColocaciónGráfica → Línea, Arco, Rectángulo, Polilínea, Spline, Texto,
    │                       TextoDeRuta, TextoDePropiedad, Imagen, Hipervínculo, QR,
    │                       Cota, Pantalla(gráfica), Marcador(placeholder)
    └── ReferenciaDeSímbolo   (= símbolo colocado con significado lógico)
        ├── FunciónBase  (= todo lo que tiene identificador de dispositivo)
        │   ├── Función
        │   │   ├── CajaNegra · PuntoDeDefiniciónDeConexión
        │   │   ├── Borne (→ BorneDeBus) · Regletero
        │   │   ├── Conector · RegleteroDeConectores
        │   │   ├── PLC · Cable · Pantalla · Mazo · UnidadDeCable
        │   │   └── Segmento (topología)
        │   ├── PuntoDeInterrupción
        │   └── CajaDeEstructura (location box)
        ├── DefiniciónDePotencial (→ DefiniciónDeRed, DefiniciónDeTubería)
        ├── LíneaDeConexiónDinámica · CajaDeMacro
        └── ConectorDeHaz · InterconectorDeHaz · MarcaDeObjetoBorrado
```

**Lectura para open-ecad.** EPLAN usa herencia profunda donde casi todo es "una función con
una categoría". En Rust conviene lo contrario: una `Function` con un `category` (enum) y datos
de extensión por tipo (`TerminalData`, `PlcData`, `CableData`...), más un trait de
"colocable". La herencia de EPLAN sirve como **checklist de qué datos necesita cada tipo**, no
como plantilla de diseño.

---

## 4. Función, símbolo y dispositivo (en detalle)

### 4.1 Función
Unidad lógica: un contacto, una bobina, un borne, un pin de PLC, un motor. Atributos clave:
- **Categoría** (§9.1) y **definición de función** (qué es y cuántos pines tiene).
- **¿Es función principal?** Un dispositivo tiene una función principal (la que lleva los
  datos de artículo; ej. la bobina) y funciones **auxiliares/subfunciones** (los contactos).
- **Función padre:** para una auxiliar, la principal con el mismo identificador; para un
  sub-borne, el borne principal del borne multinivel.
- **Funciones anidadas:** las que tienen un identificador subordinado colgando del del padre.
- **Pines** (puntos de conexión lógicos, §6.1) y su mapeo a los puntos gráficos del símbolo
  (el mapeo por defecto se puede alterar y restablecer).
- **Tipo de representación** en que está dibujada (§4.4).
- **Imagen de contactos** (§7.3) y si la función **separa potenciales**.
- **Dispositivo fijo:** bloquea cambios de artículo.
- **Protección contra escritura** con motivos acumulables (manual, gestión de subproyectos,
  espacio de diseño, pre-planning) y pausa temporal.
- Puede existir **sin colocar** (función no ubicada en ninguna página) — p. ej. la reserva de
  un aparato importado de una lista de dispositivos.

### 4.2 Símbolo colocado
La parte gráfica: variante de símbolo usada, ubicación, conexiones gráficas (líneas de
autoconexión que tocan sus pines, incluidos nodos T y esquinas), **área lógica** (rectángulo o
polilínea sin arcos que define qué abarca el símbolo; sirve para pantallas, cajas y selección),
**textos de propiedad** asociados y esquema de disposición de esos textos, y el **modo de
cableado** (variante gráfica de "cableado en punto" vs "cableado a destino").

### 4.3 Dispositivo
No hay tabla de dispositivos: el dispositivo es el conjunto de funciones con el mismo
**identificador identificativo** (propiedad 20005). Operaciones de dispositivo: renombrar
todas sus funciones a la vez, bloquearlo entero, convertir una auxiliar en principal,
actualizarlo desde el artículo, crear funciones según un patrón de numeración.

### 4.4 Representaciones múltiples (decisión estructural)
Una misma función lógica puede tener varias colocaciones, cada una en una página de un
tipo distinto: **multifilar**, **unifilar**, **vista general**, **montaje de armario 2D/3D**,
**referencia cruzada de pareja** (p. ej. la parte eléctrica de un guardamotor dibujada aparte),
**funcional**, **P&ID**, **fluídica**, **topología**. Cada colocación es un objeto, y una
**función fusionada** agrupa todas las colocaciones que representan la misma parte funcional
del dispositivo, con una lista de propiedades "fusionada" (ganadora = la primera). Lo mismo
para conexiones (**conexión fusionada**). Hay servicios de **sincronización** entre
representaciones (multifilar → unifilar/vista general, y viceversa).

El tipo de página decide qué cuenta: hay tipos de documento *lógicos* (sus funciones
participan en la fusión, en las conexiones y en los navegadores), *gráficos* (los símbolos son
solo dibujo), *de evaluación* (reportes generados) y *externos*.

**Para open-ecad:** `FunctionId` (lógica) ≠ `PlacementId` (aparición). v1 puede dibujar solo
multifilar, pero el modelo debe tener una relación 1:N función→colocaciones.

---

## 5. Identificación: estructura, nombres y numeración

### 5.1 Niveles de estructura (IEC 81346 extendido)
| Nivel | Prefijo | Concepto |
|---|---|---|
| Asignación funcional | `==` | agrupación por función del proceso |
| Función superior (planta) | `=` | instalación / sistema |
| Lugar de instalación | `++` | emplazamiento físico amplio |
| Ubicación (montaje) | `+` | armario, pupitre... |
| Número de función superior | — | numeración adicional de la planta |
| Estructura de usuario | `#` | libre |
| Tipo de documento | `&` | clasificación de documentos (IEC 61355) |
| Producto | `-` | el identificador del aparato |

Cada nivel tiene **identificador principal**, **identificadores anteriores** (jerarquía de
puntos, p. ej. `+A.B.C`), **sub-identificador**, **descripción** multilingüe y una forma
**"visible"**. Las ubicaciones forman un **árbol ordenado** por nivel (nodo padre, hijos,
orden de clasificación, "en uso").

### 5.2 Anatomía del identificador de producto
`[prefijo] letra(s)-de-código contador [subcontador]`, más un **identificador subordinado
(anidado)** con su propio prefijo, letra, contador y subcontador. Además hay variantes que
derivan el contador **de la posición en el esquema**: página, columna, fila, sección,
aplicación y campos suplementarios. Es la numeración "orientada a la ruta" típica de esquemas
alemanes (`-K3.4` = página 3, columna 4).

### 5.3 Nombre completo, identificativo y visible
- **Visible** (lo dibujado): suele ser solo `-K1`.
- **Completo**: el visible más la estructura heredada de la **página**, de las **cajas de
  estructura** y de las **cajas negras** que lo contienen.
- **Identificativo**: el que determina a qué dispositivo pertenece la función.
- Hay un indicador para usar el visible tal cual como completo (sin evaluar herencias).
- Una función **sin nombre visible** toma el de una "función que da nombre" cercana, según
  una **dirección de búsqueda** configurable (estándar o alternativa; el marco de página
  puede restringirla a cajas o a reglas GOST).

El servicio de nombres recalcula *completo ← visible + contexto* y *visible ← completo −
contexto*, elimina estructuras redundantes y mueve identificadores anidados huérfanos al
principal. **Esta evaluación es una regla del dominio**, no de la UI: va en el crate de modelo.

### 5.4 Formatos y sintaxis por proyecto
Cada proyecto define un **formato de nombre por grupo de función** (página, dispositivo,
regletero, conector, PLC, cable, punto de interrupción, caja de estructura, caja negra, barra,
dispositivo mecánico, fluido) — qué niveles de estructura lleva cada uno. Opcionalmente, un
**chequeo de sintaxis** (letras permitidas, caracteres especiales, solo números en planta...).

### 5.5 Páginas
Nombre de página = estructura (los mismos niveles) + **contador** + **subcontador**, con
validación (contador vacío, subcontador incorrecto...). Una página tiene tipo de documento,
**descripción**, **formulario/marco**, **escala**, **tamaño de rejilla** y datos de revisión.

### 5.6 Numeración
- **Numeración online**: se asigna al insertar el símbolo.
- **Renumeración** de identificadores (todo o solo los inválidos con `?`, respetando o no lo
  numerado por PLC, incluyendo o no cables), de **bornes** (secuencia, PE/N, alfanuméricos,
  designaciones repetidas permitidas), de **conectores**, de **páginas** (con subpáginas) y de
  **posiciones de lista de materiales**.
- Al insertar macros: *no modificar*, *numerar*, *numerar con `?`*, *numerar respetando
  prefijo*.

---

## 6. Conexiones, redes, potenciales y señales (el motor)

### 6.1 Pines (puntos de conexión lógicos)
Dos conceptos distintos:
- **Punto gráfico** del símbolo: índice, posición relativa al punto de inserción y
  **dirección** (máscara de bits: arriba=1, derecha=2, abajo=4, izquierda=8).
- **Pin lógico** de la función, mapeado (normalmente) a un punto gráfico, con:
  - **designación** (el "13", "14", "A1"...) — propiedad indexada por pin;
  - **descripción**;
  - **tipo de conexión**: cableado, puente, directa, fibra óptica, radio, no eléctrica,
    funcional, fluido, interna, de proceso, puente en peine;
  - **tipo de potencial**: L, N, PE, PEN, +, −, M, pantalla;
  - **tipo de señal**: sensor, actuador, alimentación de conexión, alimentación de equipo,
    fuente, entradas/salidas digitales y analógicas, bus (fuente, entrada, salida, fin),
    puertos de presión/succión (fluidos);
  - **fuente / destino** (dirección del flujo de señal);
  - **interno / externo** (lado del borne);
  - **máximo de conexiones**, **máximo de puentes**, si admite **cable**;
  - **pin opuesto**: "reenvío de potencial" dentro de la función o entre funciones del mismo
    dispositivo (el potencial que entra por 13 sale por 14);
  - **seguridad intrínseca posible**; **seguimiento PLC**.

### 6.2 Topología gráfica → conexiones
- El usuario dibuja símbolos; las **líneas de autoconexión** salen de los pines alineados.
- Piezas de enrutado como símbolos propios: **esquinas**, **nodos T** (4 orientaciones),
  **cruce**, **puente**, y un símbolo de **corte de autoconexión**.
- Las líneas de autoconexión tienen su propia capa, color, tipo de línea, grosor y longitud
  de patrón (propiedades 31015–31018).
- Solo algunos tipos de página **admiten autoconexión** y solo algunos son **relevantes para
  generar conexiones**.
- El resultado es una **conexión** por tramo lógico: función+pin de inicio → función+pin de
  final, con la lista de colocaciones que atraviesa (nodos T, esquinas, inicio y fin).
- Una conexión que cruza **puntos de interrupción** se compone de **subconexiones**.
- **Clase de cable** de la conexión: conexión individual, cable, conducto, barra de fases, línea.
- Las conexiones tienen **representación** (multifilar, unifilar...) igual que las funciones.
- Se puede **suspender la actualización de conexiones** durante una edición masiva y
  regenerarlas después (por páginas, solo actualizar o reconstruir todo).

### 6.3 Propiedades de la conexión: puntos de definición
Los datos del hilo (designación, sección, color/número, longitud, tipo, procesado de puntas,
longitud de pelado, manguito) no se dibujan en la línea: se colocan **puntos de definición de
conexión** sobre ella. Si hay varios con valores distintos, hay conflicto que resolver. Cambiar
uno no actualiza la conexión hasta regenerar. Servicios: colocar puntos de definición
automáticamente, **designar hilos** (con política de sobrescritura y evitando duplicados),
alinear/formatear, borrar designaciones, intercambiar fuente y destino, **exportar datos de
fabricación de hilos** para máquinas.

### 6.4 Tres niveles de "lo que está unido"
El servicio de conexiones distingue tres búsquedas, que son tres conceptos del dominio:
1. **Red**: conexiones unidas galvánicamente, solo por hilos.
2. **Potencial**: además, se propaga a través de funciones que *no separan* potencial
   (bornes, puntos de interrupción, pines con reenvío) y se corta en las que lo separan.
3. **Señal**: lo que transporta la misma señal en el esquema (atraviesa la separación de
   potencial si la definición de función no aísla la señal).

> Los tres niveles y los indicadores *separa potencial* / *aísla señal* / *pin opuesto* están
> en la API; las reglas exactas de propagación de los puntos 2 y 3 son interpretación y hay
> que confirmarlas observando EPLAN (§19).

**Definiciones de potencial**: puntos colocados sobre la red que le dan nombre, tipo, valor,
frecuencia, nombre de señal, potencial superior y contra-potenciales válidos. Una variante,
la **definición de red**, permite el **cableado por red** (unir pines por nombre sin dibujar
líneas, con *daisy chains* para fijar el orden).

**Para open-ecad:** el grafo de conexiones es un dato derivado e incremental. Hay que diseñar
desde v1: (a) incrementalidad por página, (b) suspensión, (c) los tres niveles de cierre
transitivo, (d) conflicto de puntos de definición como mensaje, no como error.

### 6.5 Puntos de interrupción
Continúan un potencial/red entre páginas: dos (o más) con el mismo nombre se emparejan. Un
punto puede ser **fuente en estrella** (una fuente, varios destinos). Muestran la
**referencia cruzada** a su(s) pareja(s), con separador, formato, número máximo de
referencias y filas/columnas. Se verifican: emparejamiento, nombres duplicados y puntos de
conexión usados.

---

## 7. Páginas, marcos y referencias cruzadas

### 7.1 Tipos de documento
~70 tipos. Agrupados:
- **Lógicos (se dibujan)**: esquema multifilar, unifilar, vista general, fluídica, P&ID,
  funcional, montaje de armario 2D/3D, topología, pre-planning.
- **Gráficos**: gráfico libre, documento externo, portada, modelo 3D en 2D.
- **De evaluación (generados por reportes)**: índice, lista de dispositivos, de piezas,
  resumida, de fabricantes, de montaje; diagramas y vistas de bornes, conectores, cables,
  regleteros; lista de conexiones; vista de potenciales; PLC (tarjetas, diagrama, direcciones);
  resumen de identificadores de estructura; símbolos; formularios; marcos; revisiones;
  opciones; placeholders; leyenda de armario; plan de taladrado; lista de rutas/cables ruteados.
- **Internos del editor de datos maestros**: símbolo, formulario, marco, macro.

### 7.2 Marco de página
Define cabecera, tamaño, orientación y la **rejilla de referencias**: número de columnas y
filas, ancho de columna, alto de fila, valores iniciales, cadenas de designación (alfabéticas
o numéricas), numeración inversa, desplazamiento de la rejilla y si se muestran designaciones
de columna en páginas no lógicas. **Las referencias cruzadas se expresan como página/columna
(o página/columna.fila)** usando esta rejilla — por eso el marco es dato del dominio, no
decoración.

### 7.3 Referencias cruzadas
- Se calculan entre objetos con el **mismo nombre** (bobina ↔ contactos, punto de
  interrupción ↔ pareja, función principal ↔ auxiliares, conector macho ↔ hembra,
  representaciones de pareja).
- **Imagen de contactos**: bajo la bobina o junto al componente, la tabla de contactos del
  dispositivo con la referencia de dónde está cada uno; también contactos *no usados* (de las
  plantillas sin cubrir, §9.3).
- Existe la **referencia cruzada manual** para casos sin relación de nombre.
- Variante de formato para normas NFPA.

### 7.4 Reportes
Un **bloque de reporte** guarda cómo se generó un conjunto de páginas: formulario, tipo de
evaluación, esquemas de filtro y orden (de funciones y de objetos), dispositivos incluidos y
descripción de página automática o manual. Se puede **actualizar**, **borrar** o **congelar**
(las páginas quedan pero ya no se regeneran). También hay **reportes incrustados** y
**formularios dinámicos** dentro de páginas de esquema. Los formularios usan marcadores en
grupos (cabecera, datos, pie de grupo, pie de página) y tipos especiales: tabla de cables,
tabla de hilos, gráfico de puentes (en peine, de hilo, enchufable, conmutable, interno/externo),
subtotales, acumulados y totales.

---

## 8. Objetos eléctricos especializados

### 8.1 Bornes y regleteros
- El **regletero** es la función principal (la "definición de regletero"); los **bornes** son
  sus auxiliares.
- **Borne principal / auxiliar**: el mismo borne puede representarse varias veces.
- **Bornes multinivel**: niveles con sub-bornes; puentes "verticales" entre niveles del mismo
  borne.
- **Bornes distribuidos**: un borne físico repartido en varias representaciones.
- Conexiones del borne por **lado interno / externo** (se puede forzar el lado).
- **Puentes**: internos, externos, en peine (ranuras 3/4/5), de hilo, enchufables, conmutables;
  se modelan como conexiones de tipo puente y se pueden consultar agrupados por puente
  completo o por segmentos.
- Atributos: categoría de borne (p. ej. PE/N), con LED, con adaptador, abierto, bloqueo de
  conmutación, código de orden (lógico y gráfico), permitir designaciones repetidas, nivel,
  posición, formulario de diagrama.
- Estadísticas derivadas del regletero: nº de bornes, PE, N, pantalla, principales, primer y
  último borne. **Accesorios** de regletero (placas finales, separadores) como referencias de
  artículo.

### 8.2 Conectores
Igual patrón: **regletero de conectores** + **conectores** (pines/casquillos) como auxiliares;
referencia cruzada a la **contrapieza** directamente conectada.

### 8.3 PLC y buses
- **Caja PLC** (tarjeta) con **puntos de conexión PLC**: dirección, dirección simbólica, tipo
  de dato, canal, designación de enchufe, rango de señal.
- **Puertos de bus**: sistema de bus (~50: Profinet, Profibus, EtherCAT, EtherNet/IP,
  Modbus/TCP, CAN, AS-i, IO-Link, KNX, DALI...), dirección de bus, maestro/esclavo, red
  física y lógica.
- Atributos de tarjeta: CPU, rack, dirección de inicio, cabecera de bus, alimentación,
  módulo integrado, subdispositivos.
- **Entradas de E/S libres**: se importan de una lista de asignación y se "reservan" sobre
  pines PLC (los datos pasan al pin y la entrada desaparece).
- Direccionamiento/renumeración y export/import a configuradores (AutomationML y formatos de
  fabricante).

### 8.4 Cables, pantallas y mazos
- Un **cable** se dibuja como una **línea de definición de cable** que cruza las conexiones
  que contiene.
- Sus **hilos** vienen de **plantillas** del artículo (color/número, sección); las conexiones
  se asignan a plantillas (y se puede reasignar automáticamente).
- **Pantalla**: función con área lógica que apantalla conexiones; los hilos pueden ser
  "apantallados por" una pantalla y tener índice de par.
- **Generación automática de cables** (crear, numerar, seleccionar tipo según esquemas),
  borrado de lo autogenerado, suma de longitudes.
- Datos derivados: hilos usados/libres/PE/N/pantallas, páginas de diagrama.
- **Mazos** y **unidades de cable** agrupan por nombre de mazo.

### 8.5 Cajas
- **Caja de estructura**: rectángulo que asigna estructura (`=`, `+`...) a lo que contiene.
- **Caja negra**: dispositivo de forma arbitraria; las funciones de dentro le pertenecen; sus
  **conexiones de dispositivo** son pines sueltos.
- **Grupo de dispositivo**, **grupo protegido** (lo genera una macro), **bloque** (gráficos +
  como mucho un símbolo, reutilizable por referencia).

---

## 9. Datos maestros

### 9.1 Categorías de función
El catálogo es un enum numérico estable. Por rangos:

| Rango | Familia |
|---|---|
| 4–23 | conexión (hilo), placa de montaje, colocación de artículo, punto de definición, topología, mazo, unidad de cable, extremo abierto |
| 100–106 | borne, conector, definición de regletero/conector, borne seccionable, accesorio de regletero, base de enchufe |
| 200–204 | bobina, contacto NA, NC, conmutado, circuito de protección |
| 300–301 | punto de conexión PLC, caja PLC |
| 400–401, 430 | cable, pantalla, antena |
| 500–507 | fusible, magnetotérmico, guardamotor, relé de protección, diferencial, interruptor automático, descargador |
| 600–606 | interruptor/pulsador, final de carrera, barrera, inductivo/reed, sensor digital y analógico, nivel |
| 700–701, 800–802 | fuente, generador; transformador, convertidor, rectificador |
| 900–904 | señalización óptica/acústica, instrumento de medida, registrador, display |
| 1000–1002, 1100–1102 | motor, lámpara, calefacción; válvula, freno, embrague |
| 1201–1211 | pasivos y semiconductores: condensador, inductancia, resistencia, diodo, transistor, regulador, acoplador... |
| 1301–1306, 1400 | indefinido, conexiones de dispositivo, caja negra, barras, barra de fases, distribuidor |
| 5000–6201 | fluídica |
| 10000–11306 | ingeniería de procesos |
| 15000–16200 | mecánica de armario, áreas, ruteo, vistas (3D) |

**Para open-ecad v1:** basta el subconjunto 4–1400; reservar los rangos para no colisionar.

### 9.2 Definición de función
Árbol **oficio → área → categoría → grupo → definición**, con id y grupo. Define: nº de pines
(fijo o variable) y sus atributos (§6.1), si es función principal, si conecta redes, si es de
seguridad, si aísla señal, **letras de código por norma** (varias normas) y el **símbolo base**
recomendado. Va asignada al símbolo y se copia a la función al colocarlo; el artículo también
la lleva (así se pueden elegir aparatos compatibles).

### 9.3 Artículos
- Clave: **nº de pieza + variante**. Viven en la base de piezas del sistema; al usarse se
  copian al proyecto (proyecto autónomo) y se **sincronizan** en ambos sentidos.
- **Referencia de artículo** desde función, conexión o proyecto, con cantidad y posición. Es
  un objeto "offline": se edita y luego se escribe en el dueño. Si la referencia deja vacío
  un dato, se toma del artículo.
- **Conjuntos** (assemblies): el artículo se compone de otros, recursivamente. **Módulos**:
  subpartes con su propio identificador. **Accesorios**: necesarios u opcionales, o listas de
  accesorios. Una **referencia fusionada** agrupa referencias de un conjunto, de un módulo o
  de todas las representaciones de una función.
- **Plantillas de función**: el artículo declara qué funciones tiene el aparato (de un
  dispositivo o de los hilos de un cable). Una plantilla está **sin cubrir** hasta que se
  coloca una función que la cubre. Las sin cubrir se pueden "hacer persistentes" (crear la
  función sin colocarla). **De aquí salen la imagen de contactos completa y el aviso de
  contactos de más.**
- **Patrón de puntos de conexión** del artículo, por punto: tipo de borna (tornillo simple
  o doble, jaula, IDC, tornillo, puntera, enchufe, soldadura, rosca, racor...), tamaño,
  sección mín./máx. (mm² y AWG), par de apriete mín./máx., longitud de pelado, nº máximo de
  hilos, doble manguito, dirección de conexión y de ruteo, lado interno/externo, nivel.
- El artículo puede llevar una **macro** asociada: "insertar dispositivo desde pieza" coloca
  esa macro.
- **Lista de dispositivos** (planificación): aparatos previstos aún no colocados.

### 9.4 Símbolos
- **Biblioteca** (nombre) → **símbolo** (número/nombre único en la biblioteca) →
  **variantes** (A–H: las orientaciones/espejos), cada una con sus puntos de conexión y su
  disposición de textos por defecto.
- **Tipo de símbolo**: función normal, esquina, punto de interrupción, nodos T, cruce,
  puente, corte de autoconexión, caja de estructura, caja negra, texto de función, línea de
  definición de cable, pantalla, definición de potencial/red/conexión, borne de potencial,
  caja de macro, marcador, objetos de proceso y de ruteo...
- Metadatos: función principal, conecta redes, separa potencial, función de seguridad,
  intrínsecamente seguro, "impedir nueva colocación", tipo de representación del símbolo,
  etiquetas, versión, autoría.
- La biblioteca de símbolos del sistema es de solo lectura desde el proyecto.

### 9.5 Macros
- **De ventana** (fragmento), **de símbolo** (fragmento pequeño que se comporta como símbolo)
  y **de página** (páginas enteras).
- Un archivo de macro contiene **varias representaciones** (multifilar, unifilar, vista
  general, referencia de pareja, gráfico, colocación de artículo 2D/3D, P&ID, fluídica,
  funcional, planificación, cableado) **× variantes**. Al insertar, la página elige la
  representación por su tipo.
- **Caja de macro**: define el contorno, el punto de inserción, el nombre/variante, si se
  genera grupo protegido, si se respetan las designaciones de pines y su uso (definición,
  referencia, subordinada).
- **Marcadores (placeholders)**: tabla de **variables × conjuntos de valores** (registros);
  aplicar un conjunto ("Motor 0,75 kW") escribe propiedades en los objetos referenciados.
- Inserción con coordenadas absolutas o relativas y modo de numeración (§5.6). Las macros de
  página pueden sobrescribir páginas existentes.

---

## 10. Gráficos

- Primitivas: línea, arco (varios tipos), círculo/elipse vía arco, rectángulo, polilínea
  (con relleno), spline, texto, **texto de ruta** (texto de función que se aplica a la
  columna), imagen, hipervínculo, QR, cotas (lineal, continua, incremental, circular),
  pantalla gráfica, comentarios importados de PDF.
- **Pen**: color (índice 0–255 de una tabla, RGB o "por capa"), grosor, tipo de línea, factor
  de patrón, extremo de línea. Un valor centinela significa "tomar de la capa".
- **Capas**: visibilidad, impresión, bloqueo, fondo, 3D, escalable, transparencia, pen por
  defecto, altura/rotación/alineación de texto, interlineado. La visibilidad de un objeto es
  *visible*, *invisible* o *por capa*.
- **Orden de dibujo** explícito dentro de un grupo.
- **Texto**: fuente, altura, rotación, justificación (9 puntos + línea base), **caja de
  alineación** (ancho/alto fijo, ajuste, sin guiones), modo de idioma (proyecto, fijo...),
  traducción automática sí/no, línea de referencia.
- **Texto de propiedad**: muestra el valor de una propiedad del objeto padre (o, por
  relación, de un objeto relacionado; o de la página/proyecto). Se puede "acoplar" al símbolo
  y se organiza en **esquemas de disposición** guardados por variante de símbolo.

---

## 11. Sistema de propiedades

- **Tipos**: booleano, entero largo, real, texto, **texto multilingüe**, punto, fecha/hora,
  coordenada y **valor con unidad**.
- **Indexadas**: una propiedad puede tener índices **1..MaxIndex** (los índices empiezan en 1):
  designaciones de pin, datos de artículos por posición... Se consulta qué índices se usan y
  el último usado.
- Por definición de propiedad: solo lectura, forma parte del nombre, obsoleta, interna, con
  licencia, límites inferior/superior, validación y **cadena de visualización** formateada.
- **Propiedades de usuario**: nombre identificativo, nombre mostrado, categoría (~35:
  general, datos de cable, PLC, pieza, revisión...), a qué objetos aplica (proyecto, página,
  función, conexión, referencia de artículo, punto de interrupción), ayuda de entrada (libre,
  lista de selección, lista autoextensible), **grupo de unidades** (~40: longitud, corriente,
  tensión, potencia...), solo lectura en GUI, valores por defecto.
- **Propiedades derivadas**: muchas (contadores de un regletero, referencias cruzadas, nombre
  completo...) se calculan y son solo lectura. Conviene marcarlas como tales en open-ecad y
  no guardarlas.

La tabla de IDs útiles para interoperar está en el **Anexo A**.

---

## 12. Proyecto, opciones, revisiones y colaboración

- **Proyecto**: tipo (*esquema* o *proyecto de macros*), datos de cabecera (cliente, empresa,
  descripción, nº de pedido, GUID), **settings** propios (árbol con defaults y tipos), tabla
  de capas, bibliotecas de símbolos usadas, biblioteca de definiciones de función, propiedades
  de usuario y usuarios que lo tienen abierto.
- Operaciones: crear, abrir (estándar, solo lectura, exclusivo), copiar (todo, sin evaluación,
  solo cabecera, solo lógica, instantánea), actualizar versión de base de datos, backup/restore.
- **Opciones** (variantes de producto): grupos de opciones con opciones activas/inactivas;
  cada opción tiene **fragmentos** que contienen objetos. Permite un esquema con equipamiento
  opcional.
- **Revisiones**: comparación con un proyecto de referencia → entradas *añadido / borrado /
  cambiado / sin cambios*, con detalle por propiedad (valor viejo y nuevo); marcas de objetos
  borrados; datos de revisión por página (revisado, aprobado, motivo, borrador, marca de agua).
- **Secciones de trabajo** y **subproyectos**: exportar una parte del proyecto, trabajarla
  aparte y reinsertarla.
- **Bloqueo** por objeto/dispositivo/página/proyecto y **protección contra escritura**.

---

## 13. Infraestructura del modelo digna de imitar

- **Transacciones anidables** todo-o-nada: una interna cuenta como una operación de la externa.
- **Undo por pasos** con descripción. Lo que toca el sistema de archivos (abrir/cerrar,
  restaurar...) queda fuera del undo.
- **Puntos de seguridad**: grabar cambios y hacer commit o rollback.
- **Suspensión de la actualización de conexiones** durante operaciones masivas.
- **Eventos de cambio** del proyecto y de "antes de borrar".
- **Consulta del grafo**: buscador por tipo + filtros (página, categoría, colocado, nombre
  exacto o prefijo, lista de propiedades a coincidir, filtro personalizado) y búsqueda de
  texto con ámbitos (identificadores, todas las propiedades, textos, datos de página/proyecto,
  páginas lógicas/gráficas/de reporte, sin colocar, marcadores).
- **Navegadores** como vistas derivadas: piezas, dispositivos, páginas, regleteros, conectores,
  PLC, cables, conexiones, puntos de interrupción, potenciales, símbolos, topología.

---

## 14. Verificación y mensajes

- Un **mensaje** tiene id, **región** (001 bornes, 002 conectores, 003 cables, 004 PLC/bus,
  005 conexiones, 007 dispositivos, 008 idiomas, 010 referencias cruzadas, 011 puntos de
  interrupción, 013 datos de piezas, 014 macros, 015 reportes, 016 cajas negras, 017
  identificadores...), **clasificación** (error / aviso / indicación), texto con parámetros,
  hasta dos objetos asociados, pieza asociada, autor, fecha, versión y marca de **resuelto**.
- **Origen**: *online* (implícito al editar), *offline* (en una verificación explícita) o
  *explícito* (lo emite otro módulo, p. ej. generación de cables).
- Cada verificación tiene un **permiso máximo** y un **estado** elegido en un esquema:
  no ejecutar · solo offline · online+offline · **restrictivo** (impide/revierte el cambio).
- Tipos de verificación según qué recorren: funciones (con filtro de categoría y acceso a
  "todas las funciones con el mismo nombre"), potenciales, puntos de interrupción, piezas.
- Ámbitos: proyecto, páginas, espacios de instalación, piezas de la base.

---

## 15. Editor: interacciones

El editor gráfico es una **máquina de estados de interacciones con prioridad**: una
interacción nueva de igual o mayor prioridad desactiva la actual hasta que acaba, y puede
haber **sub-interacciones**. Cada interacción recibe eventos (inicio, punto introducido,
selección, movimiento de cursor, carácter, longitud/ángulo tecleados, arrastre, cancelar,
éxito, parada, reactivación) y responde con una **petición** en máscara de bits: *necesito un
punto*, *selección (simple o múltiple, permitir vacía)*, *arrastre*, *resaltar bajo el ratón*,
*longitud*, *ángulo*, *éxito*, *parar*, *reiniciar*, *abortar*, *llamar al comportamiento por
defecto*, *autoconexión*... Además puede dibujar un **cursor** (objeto temporal, goma elástica
de línea o de rectángulo), filtrar qué objetos son seleccionables y escribir en la línea de
estado/comando.

**Snaps**: extremo, punto medio, perpendicular, centro, intersección, punto de inserción,
eje, texto, tangente, intersección virtual y personalizados; también rejilla y modo orto.
Hay entrada de coordenadas por línea de comandos.

Es un buen modelo para el canvas egui de open-ecad: herramientas como estados con un contrato
de eventos → peticiones, no callbacks sueltos.

---

## 16. Servicios del flujo de trabajo

| Servicio | Qué hace para el proyectista | open-ecad |
|---|---|---|
| Interacciones del editor | Insertar símbolo/macro, seleccionar, mover, borrar, capturar puntos (§15) | **v1** |
| Motor de conexiones | Topología → conexiones/redes/potenciales; regenerar por páginas (§6) | **v1** |
| Evaluación de nombres | Completo ↔ visible según página y cajas (§5.3) | **v1** |
| Referencias cruzadas e imagen de contactos | Página/columna vía rejilla del marco (§7) | **v1** |
| Inserción de macros | Representación según página, numeración, marcadores (§9.5) | **v1** |
| Puntos de definición de conexión | Colocar, designar hilos, alinear, borrar (§6.3) | v1 mínimo / v2 |
| Mensajería del proyecto | Bandeja clasificada ligada a objetos (§14) | v1 mínimo |
| Seguimiento | Resaltar red / potencial / señal desde una conexión | v1 |
| Verificaciones | Por dominios, online/offline/restrictivo (§14) | v2 |
| Dispositivos desde pieza | Crear dispositivo + macro, patrones de funciones, lista de dispositivos | v2 |
| Numeración | Identificadores, bornes, conectores, páginas, posiciones de BOM (§5.6) | v2 |
| Reportes | Generar/actualizar/congelar sobre formularios (§7.4) | v2 |
| Cables | Generar, numerar, seleccionar, asignar hilos a plantillas (§8.4) | v2 |
| Sincronización | Entre representaciones y entre proyecto y base de piezas | v2 |
| Búsqueda global | Ámbitos y opciones (§13) | v2 |
| Impresión / export | PDF, imagen, DXF/DWG; import DXF/DWG | v1 (PDF/SVG), v2 (DXF) |
| PLC | Direccionamiento, E/S libres, export a configuradores (§8.3) | v3 |
| Traducción | Diccionario multilingüe del proyecto | v3 |
| Revisiones / comparación | §12 | v3 |
| Backup/restore, subproyectos | §12 | v3 |

---

## 17. El "ruido" de EPLAN que open-ecad NO debe replicar

1. **Toda la capa de extensibilidad**: add-ins .NET, add-ons con `install.xml`, shadow
   copying, scripting C#/VB en caliente, actions con ordinales y parámetros en texto, eventos
   por string, API de ribbon. Un clon moderno usa comandos propios y, si hace falta, plugins
   diseñados de cero.
2. **Remoting/COM/cloud**: servidor gRPC embebido, interop COM, IdentityClient, Data Portal,
   firma de assemblies, licencias.
3. **La granularidad burocrática**: cientos de clases `*PropertyList` autogeneradas. Con un
   sistema de propiedades genérico (§11) todo eso es una sola abstracción.
4. **Herencia profunda por categoría** (§3): Terminal, Plug, PLC, Cable... como subclases de
   Function. En Rust: categoría + datos de extensión.
5. **Objetos "offline" que hay que guardar a mano** (referencias de artículo, posiciones de
   puntos de conexión, propiedades de navegación que devuelven copias). Es una trampa de API
   de EPLAN, no del dominio: en open-ecad toda mutación pasa por comandos sobre el modelo.
6. **Pro Panel (3D)**, **Pre-planning**, **fluídica** y **procesos**: productos aparte.
   Reservar categorías y tipos de representación para no cerrarles la puerta, pero ni una
   línea de código v1.
7. **El doble modo de transacción** (API vs interna, con commits implícitos), los *locking
   steps* obligatorios y los settings en muchos niveles. Una sola primitiva de transacción y
   un árbol de settings plano.

---

## 18. Propuesta de alcance (revisada con este mapa)

### v1 — Esquemático 2D funcional
- Modelo: proyecto → páginas → **funciones lógicas con 1..N colocaciones** + primitivas
  gráficas; identificador IEC 81346 con niveles y evaluación completo↔visible; propiedades
  tipadas/indexadas/multilingües; transacciones con undo/redo.
- Motor de conexiones: autoconexión por topología (esquinas, nodos T, cortes), redes y
  potenciales, puntos de interrupción con referencias cruzadas página/columna.
- Marco de página con rejilla; imagen de contactos simple.
- Librería de símbolos propia (SVG + JSON con puntos de conexión, dirección y variantes A–H)
  con una librería inicial IEC 60617; definiciones de función mínimas (categorías 4–1400).
- Editor: canvas con rejilla y snaps, interacciones como máquina de estados, macros de ventana
  con caja de macro y marcadores.
- Persistencia: formato propio legible (JSON/SQLite); export PDF/SVG.

### v2 — Proyectar de verdad
- Base de artículos (SQLite) con conjuntos, accesorios, patrón de puntos de conexión y
  **plantillas de función**; dispositivos desde pieza.
- Bornes/regleteros (multinivel, puentes), conectores, cables (plantillas de hilos,
  pantallas), puntos de definición de conexión.
- Verificaciones online/offline/restrictivas; renumeración; reportes sobre formularios;
  representaciones unifilar y vista general con sincronización; DXF/DWG; búsqueda global.

### v3 — Ecosistema
- PLC y buses, AutomationML, intercambio con los XML de EPLAN vía IDs de propiedad (Anexo A),
  opciones/variantes, revisiones y comparación, multiusuario (bloqueo), subproyectos,
  traducción, API de plugins propia.

---

## 19. Lo que la API NO revela (hay que observarlo usando EPLAN)

La API describe datos y operaciones, no los algoritmos. Para estas reglas hará falta
**documentar el comportamiento observable** usando EPLAN como usuario (sigue siendo
clean-room):
- Reglas geométricas exactas de la autoconexión (tolerancias, prioridad entre nodos T y
  esquinas, qué pasa con pines enfrentados a distinta altura).
- Algoritmo de "función que da nombre" y su dirección de búsqueda.
- Orden y formato por defecto de las referencias cruzadas e imagen de contactos.
- Cómo se resuelven conflictos entre varios puntos de definición de conexión.
- Reglas de fusión entre representaciones (qué propiedad gana).
- Propagación exacta de potencial y de señal a través de funciones (§6.4).
- Comportamiento exacto de los modos de numeración al insertar macros.

---

## Anexo A — IDs de propiedad para interoperar (selección)

Los exports XML de EPLAN identifican propiedades por número. Descripciones propias.

### Identificación (cualquier objeto con identificador)
| ID | Significado |
|---|---|
| 20000 | Nombre identificativo |
| 20001 | Nombre completo |
| 20002 | Nombre visible |
| 20005 | Identificador de dispositivo identificativo (**define el dispositivo**) |
| 20006 | Identificador completo |
| 20008 | Identificativo sin estructuras de proyecto |
| 20009 | Completo sin estructuras de proyecto |
| 20010 | Identificador mostrado |
| 20003 / 20004 | Identificador superior / subordinado (sin estructuras) |
| 20012 / 20013 / 20014 / 20015 | Prefijo / letra de código / contador / subcontador |
| 20016–20019 | Ídem para el identificador subordinado |
| 20150–20160 | Contador por posición: página, columna, fila, sección, aplicación, campos suplementarios |
| 20051 | Usar el identificador mostrado como completo |
| 20121 | Tipo de representación |
| 1100 / 1120 | Planta `=` (principal / completo) |
| 1200 / 1220 | Ubicación `+` (principal / completo) |
| 1300 / 1320 | Asignación funcional `==` |
| 1400 / 1420 | Lugar de instalación `++` |
| 1600 / 1620 | Estructura de usuario `#` |
| 1700 / 1720 | Número de función superior |
| 1800 / 1820 | Producto |

Para cada nivel, sobre el ID del identificador principal: +20 = completo, +21 = sub-identificador
completo, +22 = identificadores anteriores, +29 = visible, +40 = completo con prefijo,
+50 = descripción (p. ej. ubicación: 1220, 1221, 1222, 1229, 1240, 1250).

### Función
| ID | Significado |
|---|---|
| 20011 | Texto de función |
| 20031 | Texto de función automático |
| 20022 | Designaciones de los pines (indexada por pin) |
| 20023 | Descripciones de los pines (indexada) |
| 20028 / 20029 | Designación / descripción de punto de conexión (bornes) |
| 20026 | Definición de función |
| 20115 / 20116 / 20117 | Categoría / grupo / descripción de la definición |
| 20188 | Categoría/grupo/definición codificados |
| 20122 | Función principal |
| 20123 | Con aislamiento de señal |
| 20166 | Tipo de potencial de los puntos de conexión |
| 20024 | Lugar de montaje (descriptivo) |
| 20025 | Texto de grabado |
| 20027 | Características técnicas |
| 20045 | Observación |
| 20100 / 20102 | Nº de pieza / cantidad (indexadas por referencia de artículo) |
| 20300 | Referencia cruzada (principal/auxiliar) |
| 20302 | Referencia cruzada manual |
| 20303 | Visualización de imagen de contactos |
| 20304 | Referencia cruzada a contrapieza de conector |
| 20220 / 20228 | Borne principal / borne auxiliar |
| 20034 | Nivel (borne multinivel) |
| 20230 | Categoría de borne |
| 20809 | Código de orden del borne/pin |
| 20811 | Permitir designaciones repetidas |
| 20350 / 20351 | Puentes en peine manuales internos / externos |
| 20400 | Dirección PLC |
| 20405 | Tipo de dato PLC |
| 20407 | Designación de canal PLC |
| 20308 | Sistema de bus |
| 20311 | Dirección de bus |
| 20167 | Es CPU |

### Conexión y potencial
| ID | Significado |
|---|---|
| 31011 | Designación de conexión |
| 31009 | Descripción de conexión |
| 31002 / 31059 / 31007 | Sección / unidad / sección con unidad |
| 31004 | Color / número de hilo |
| 31048 | Tipo de hilo |
| 31000 / 31001 / 31003 | Longitud / unidad / longitud con unidad |
| 31019 / 31020 | Fuente / destino |
| 31140 / 31141 | Índice interno/externo en fuente / destino |
| 31051 / 31052 | Procesado de punta en fuente / destino |
| 31053 / 31054 | Sección de manguito en fuente / destino |
| 31055 / 31056 | Longitud de pelado en fuente / destino |
| 31049 / 31050 | Apantallado por / índice de par |
| 31076 | Índice de red |
| 31142 | Asociación (clase de cable) |
| 31015–31018, 31005 | Línea de autoconexión: tipo, grosor, capa, patrón, color |
| 33000 | Nombre de potencial |
| 31006 | Tipo de potencial |
| 33001 | Potencial superior |
| 33003 / 33004 | Valor / frecuencia |
| 33005 | Contra-potenciales válidos |
| 33006 | Nombre de señal |
| 33007 | Nombre de red |

### Punto de interrupción
| ID | Significado |
|---|---|
| 24000 | Destino del punto de interrupción |
| 24300 / 24301 / 24302 | Referencia cruzada configurable / simple / NFPA |
| 24350 | Separador entre punto y referencia |
| 24020 / 24021 | Mostrar todas las referencias / todos los destinos |
| 24809 | Código de orden |
| 24810 | Descripción |
| 20049 / 20050 / 20198 | Mostrar bloque de referencias / filas-columnas / máximo |

### Página y marco
| ID | Significado |
|---|---|
| 11000 | Nombre de página |
| 11009 / 11010 | Nombre completo / identificativo |
| 11011 | Descripción de página |
| 11012 | Contador de página |
| 11015 | Formulario |
| 11017 / 11029 | Tipo de página (texto / numérico) |
| 11044 / 11048 | Escala (relación / valor) |
| 11070–11088 | Revisión de página (autor, fecha, motivo, borrador, revisado, aprobado...) |
| 11003 | Marca de agua de revisión |
| 12005 / 12006 | Nº de columnas / nº de filas del marco |
| 12102 / 12002 | Ancho de columna / alto de fila |
| 12025 / 12026 | Valor inicial de columna / fila |
| 12027 / 12028 | Cadena de designación de columnas / filas |
| 12035 / 12036 | Numeración inversa de columnas / filas |
| 12003 / 12004 | Desplazamiento de la rejilla X / Y |
| 12016 | Orientación / formato de papel |

> Nota: en la documentación de EPLAN los nombres de algunas propiedades del marco llaman
> "filas" a lo que se describe como columnas y viceversa. Antes de mapear, verificar con un
> export real.

### Proyecto
| ID | Significado |
|---|---|
| 10000 | Nombre de proyecto |
| 10011 | Descripción del proyecto |
| 10013 | Nº de pedido |
| 10015–10017 | Empresa y dirección |
| 10021 | Fecha de creación |
| 10031 | Tipo de proyecto |
| 10100–10117 | Datos del cliente |
| 10184 | GUID del proyecto |
| 10070–10086 | Formatos de nombre por grupo de función (página 10070, dispositivo 10075, regletero 10079, conector 10080, cable 10081, punto de interrupción 10071, caja de estructura 10082, PLC 10077...) |
| 10089 | Orden de las estructuras en el nombre de página |

### Símbolo
| ID | Significado |
|---|---|
| 15000 | Biblioteca de símbolos |
| 16000 / 16002 | Nombre / número de símbolo |
| 16001 | Nº de puntos de conexión |
| 16003 | Variante por defecto |
| 16004–16015 | Disposición de textos por defecto de las variantes A–H |
| 16011 | Descripción |
| 16018 | Función principal |
| 16042 / 16043 / 16044 | Separa potencial / conecta redes / función de seguridad |
| 16047 | Tipo de símbolo |

### Regletero y cable (derivadas)
| ID | Significado |
|---|---|
| 35006 / 35012 | Nº de bornes / de bornes principales |
| 35000 / 35003 / 35004 | Nº de bornes de pantalla / N / PE |
| 35007 / 35008 | Primer / último borne |
| 20040 / 20042 / 20043 | Tipo de cable / nº de hilos / sección |
| 20041 | Longitud de cable |
| 20044 | Tensión del cable |
| 35100 / 35101 / 35108 | Hilos usados / libres / totales |
| 35103 / 35104 / 35102 | Nº de hilos PE / N / pantallas |

---

## Anexo B — Enumeraciones de dominio (vocabulario)

- **Dirección de punto de conexión**: arriba 1, derecha 2, abajo 4, izquierda 8 (combinable).
- **Tipo de conexión de pin**: neutro, cableado, puente, directa, fibra, radio, no eléctrica,
  funcional, fluido, interna, proceso, puente en peine.
- **Tipo de potencial**: neutro, L, N, PE, +, M, −, pantalla, PEN.
- **Tipo de señal**: sensor, actuador, alim. de conexión, alim. de equipo, fuente, DI, DO,
  AI, AO, fuente/entrada/salida/fin de bus, puertos de presión/succión.
- **Fuente/destino**: indefinido, fuente, destino.
- **Clase de cable**: conexión individual, cable, conducto, barra de fases, línea.
- **Tipo de borna** (patrón de conexión): tornillo simple, doble, jaula, IDC, tornillo,
  puntera, enchufe, soldadura, enchufable, boquilla, rosca interior/exterior, tuerca, racor,
  definida por usuario.
- **Tipo de puente**: externo, interno, pequeño (obsoleto), peine ranura 3/4/5.
- **Visibilidad**: visible, invisible, por capa.
- **Tipos de propiedad**: bool, entero, real, texto, multilingüe, punto, tiempo, coordenada,
  valor con unidad.
- **Mensajes**: clasificación indicación/aviso/error; origen online/offline/explícito.
- **Estado de comparación**: añadido, borrado, cambiado, sin cambios.
- **Modo de apertura**: estándar, solo lectura, exclusivo.

---

*Documento generado a partir de la documentación pública de la API de EPLAN (eplan.help) como
especificación funcional, siguiendo un enfoque clean-room: conceptos y datos de
interoperabilidad sí; código, texto e interfaz de EPLAN no.*
