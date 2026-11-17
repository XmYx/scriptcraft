//! Breaking a script into pages.

use scriptcraft_model::{ElementKind, Screenplay, TitlePage, character_name};
use serde::Serialize;

use crate::format::{Align, Format, format_of};
use crate::wrap::wrap;
use crate::{PageSetup, Paper};

/// What a line of a page is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Role {
    /// Text of an element of this kind.
    Element(ElementKind),
    /// `(MORE)` under dialogue that continues on the next page.
    More,
    /// `NAME (CONT'D)` at the top of the page where dialogue continues.
    Continued,
    /// A scene number in the margin.
    SceneNumber,
    /// A line of the title page.
    TitlePage,
}

/// One line of text on a page.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Line {
    /// Body row, 0 at the top margin.
    pub row: usize,
    /// First column, from the left edge of the paper, in characters.
    pub column: usize,
    pub text: String,
    pub role: Role,
    /// The element the line belongs to.
    pub element: Option<usize>,
    /// Which wrapped line of the element this is.
    pub line_in_element: usize,
}

/// One page.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Page {
    /// 1 for the first script page (the title page is 0).
    pub number: u32,
    /// The page number printed at the top right (`2.`); none on the first page.
    pub header: Option<String>,
    pub lines: Vec<Line>,
}

/// Where an element starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Position {
    pub page: u32,
    pub row: usize,
}

/// The first line of a page: which element and which of its wrapped lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PageStart {
    pub page: u32,
    pub element: usize,
    pub line_in_element: usize,
}

/// A paginated script.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Layout {
    pub paper: Paper,
    pub rows_per_page: usize,
    pub title_page: Option<Page>,
    pub pages: Vec<Page>,
    /// Where each element starts; `None` for elements that are not printed.
    pub starts: Vec<Option<Position>>,
    /// The element each page starts with, in page order.
    pub page_starts: Vec<PageStart>,
}

impl Layout {
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Rows from the top of page 1 to `pos`, counting whole pages.
    pub fn absolute_row(&self, pos: Position) -> usize {
        (pos.page.saturating_sub(1) as usize).saturating_mul(self.rows_per_page).saturating_add(pos.row)
    }

    /// The page element `index` starts on, or the page of the nearest printed element before it.
    pub fn page_of(&self, index: usize) -> u32 {
        let end = index.saturating_add(1).min(self.starts.len());
        self.starts.get(..end).and_then(|s| s.iter().rev().flatten().next()).map_or(1, |p| p.page)
    }
}

struct Block {
    element: usize,
    kind: ElementKind,
    format: Format,
    lines: Vec<String>,
    /// For dialogue, lyrics and parentheticals: the speaker, for `(CONT'D)`.
    speaker: Option<String>,
}

impl Block {
    fn is_speech_body(&self) -> bool {
        self.speaker.is_some() && matches!(self.kind, ElementKind::Dialogue | ElementKind::Lyric | ElementKind::Parenthetical)
    }
}

/// Lay out a whole script.
pub fn paginate(sp: &Screenplay, setup: &PageSetup) -> Layout {
    let blocks = blocks(sp);
    let mut pager = Pager {
        rows: setup.paper.rows_per_page(),
        setup: *setup,
        pages: Vec::new(),
        current: Page { number: 1, header: None, lines: Vec::new() },
        row: 0,
        starts: vec![None; sp.elements.len()],
        page_starts: Vec::new(),
        scene: 0,
    };
    for (i, block) in blocks.iter().enumerate() {
        let keep = keep_with_next(block, blocks.get(i + 1));
        pager.lay(block, keep);
    }
    let Pager { mut pages, current, starts, page_starts, .. } = pager;
    if !current.lines.is_empty() || pages.is_empty() {
        pages.push(current);
    }
    let title_page = (!sp.title_page.is_empty()).then(|| title_page(&sp.title_page, setup.paper.rows_per_page()));
    Layout { paper: setup.paper, rows_per_page: setup.paper.rows_per_page(), title_page, pages, starts, page_starts }
}

