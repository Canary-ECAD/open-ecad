//! Canvas de página: marco con rejilla de referencias, rejilla de dibujo, zoom y
//! desplazamiento. Coordenadas del mundo en mm, con origen abajo a la izquierda y Y hacia
//! arriba (como en un plano).

use egui::{
    Align2, Color32, FontId, PointerButton, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, pos2,
    vec2,
};

use crate::mock::Page;

/// A3 apaisado.
pub const PAGE_SIZE: Vec2 = vec2(420.0, 297.0);
const BORDER: f32 = 5.0;
const COLUMN_STRIP: f32 = 5.0;
const TITLE_BLOCK: f32 = 30.0;
const COLUMNS: u32 = 10;

const MIN_ZOOM: f32 = 0.3;
const MAX_ZOOM: f32 = 80.0;
/// 96 ppp: 1 mm en pantalla a escala 1:1.
const PX_PER_MM_1_1: f32 = 96.0 / 25.4;

/// Ajustes de visualización compartidos por todas las páginas.
pub struct ViewSettings {
    pub grid_visible: bool,
    pub snap: bool,
    pub ortho: bool,
    /// Paso de la rejilla de dibujo, en mm.
    pub grid_mm: f32,
}

impl Default for ViewSettings {
    fn default() -> Self {
        Self { grid_visible: true, snap: true, ortho: false, grid_mm: 4.0 }
    }
}

/// Vista de una página abierta: escala y posición. No se persiste.
pub struct PageView {
    /// Píxeles por mm.
    zoom: f32,
    /// Posición en pantalla del origen de la página, relativa a la esquina del canvas.
    offset: Vec2,
    fit_pending: bool,
}

impl Default for PageView {
    fn default() -> Self {
        Self { zoom: 1.0, offset: Vec2::ZERO, fit_pending: true }
    }
}

/// Lo que el canvas devuelve a la ventana para la barra de estado.
pub struct CanvasOutput {
    pub cursor_mm: Option<Pos2>,
    pub zoom_percent: f32,
}

impl PageView {
    pub fn request_fit(&mut self) {
        self.fit_pending = true;
    }

    pub fn set_one_to_one(&mut self, canvas: Rect) {
        self.zoom_around(canvas, canvas.center(), PX_PER_MM_1_1 / self.zoom);
    }

    fn fit(&mut self, canvas: Rect) {
        let margin = 16.0;
        let avail = (canvas.size() - Vec2::splat(2.0 * margin)).max(Vec2::splat(10.0));
        self.zoom = (avail.x / PAGE_SIZE.x).min(avail.y / PAGE_SIZE.y).clamp(MIN_ZOOM, MAX_ZOOM);
        self.offset = (canvas.size() - PAGE_SIZE * self.zoom) / 2.0;
    }

    fn zoom_around(&mut self, canvas: Rect, screen_pos: Pos2, factor: f32) {
        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let anchor = screen_pos - canvas.min;
        self.offset = anchor - (anchor - self.offset) * (new_zoom / self.zoom);
        self.zoom = new_zoom;
    }

    pub fn ui(&mut self, ui: &mut Ui, page: &Page, settings: &ViewSettings) -> CanvasOutput {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let canvas = response.rect;
        if self.fit_pending && canvas.width() > 1.0 {
            self.fit(canvas);
            self.fit_pending = false;
        }

        // Navegación: rueda = zoom en el cursor; arrastre con botón central (o derecho) = desplazar.
        if response.hovered() {
            let (scroll, pinch, hover) =
                ui.input(|i| (i.smooth_scroll_delta, i.zoom_delta(), i.pointer.hover_pos()));
            let factor = pinch * (scroll.y * 0.0025).exp();
            if let Some(hover) = hover
                && (factor - 1.0).abs() > f32::EPSILON
            {
                self.zoom_around(canvas, hover, factor);
            }
        }
        if response.dragged_by(PointerButton::Middle) || response.dragged_by(PointerButton::Secondary) {
            self.offset += response.drag_delta();
        }
        if response.double_clicked_by(PointerButton::Middle) {
            self.fit(canvas);
        }

        let to_screen = Transform { origin: canvas.min + self.offset, zoom: self.zoom };
        painter.rect_filled(canvas, 0.0, crate::theme::CANVAS_BG);
        paint_page(&painter.with_clip_rect(canvas), &to_screen, page, settings, canvas);

        let cursor_mm = response.hover_pos().map(|p| {
            let mm = to_screen.to_mm(p);
            if settings.snap { snap(mm, settings.grid_mm) } else { mm }
        });
        if let Some(mm) = cursor_mm {
            let p = to_screen.to_screen(mm);
            let c = Color32::from_rgb(0, 120, 215);
            let r = 6.0;
            painter.line_segment([p - vec2(r, 0.0), p + vec2(r, 0.0)], Stroke::new(1.0, c));
            painter.line_segment([p - vec2(0.0, r), p + vec2(0.0, r)], Stroke::new(1.0, c));
        }

        CanvasOutput { cursor_mm, zoom_percent: self.zoom / PX_PER_MM_1_1 * 100.0 }
    }
}

fn snap(p: Pos2, step: f32) -> Pos2 {
    pos2((p.x / step).round() * step, (p.y / step).round() * step)
}

pub struct Transform {
    /// Esquina superior izquierda de la página en pantalla.
    pub origin: Pos2,
    pub zoom: f32,
}

impl Transform {
    /// Ajusta la página entera dentro de `rect` (para miniaturas).
    pub fn fit(rect: Rect) -> Self {
        let zoom = (rect.width() / PAGE_SIZE.x).min(rect.height() / PAGE_SIZE.y);
        let origin = rect.center() - PAGE_SIZE * zoom / 2.0;
        Self { origin, zoom }
    }

