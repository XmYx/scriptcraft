//! The reports window: statistics, characters and locations.

use egui::{Context, RichText};
use scriptcraft_engine::reports;
use scriptcraft_layout::Layout;
use scriptcraft_model::Screenplay;

use crate::i18n::tr;

pub fn show(ctx: &Context, open: &mut bool, sp: &Screenplay, layout: &Layout) {
    egui::Window::new(tr("Reports")).open(open).default_width(520.0).show(ctx, |ui| {
        egui::ScrollArea::vertical().max_height(560.0).show(ui, |ui| {
            let s = reports::statistics(sp, layout);
            ui.heading(tr("Statistics"));
            egui::Grid::new("report-stats").num_columns(2).striped(true).show(ui, |ui| {
                let rows: [(&str, usize); 11] = [
                    (tr("Pages"), s.pages),
                    (tr("Scenes"), s.scenes),
                    (tr("Words"), s.words),
                    (tr("Words of dialogue"), s.dialogue_words),
                    (tr("Speeches"), s.speeches),
                    (tr("Characters"), s.characters),
                    (tr("Locations"), s.locations),
                    (tr("Interior scenes"), s.interior_scenes),
                    (tr("Exterior scenes"), s.exterior_scenes),
                    (tr("Day scenes"), s.day_scenes),
                    (tr("Night scenes"), s.night_scenes),
                ];
                for (name, value) in rows {
                    ui.label(name);
                    ui.label(value.to_string());
                    ui.end_row();
                }
            });
            ui.separator();
            ui.heading(tr("Characters"));
            egui::Grid::new("report-characters").num_columns(5).striped(true).show(ui, |ui| {
                for h in [tr("Name"), tr("Speeches"), tr("Words"), tr("Scenes"), tr("First page")] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for c in reports::characters(sp, layout) {
                    ui.label(RichText::new(&c.name).monospace());
                    ui.label(c.speeches.to_string());
                    ui.label(c.words.to_string());
                    ui.label(c.scenes.to_string());
                    ui.label(c.first_page.to_string());
                    ui.end_row();
                }
            });
            ui.separator();
            ui.heading(tr("Locations"));
            egui::Grid::new("report-locations").num_columns(4).striped(true).show(ui, |ui| {
                for h in [tr("Location"), tr("Int./Ext."), tr("Scenes"), tr("Length (eighths)")] {
                    ui.label(RichText::new(h).strong());
                }
                ui.end_row();
                for l in reports::locations(sp, layout) {
                    ui.label(RichText::new(&l.location).monospace());
                    ui.label(&l.interior_exterior);
                    ui.label(l.scenes.to_string());
                    ui.label(l.eighths.to_string());
                    ui.end_row();
                }
            });
        });
    });
}
