//! Interface translations through craft-libs' `craft-i18n`, over the optional `CRAFT_I18N_DIR`
//! catalogs (`build.rs`). The English source text is the key; a missing translation shows English.

use std::cell::Cell;
use std::sync::LazyLock;

pub use craft_i18n::{Lang, fmt};

include!(concat!(env!("OUT_DIR"), "/craft_i18n.rs"));

/// The languages ScriptCraft offers once craft-i18n has a `scriptcraft.tsv` for them. All are
/// left-to-right: right-to-left scripts wait for a right-to-left editor layout (docs/localization-parity.md).
pub const OFFERED: [&str; 12] = ["de", "es", "fr", "hu", "it", "ja", "ko", "nl", "pl", "pt-br", "sv", "zh-hans"];

pub static I18N: LazyLock<craft_i18n::I18n> = LazyLock::new(|| {
    let (i18n, problems) = craft_i18n::I18n::new(CRAFT_I18N, &OFFERED);
    for p in problems {
        log::warn!("translation catalog: {p}");
    }
    i18n
});

thread_local! {
    static CURRENT: Cell<Lang> = const { Cell::new(Lang::EN) };
}

/// The interface language.
pub fn current() -> Lang {
    CURRENT.get()
}

pub fn set_current(lang: Lang) {
    CURRENT.set(lang);
}

/// Use the system language when ScriptCraft offers it.
pub fn use_system_language() {
    set_current(I18N.system_lang());
}

/// Translate an interface string.
pub fn tr(source: &'static str) -> &'static str {
    I18N.t(current(), source)
}

/// Translate a command's label by its id (`@id file.save` rows), else the English label.
pub fn tr_command(id: &str, label: &'static str) -> &'static str {
    I18N.tid(current(), id, label)
}

/// Translate a template and fill `{name}` placeholders.
pub fn trf(source: &'static str, args: &[(&str, &str)]) -> String {
    fmt(tr(source), args)
}

/// The languages the interface can be shown in now: English plus every offered language that is
/// embedded.
pub fn languages() -> Vec<(Lang, String)> {
    I18N.languages().into_iter().map(|l| (l, I18N.name(l).to_owned())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_without_catalogs() {
        assert_eq!(tr("Save"), "Save");
        assert_eq!(trf("Page {page} of {pages}", &[("page", "2"), ("pages", "9")]), "Page 2 of 9");
        assert!(I18N.check_all().is_empty());
    }
}
