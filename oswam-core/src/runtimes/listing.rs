use super::firmlinked;
use std::path::{Path, PathBuf};

const HEADER: &str = "== Disk Images ==";
const TOTAL: &str = "Total Disk Images:";

pub fn registered_images(listing: &str) -> Option<Vec<PathBuf>> {
    if !listing.contains(HEADER) {
        return None;
    }
    let images: Vec<PathBuf> = listing
        .lines()
        .filter_map(|l| l.trim().strip_prefix("Image Path:"))
        .map(|p| firmlinked(Path::new(p.trim())))
        .collect();
    (declared_total(listing)? == images.len()).then_some(images)
}

fn declared_total(listing: &str) -> Option<usize> {
    listing
        .lines()
        .find_map(|l| l.trim().strip_prefix(TOTAL))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}
