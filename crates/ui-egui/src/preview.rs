//! Page preview: the printed pages exactly as laid out by `scriptcraft-layout`, read-only.

use egui::{Align2, Color32, Rect, Stroke, Ui, pos2, vec2};

use crate::app::ScriptApp;
use crate::editor::{INK, MUTED, PAPER};
use crate::fonts;
use crate::i18n::{tr, trf};

pub fn show(app: &mut ScriptApp, ui: &mut Ui) {
    let zoom = app.zoom;
    let layout = app.doc.layout().clone();
    let ppi = 72.0 * zoom;
    let (w_in, h_in) = layout.paper.size_in();
    let (page_w, page_h) = (w_in * ppi, h_in * ppi);
    let char_w = ppi / scriptcraft_layout::CHARS_PER_INCH;
    let row_h = ppi / scriptcraft_layout::LINES_PER_INCH;
    // 12 pt type: at 72 points per inch that is exactly one row.
    let font = fonts::script(row_h);
    // Fit the face's advance to 10 characters per inch.
    let advance = ui.ctx().fonts_mut(|f| f.glyph_width(&font, 'M')).max(0.1);
    let font = fonts::script(row_h * (char_w / advance).min(1.2));
    egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("preview-scroll").show(ui, |ui| {
        ui.add_space(16.0);
        let side = ((ui.available_width() - page_w) / 2.0).max(12.0);
        for page in layout.title_page.iter().chain(&layout.pages) {
            let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), page_h + 24.0), egui::Sense::hover());
            // The painted page is one document node with its text, for screen readers.
            let name = if page.number == 0 { tr("Title page").to_owned() } else { trf("Page {page}", &[("page", &page.number.to_string())]) };
            let text = page_text(page);
            ui.ctx().accesskit_node_builder(response.id, |node| {
                node.set_role(egui::accesskit::Role::Document);
                node.set_label(name);
                node.set_value(text);
            });
            let paper = Rect::from_min_size(pos2(rect.left() + side, rect.top()), vec2(page_w, page_h));
            if !ui.is_rect_visible(paper) {
                continue;
            }
            let painter = ui.painter_at(paper.expand(4.0));
            painter.rect_filled(paper.translate(vec2(2.0, 3.0)), 0.0, Color32::from_black_alpha(40));
            painter.rect(paper, 0.0, PAPER, Stroke::new(1.0, Color32::from_gray(190)), egui::StrokeKind::Inside);
            let top = paper.top() + scriptcraft_layout::TOP_MARGIN_IN * ppi;
            if let Some(header) = &page.header {
                painter.text(pos2(paper.left() + 75.0 * char_w, paper.top() + 0.5 * ppi), Align2::RIGHT_TOP, header, font.clone(), INK);
            }
            for line in &page.lines {
                let colour = if line.role == scriptcraft_layout::Role::SceneNumber { MUTED } else { INK };
                let pos = pos2(paper.left() + line.column as f32 * char_w, top + line.row as f32 * row_h);
                painter.text(pos, Align2::LEFT_TOP, &line.text, font.clone(), colour);
            }
        }
        ui.add_space(32.0);
    });
}

/// A page's text in reading order, one line per row.
fn page_text(page: &scriptcraft_layout::Page) -> String {
    let mut lines: Vec<&scriptcraft_layout::Line> = page.lines.iter().filter(|l| l.role != scriptcraft_layout::Role::SceneNumber).collect();
    lines.sort_by_key(|l| (l.row, l.column));
    lines.iter().map(|l| l.text.as_str()).collect::<Vec<_>>().join("\n")
}