    pub fn to_screen(&self, mm: Pos2) -> Pos2 {
        self.origin + vec2(mm.x, PAGE_SIZE.y - mm.y) * self.zoom
    }

    pub fn to_mm(&self, screen: Pos2) -> Pos2 {
        let v = (screen - self.origin) / self.zoom;
        pos2(v.x, PAGE_SIZE.y - v.y)
    }

    fn rect(&self, min_mm: Pos2, max_mm: Pos2) -> Rect {
        Rect::from_two_pos(self.to_screen(min_mm), self.to_screen(max_mm))
    }
}

/// Dibuja la hoja, la rejilla y un marco de ejemplo (cajetín y columnas). El marco real
/// vendrá de los datos maestros (MAPA-DOMINIO.md §7.2).
pub fn paint_page(
    painter: &egui::Painter,
    t: &Transform,
    page: &Page,
    settings: &ViewSettings,
    visible: Rect,
) {
    let ink = Color32::from_gray(30);
    let thin = Stroke::new((0.25 * t.zoom).clamp(0.5, 1.5), ink);
    let thick = Stroke::new((0.5 * t.zoom).clamp(1.0, 2.5), ink);

    let sheet = t.rect(Pos2::ZERO, PAGE_SIZE.to_pos2());
    painter.rect_filled(sheet.translate(vec2(3.0, 3.0)), 0.0, Color32::from_black_alpha(80));
    painter.rect_filled(sheet, 0.0, Color32::WHITE);

    if settings.grid_visible {
        paint_grid(painter, t, settings.grid_mm, sheet.intersect(visible));
    }

    let (x0, x1) = (BORDER, PAGE_SIZE.x - BORDER);
    let (y0, y1) = (BORDER, PAGE_SIZE.y - BORDER);
    painter.rect_stroke(t.rect(pos2(x0, y0), pos2(x1, y1)), 0.0, thick, StrokeKind::Middle);

    // Rejilla de referencias: columnas numeradas arriba (las referencias cruzadas son
    // página/columna).
    let strip_y = y1 - COLUMN_STRIP;
    painter.line_segment([t.to_screen(pos2(x0, strip_y)), t.to_screen(pos2(x1, strip_y))], thin);
    let col_w = (x1 - x0) / COLUMNS as f32;
    let text_size = 2.5 * t.zoom;
    for i in 0..COLUMNS {
        let cx = x0 + col_w * i as f32;
        if i > 0 {
            painter.line_segment([t.to_screen(pos2(cx, strip_y)), t.to_screen(pos2(cx, y1))], thin);
        }
        if text_size >= 1.5 {
            painter.text(
                t.to_screen(pos2(cx + col_w / 2.0, strip_y + COLUMN_STRIP / 2.0)),
                Align2::CENTER_CENTER,
                i + 1,
                FontId::proportional(text_size),
                ink,
            );
        }
    }

    // Cajetín.
    let tb_y = y0 + TITLE_BLOCK;
    painter.line_segment([t.to_screen(pos2(x0, tb_y)), t.to_screen(pos2(x1, tb_y))], thick);
    let cells = [x0 + 60.0, x0 + 230.0, x0 + 330.0];
    for cx in cells {
        painter.line_segment([t.to_screen(pos2(cx, y0)), t.to_screen(pos2(cx, tb_y))], thin);
    }
    if text_size >= 1.5 {
        let small = FontId::proportional(text_size);
        let big = FontId::proportional(text_size * 1.6);
        let mid_y = y0 + TITLE_BLOCK / 2.0;
        painter.text(t.to_screen(pos2(x0 + 30.0, mid_y)), Align2::CENTER_CENTER, "open-ecad", big.clone(), ink);
        painter.text(
            t.to_screen(pos2((cells[0] + cells[1]) / 2.0, mid_y)),
            Align2::CENTER_CENTER,
            &page.description,
            big,
            ink,
        );
        painter.text(
            t.to_screen(pos2((cells[1] + cells[2]) / 2.0, mid_y)),
            Align2::CENTER_CENTER,
            &page.kind,
            small.clone(),
            ink,
        );
        painter.text(
            t.to_screen(pos2((cells[2] + x1) / 2.0, mid_y)),
            Align2::CENTER_CENTER,
            page.name(),
            FontId::proportional(text_size * 1.3),
            ink,
        );
    }
}

/// Puntos de rejilla. Si el paso real queda demasiado denso en pantalla, se dibuja un
/// múltiplo (el snap sigue usando el paso real).
fn paint_grid(painter: &egui::Painter, t: &Transform, grid_mm: f32, visible: Rect) {
    const MIN_PX: f32 = 8.0;
    let Some(mult) = [1.0, 2.5, 5.0, 10.0, 25.0, 50.0].into_iter().find(|m| grid_mm * m * t.zoom >= MIN_PX)
    else {
        return;
    };
    let step = grid_mm * mult;
    let a = t.to_mm(visible.left_bottom());
    let b = t.to_mm(visible.right_top());
    let (ix0, ix1) = ((a.x / step).ceil() as i64, (b.x / step).floor() as i64);
    let (iy0, iy1) = ((a.y / step).ceil() as i64, (b.y / step).floor() as i64);
    if (ix1 - ix0 + 1) * (iy1 - iy0 + 1) > 60_000 {
        return;
    }
    let color = Color32::from_gray(185);
    let r = 0.9;
    for ix in ix0..=ix1 {
        for iy in iy0..=iy1 {
            let p = t.to_screen(pos2(ix as f32 * step, iy as f32 * step));
            painter.rect_filled(Rect::from_center_size(p, Vec2::splat(r * 2.0)), 0.0, color);
        }
    }
}