fn blocks(sp: &Screenplay) -> Vec<Block> {
    let mut out = Vec::new();
    let mut speaker: Option<String> = None;
    for (element, e) in sp.elements.iter().enumerate() {
        if e.kind == ElementKind::PageBreak {
            let format = Format { left: 0, width: 0, align: Align::Left, space_before: 0 };
            out.push(Block { element, kind: e.kind, format, lines: Vec::new(), speaker: None });
            speaker = None;
            continue;
        }
        let Some(format) = format_of(e.kind) else { continue };
        match e.kind {
            ElementKind::Character => speaker = Some(character_name(&e.text)),
            k if k.is_speech() => {}
            _ => speaker = None,
        }
        let speaker = if e.kind.is_speech() && e.kind != ElementKind::Character { speaker.clone() } else { None };
        let lines = wrap(&e.display_text(), format.width);
        out.push(Block { element, kind: e.kind, format, lines, speaker });
    }
    out
}

/// Rows that must fit on the same page after `block`, so it is not left alone at the bottom.
fn keep_with_next(block: &Block, next: Option<&Block>) -> usize {
    let Some(next) = next else { return 0 };
    if next.kind == ElementKind::PageBreak {
        return 0;
    }
    let n = next.lines.len();
    let speech_room = if n <= 2 { n } else { 3 }; // two lines and (MORE)
    match block.kind {
        ElementKind::SceneHeading | ElementKind::Shot => next.format.space_before + n.min(2),
        ElementKind::Character | ElementKind::Parenthetical if next.is_speech_body() => speech_room,
        _ => 0,
    }
}

/// Where to cut a block of `lines` when `room` rows are left: at least two lines on each side,
/// at the end of a sentence when possible.
fn choose_split(lines: &[String], room: usize) -> Option<usize> {
    let n = lines.len();
    if n < 4 || room < 2 {
        return None;
    }
    let max = room.min(n - 2);
    if max < 2 {
        return None;
    }
    let sentence_end = (2..=max).rev().find(|&k| lines.get(k - 1).is_some_and(|l| l.trim_end().ends_with(['.', '?', '!', '"'])));
    Some(sentence_end.unwrap_or(max))
}

struct Pager {
    rows: usize,
    setup: PageSetup,
    pages: Vec<Page>,
    current: Page,
    row: usize,
    starts: Vec<Option<Position>>,
    page_starts: Vec<PageStart>,
    scene: usize,
}

impl Pager {
    fn new_page(&mut self) {
        let number = self.current.number.saturating_add(1);
        let finished = std::mem::replace(&mut self.current, Page { number, header: Some(format!("{number}.")), lines: Vec::new() });
        self.pages.push(finished);
        self.row = 0;
    }

    fn lay(&mut self, b: &Block, keep: usize) {
        if b.kind == ElementKind::PageBreak {
            if !self.current.lines.is_empty() {
                self.new_page();
            }
            return;
        }
        let n = b.lines.len();
        let before = if self.current.lines.is_empty() { 0 } else { b.format.space_before };
        if self.row + before + n + keep <= self.rows {
            self.row += before;
            self.emit(b, 0, n);
            return;
        }
        let speech = b.is_speech_body();
        let reserve = usize::from(speech);
        let splittable = matches!(b.kind, ElementKind::Action | ElementKind::Dialogue | ElementKind::Lyric);
        let room = self.rows.saturating_sub(self.row + before + reserve);
        match splittable.then(|| choose_split(&b.lines, room)).flatten() {
            Some(k) => {
                self.row += before;
                self.emit(b, 0, k);
                self.break_page(b);
                self.flow(b, k);
            }
            None => {
                if !self.current.lines.is_empty() {
                    if speech && self.continues_speech() { self.break_page(b) } else { self.new_page() }
                }
                self.flow(b, 0);
            }
        }
    }

    /// Whether the last thing on the page belongs to the speech being laid out.
    fn continues_speech(&self) -> bool {
        self.current.lines.last().is_some_and(|l| matches!(l.role, Role::Element(k) if k.is_speech()) || l.role == Role::Continued)
    }

    /// Place lines `start..` from the current row, breaking pages as needed (a block longer than
    /// a page is cut wherever the page ends).
    fn flow(&mut self, b: &Block, start: usize) {
        let n = b.lines.len();
        let reserve = usize::from(b.is_speech_body());
        let mut i = start;
        while i < n {
            let room = self.rows.saturating_sub(self.row);
            if n - i <= room {
                self.emit(b, i, n);
                return;
            }
            let take = room.saturating_sub(reserve).max(1).min(n - i);
            self.emit(b, i, i + take);
            i += take;
            self.break_page(b);
        }
    }

