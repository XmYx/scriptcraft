//! Index cards: one card per scene with its number, heading, synopsis, opening action, length
//! and cast. Arrow keys move between cards, Enter opens the scene in the draft, Alt+↑/↓ (and
//! Alt+←/→) move the scene earlier or later, and the synopsis is edited on the card.

use egui::{Color32, Key, Modifiers, RichText, Stroke, Ui, vec2};
use scriptcraft_interchange::scene_list;
use scriptcraft_model::{Element, ElementKind};

use crate::app::{ScriptApp, View};
use crate::i18n::{tr, trf};

#[derive(Default)]
pub struct CardsState {
    /// The selected scene (0-based).
    pub selected: Option<usize>,
    /// Move keyboard focus to the selected card on the next frame.
    pub focus_selected: bool,
}

const CARD_W: f32 = 260.0;
const CARD_H: f32 = 190.0;
const SELECTED: Color32 = Color32::from_rgb(0x1F, 0x5F, 0xBF);

fn card_id(scene: usize) -> egui::Id {
    egui::Id::unique(("scriptcraft-card", scene))
}

pub fn show(app: &mut ScriptApp, ui: &mut Ui) {
    let list = {
        let sp = app.doc.screenplay().clone();
        let layout = app.doc.layout();
        scene_list(&sp, layout, None)
    };
    let count = list.scenes.len();
    // A card is its width, 10 px inner margin each side, the stroke and 16 px of gap.
    let columns = ((ui.available_width() - 32.0) / (CARD_W + 20.0 + 4.0 + 16.0)).floor().max(1.0) as usize;
    let mut open: Option<usize> = None;
    let mut synopsis_edit: Option<(usize, String)> = None;
    // Keyboard on a focused card: arrows move the selection, Alt+←/→ move the scene.
    let focused_card = (0..count).find(|i| ui.memory(|m| m.has_focus(card_id(*i))));
    if let Some(i) = focused_card {
        // The cards handle the arrow keys themselves; egui's own arrow-key focus movement stays out.
        let filter = egui::EventFilter { horizontal_arrows: true, vertical_arrows: true, ..Default::default() };
        ui.memory_mut(|m| m.set_focus_lock_filter(card_id(i), filter));
        app.cards.selected = Some(i);
        // Alt+←/→ first: plain-arrow matching would also take them (it ignores an extra Alt).
        let ctx = ui.ctx().clone();
        if ui.input_mut(|input| input.consume_key(Modifiers::ALT, Key::ArrowLeft)) {
            app.run(&ctx, scriptcraft_engine::Command::MoveSceneUp);
        } else if ui.input_mut(|input| input.consume_key(Modifiers::ALT, Key::ArrowRight)) {
            app.run(&ctx, scriptcraft_engine::Command::MoveSceneDown);
        }
        let step = ui.input_mut(|input| {
            if input.consume_key(Modifiers::NONE, Key::ArrowRight) {
                1
            } else if input.consume_key(Modifiers::NONE, Key::ArrowLeft) {
                -1
            } else if input.consume_key(Modifiers::NONE, Key::ArrowDown) {
                columns as isize
            } else if input.consume_key(Modifiers::NONE, Key::ArrowUp) {
                -(columns as isize)
            } else {
                0
            }
        });
        if step != 0 {
            let next = (i as isize + step).clamp(0, count.saturating_sub(1) as isize) as usize;
            app.cards.selected = Some(next);
            app.cards.focus_selected = true;
        }
    }
    egui::ScrollArea::vertical().auto_shrink([false, false]).id_salt("cards-scroll").show(ui, |ui| {
        ui.add_space(16.0);
        if count == 0 {
            ui.label(tr("No scene headings yet. Start one with INT. or EXT."));
        }
        for row in list.scenes.chunks(columns) {
            ui.horizontal_top(|ui| {
                ui.add_space(16.0);
                for scene in row {
                    let i = scene.index - 1;
                    let selected = app.cards.selected == Some(i);
                    let stroke = if selected { Stroke::new(2.5, SELECTED) } else { Stroke::new(1.0, Color32::from_gray(150)) };
                    egui::Frame::new().fill(Color32::from_rgb(0xFF, 0xFD, 0xF2)).stroke(stroke).corner_radius(4.0).inner_margin(10.0).show(
                        ui,
                        |ui| {
                            ui.set_min_size(vec2(CARD_W, CARD_H));
                            ui.set_max_width(CARD_W);
                            *ui.visuals_mut() = egui::Visuals::light();
                            // The card sits in a row; its own contents flow top to bottom.
                            ui.vertical(|ui| {
                                let title = format!("{}  {}", scene.number, scene.heading);
                                let name = trf("Scene {number}: {heading}", &[("number", &scene.number), ("heading", &scene.heading)]);
                                // The card's focusable heading carries the card's id, so arrow keys can move focus.
                                let (rect, _) = ui.allocate_exact_size(vec2(CARD_W, 22.0), egui::Sense::hover());
                                let heading = ui.interact(rect, card_id(i), egui::Sense::click());
                                let visuals = ui.style().interact_selectable(&heading, selected);
                                ui.painter().rect_filled(rect, 3.0, visuals.weak_bg_fill);
                                if heading.has_focus() {
                                    ui.painter().rect_stroke(rect, 3.0, Stroke::new(2.0, SELECTED), egui::StrokeKind::Inside);
                                }
                                let galley = egui::WidgetText::from(RichText::new(title).strong().monospace()).into_galley(
                                    ui,
                                    Some(egui::TextWrapMode::Truncate),
                                    rect.width() - 8.0,
                                    egui::TextStyle::Button,
                                );
                                ui.painter().galley(rect.left_center() + vec2(4.0, -galley.size().y / 2.0), galley, visuals.text_color());
                                heading.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name.clone()));
                                if app.cards.focus_selected && selected {
                                    heading.request_focus();
                                    heading.scroll_to_me(None);
                                    app.cards.focus_selected = false;
                                }
                                if heading.clicked() {
                                    app.cards.selected = Some(i);
                                    heading.request_focus();
                                }
                                if heading.double_clicked()
                                    || (heading.has_focus() && ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter)))
                                {
                                    open = Some(scene.element);
                                }
                                let mut synopsis = scene.synopsis.clone().unwrap_or_default();
                                let edit = ui.add(
                                    egui::TextEdit::multiline(&mut synopsis)
                                        .hint_text(tr("Synopsis"))
                                        .desired_rows(3)
                                        .desired_width(CARD_W)
                                        .id_salt(("card-synopsis", i)),
                                );
                                ui.ctx().accesskit_node_builder(edit.id, |node| {
                                    node.set_label(trf("Synopsis of scene {number}", &[("number", &scene.number)]))
                                });
                                if edit.changed() {
                                    synopsis_edit = Some((scene.element, synopsis));
                                }
                                let action = excerpt(&scene.action, 140);
                                if !action.is_empty() {
                                    ui.label(RichText::new(action).small().color(Color32::from_gray(60)));
                                }
                                ui.add_space(4.0);
                                let cast = if scene.characters.is_empty() { String::new() } else { format!(" · {}", scene.characters.join(", ")) };
                                let length = trf(
                                    "p. {page} · {eighths}/8",
                                    &[("page", &scene.page_start.to_string()), ("eighths", &scene.length_eighths.to_string())],
                                );
                                ui.label(RichText::new(format!("{length}{cast}")).small().color(Color32::from_gray(70)));
                            });
                        },
                    );
                    ui.add_space(16.0);
                }
            });
            ui.add_space(16.0);
        }
        ui.label(RichText::new(tr("Arrow keys move between cards · Enter opens the scene · Alt+arrows move it")).weak());
    });
    if let Some((heading, text)) = synopsis_edit {
        set_synopsis(app, heading, text);
    }
    if let Some(element) = open {
        app.view = View::Draft;
        app.jump = Some(element);
    }
}

/// Write a scene's synopsis: the first synopsis after its heading, inserted when missing.
fn set_synopsis(app: &mut ScriptApp, heading: usize, text: String) {
    let existing = app
        .doc
        .screenplay()
        .elements
        .iter()
        .enumerate()
        .skip(heading + 1)
        .take_while(|(_, e)| e.kind != ElementKind::SceneHeading)
        .find(|(_, e)| e.kind == ElementKind::Synopsis)
        .map(|(i, _)| i);
    match existing {
        Some(i) => app.doc.set_text(i, text),
        None => {
            app.doc.edit(|sp| sp.elements.insert(heading + 1, Element::new(ElementKind::Synopsis, text)));
            app.editor.reset_ids(app.doc.screenplay().elements.len());
        }
    }
}

/// The first paragraph of `text`, cut at a word to at most `max` characters, with `…` when cut.
fn excerpt(text: &str, max: usize) -> String {
    let first = text.split("\n\n").next().unwrap_or("").trim();
    if first.chars().count() <= max {
        return first.to_owned();
    }
    let cut: String = first.chars().take(max).collect();
    let at = cut.rfind(' ').unwrap_or(cut.len());
    format!("{}…", cut.get(..at).unwrap_or(&cut).trim_end())
}
