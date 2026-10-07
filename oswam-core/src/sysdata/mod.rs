mod caches;

pub use caches::{root_caches, ROOT_CACHES};

use crate::registry::{builtin_categories, SYSTEM_DATA_ID};
use crate::scan::{ScanCategory, ScanEntry, ScanResult};
use crate::select::is_deletable;

pub fn absorb(result: &mut ScanResult, entries: Vec<ScanEntry>) {
    if entries.is_empty() {
        return;
    }
    let added: u64 = entries
        .iter()
        .filter(|e| is_deletable(e))
        .map(|e| e.physical_bytes)
        .sum();
    let index = match result
        .categories
        .iter()
        .position(|c| c.id == SYSTEM_DATA_ID)
    {
        Some(i) => i,
        None => {
            result.categories.insert(0, empty_category());
            0
        }
    };
    let cat = &mut result.categories[index];
    cat.entries.extend(entries);
    cat.total_bytes += added;
    result.total_bytes += added;
}

fn empty_category() -> ScanCategory {
    let (name, glyph) = builtin_categories()
        .into_iter()
        .find(|c| c.id == SYSTEM_DATA_ID)
        .map(|c| (c.name, c.glyph))
        .unwrap_or((SYSTEM_DATA_ID, ""));
    ScanCategory {
        id: SYSTEM_DATA_ID.to_string(),
        name: name.to_string(),
        glyph: glyph.to_string(),
        entries: Vec::new(),
        total_bytes: 0,
    }
}

#[cfg(test)]
mod tests;