    /// End the page; dialogue continues with `(MORE)` and `NAME (CONT'D)`.
    fn break_page(&mut self, b: &Block) {
        let speaker = b.speaker.as_ref().filter(|_| b.is_speech_body());
        if speaker.is_some() {
            self.current.lines.push(Line {
                row: self.row,
                column: 37,
                text: "(MORE)".into(),
                role: Role::More,
                element: Some(b.element),
                line_in_element: 0,
            });
        }
        self.new_page();
        if let Some(name) = speaker {
            let text = format!("{name} (CONT'D)");
            self.current.lines.push(Line { row: 0, column: 37, text, role: Role::Continued, element: Some(b.element), line_in_element: 0 });
            self.row = 1;
        }
    }

    fn emit(&mut self, b: &Block, from: usize, to: usize) {
        for (i, text) in b.lines.iter().enumerate().take(to).skip(from) {
            if self.current.lines.iter().all(|l| l.role == Role::Continued) {
                self.page_starts.push(PageStart { page: self.current.number, element: b.element, line_in_element: i });
            }
            if i == 0 {
                if let Some(slot) = self.starts.get_mut(b.element) {
                    *slot = Some(Position { page: self.current.number, row: self.row });
                }
                if b.kind == ElementKind::SceneHeading {
                    self.scene += 1;
                    if self.setup.scene_numbers {
                        self.scene_numbers(b.element);
                    }
                }
            }
            let len = text.chars().count();
            let f = b.format;
            let column = match f.align {
                Align::Left => f.left,
                Align::Right => f.left + f.width.saturating_sub(len),
                Align::Center => f.left + f.width.saturating_sub(len) / 2,
            };
            self.current.lines.push(Line {
                row: self.row,
                column,
                text: text.clone(),
                role: Role::Element(b.kind),
                element: Some(b.element),
                line_in_element: i,
            });
            self.row += 1;
        }
    }

    fn scene_numbers(&mut self, element: usize) {
        let number = self.scene.to_string();
        let (left, right) = (10usize.saturating_sub(number.len()), 77);
        for column in [left, right] {
            self.current.lines.push(Line {
                row: self.row,
                column,
                text: number.clone(),
                role: Role::SceneNumber,
                element: Some(element),
                line_in_element: 0,
            });
        }
    }
}

fn title_page(tp: &TitlePage, rows: usize) -> Page {
    let mut lines = Vec::new();
    let mut push_centered = |row: &mut usize, text: &str| {
        for l in wrap(text, 60) {
            let column = 15 + 60usize.saturating_sub(l.chars().count()) / 2;
            lines.push(Line { row: *row, column, text: l, role: Role::TitlePage, element: None, line_in_element: 0 });
            *row += 1;
        }
    };
    let mut row = rows / 3;
    for (key, gap) in [("Title", 3), ("Credit", 1), ("Author", 0), ("Authors", 3), ("Source", 0)] {
        if let Some(value) = tp.get(key) {
            push_centered(&mut row, value);
            row += gap;
        }
    }
    let mut bottom = |key: &str, column: usize, right: bool| {
        let Some(value) = tp.get(key) else { return };
        let wrapped: Vec<String> = value.split('\n').flat_map(|l| wrap(l, 40)).collect();
        let start = rows.saturating_sub(wrapped.len());
        for (i, l) in wrapped.into_iter().enumerate() {
            let column = if right { column.saturating_sub(l.chars().count()) } else { column };
            lines.push(Line { row: start + i, column, text: l, role: Role::TitlePage, element: None, line_in_element: 0 });
        }
    };
    bottom("Contact", 15, false);
    bottom("Draft date", 75, true);
    Page { number: 0, header: None, lines }
}

/// The layout as plain text: one string per page line, pages separated by a form feed. Used by
/// the command-line tool and the plain-text export.
pub fn render_text(layout: &Layout) -> String {
    let mut pages: Vec<String> = Vec::new();
    for page in layout.title_page.iter().chain(&layout.pages) {
        let mut grid: Vec<String> = vec![String::new(); layout.rows_per_page];
        let mut ordered: Vec<&Line> = page.lines.iter().collect();
        ordered.sort_by_key(|l| (l.row, l.column));
        for line in ordered {
            let Some(row) = grid.get_mut(line.row) else { continue };
            let have = row.chars().count();
            if have < line.column {
                row.extend(std::iter::repeat_n(' ', line.column - have));
            } else if have > line.column {
                row.push(' ');
            }
            row.push_str(&line.text);
        }
        let header = page.header.as_ref().map_or(String::new(), |h| format!("{}{h}", " ".repeat(75usize.saturating_sub(h.len()))));
        let mut text = format!("{header}\n\n");
        while grid.last().is_some_and(String::is_empty) {
            grid.pop();
        }
        text.push_str(&grid.join("\n"));
        text.push('\n');
        pages.push(text);
    }
    pages.join("\u{c}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use scriptcraft_model::Element;

    fn sp(elements: Vec<Element>) -> Screenplay {
        Screenplay { title_page: TitlePage::default(), elements }
    }

    #[test]
    fn empty_script_has_one_empty_page() {
        let layout = paginate(&Screenplay::default(), &PageSetup::default());
        assert_eq!(layout.page_count(), 1);
        assert!(layout.pages[0].lines.is_empty());
    }

    #[test]
    fn columns_follow_the_formats() {
        let layout = paginate(
            &sp(vec![
                Element::new(ElementKind::SceneHeading, "int. a - day"),
                Element::new(ElementKind::Character, "maya"),
                Element::new(ElementKind::Dialogue, "Hi."),
                Element::new(ElementKind::Transition, "CUT TO:"),
            ]),
            &PageSetup::default(),
        );
        let lines = &layout.pages[0].lines;
        assert_eq!((lines[0].column, lines[0].text.as_str(), lines[0].row), (15, "INT. A - DAY", 0));
        assert_eq!((lines[1].column, lines[1].text.as_str(), lines[1].row), (37, "MAYA", 2));
        assert_eq!((lines[2].column, lines[2].row), (25, 3));
        assert_eq!((lines[3].column, lines[3].row), (75 - 7, 5));
    }

    #[test]
    fn long_dialogue_splits_with_more_and_contd() {
        let mut elements = vec![Element::new(ElementKind::Action, "x\n".repeat(45).trim_end())];
        elements.push(Element::new(ElementKind::Character, "MAYA"));
        let speech: Vec<String> = (0..12).map(|i| format!("Line {i} of a long speech.")).collect();
        elements.push(Element::new(ElementKind::Dialogue, speech.join("\n")));
        let layout = paginate(&sp(elements), &PageSetup::default());
        assert_eq!(layout.page_count(), 2);
        let p1 = &layout.pages[0].lines;
        let p2 = &layout.pages[1].lines;
        assert_eq!(p1.last().map(|l| l.text.as_str()), Some("(MORE)"));
        assert_eq!(p2[0].text, "MAYA (CONT'D)");
        assert!(p1.iter().all(|l| l.row < 54));
        assert_eq!(layout.pages[1].header.as_deref(), Some("2."));
        let dialogue = p1.iter().chain(p2).filter(|l| l.role == Role::Element(ElementKind::Dialogue)).count();
        assert_eq!(dialogue, 12);
        assert_eq!(layout.page_starts[1], PageStart { page: 2, element: 2, line_in_element: layout.page_starts[1].line_in_element });
    }

    #[test]
    fn heading_is_not_left_at_the_bottom() {
        let mut elements = vec![Element::new(ElementKind::Action, "x\n".repeat(51).trim_end())];
        elements.push(Element::new(ElementKind::SceneHeading, "INT. B - NIGHT"));
        elements.push(Element::new(ElementKind::Action, "Two\nlines."));
        let layout = paginate(&sp(elements), &PageSetup::default());
        assert_eq!(layout.starts[1], Some(Position { page: 2, row: 0 }));
    }

    #[test]
    fn page_break_and_oversized_blocks() {
        let elements = vec![
            Element::new(ElementKind::Action, "a"),
            Element::new(ElementKind::PageBreak, ""),
            Element::new(ElementKind::Action, "y\n".repeat(200).trim_end()),
        ];
        let layout = paginate(&sp(elements), &PageSetup::default());
        assert_eq!(layout.starts[2].map(|p| p.page), Some(2));
        assert_eq!(layout.page_count(), 2 + 200 / 54);
        assert!(layout.pages.iter().all(|p| p.lines.iter().all(|l| l.row < 54)));
    }

    #[test]
    fn a4_holds_more_lines() {
        assert_eq!(Paper::UsLetter.rows_per_page(), 54);
        assert_eq!(Paper::A4.rows_per_page(), 58);
    }

    #[test]
    fn scene_numbers_in_both_margins() {
        let layout =
            paginate(&sp(vec![Element::new(ElementKind::SceneHeading, "INT. A - DAY")]), &PageSetup { scene_numbers: true, ..PageSetup::default() });
        let numbers: Vec<usize> = layout.pages[0].lines.iter().filter(|l| l.role == Role::SceneNumber).map(|l| l.column).collect();
        assert_eq!(numbers, [9, 77]);
    }
}
