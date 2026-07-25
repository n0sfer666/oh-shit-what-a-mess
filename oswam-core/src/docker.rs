use crate::category::NativeSpec;
use std::collections::HashMap;
use std::io;
use std::sync::{Mutex, OnceLock};

pub fn parse_human_size(raw: &str) -> Option<u64> {
    let s = raw.trim();
    let num_end = s.find(|c: char| c.is_alphabetic()).unwrap_or(s.len());
    let (num, unit) = s.split_at(num_end);
    let value: f64 = num.trim().parse().ok()?;
    let mult = match unit.trim().to_ascii_uppercase().as_str() {
        "B" | "" => 1.0,
        "KB" | "K" => 1e3,
        "MB" | "M" => 1e6,
        "GB" | "G" => 1e9,
        "TB" | "T" => 1e12,
        "KIB" => 1024.0,
        "MIB" => 1024.0 * 1024.0,
        "GIB" => 1024.0 * 1024.0 * 1024.0,
        "TIB" => 1024.0_f64.powi(4),
        _ => return None,
    };
    Some((value * mult) as u64)
}

pub fn parse_reclaimable(output: &str) -> u64 {
    parse_reclaimable_filtered(output, &[])
}

pub fn parse_reclaimable_filtered(output: &str, types: &[String]) -> u64 {
    output
        .lines()
        .filter_map(split_row)
        .filter(|(ty, _)| types.is_empty() || types.iter().any(|t| t == ty))
        .filter_map(|(_, size)| parse_human_size(size.split_whitespace().next()?))
        .sum()
}

pub fn parse_reclaimed(output: &str) -> Option<u64> {
    output
        .lines()
        .find_map(|line| line.split_once("reclaimed space:"))
        .and_then(|(_, size)| parse_human_size(size.trim()))
}

fn split_row(line: &str) -> Option<(&str, &str)> {
    if line.trim().is_empty() {
        return None;
    }
    Some(match line.split_once('\t') {
        Some((ty, size)) => (ty.trim(), size),
        None => ("", line),
    })
}

pub fn estimate(spec: &NativeSpec) -> Option<u64> {
    for probe in &spec.probe {
        cached_output(probe, spec.privileged)?;
    }
    if spec.estimate.is_empty() {
        return Some(0);
    }
    let text = cached_output(&spec.estimate, spec.privileged)?;
    Some(if spec.estimate_filter.is_empty() {
        parse_reclaimable(&text)
    } else {
        parse_reclaimable_filtered(&text, &spec.estimate_filter)
    })
}

type OutputCache = Mutex<HashMap<(Vec<String>, bool), Option<String>>>;

fn cache() -> &'static OutputCache {
    static CACHE: OnceLock<OutputCache> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

pub fn clear_estimates() {
    if let Ok(mut map) = cache().lock() {
        map.clear();
    }
}

fn cached_output(command: &[String], privileged: bool) -> Option<String> {
    let key = (command.to_vec(), privileged);
    if let Ok(map) = cache().lock() {
        if let Some(hit) = map.get(&key) {
            return hit.clone();
        }
    }
    let fresh = run_output(command, privileged);
    if let Ok(mut map) = cache().lock() {
        map.insert(key, fresh.clone());
    }
    fresh
}

fn run_output(command: &[String], privileged: bool) -> Option<String> {
    match crate::native::command(command, privileged)?.output() {
        Err(_) => None,
        Ok(out) if !out.status.success() => None,
        Ok(out) => Some(String::from_utf8_lossy(&out.stdout).to_string()),
    }
}

pub fn run_clean(spec: &NativeSpec) -> io::Result<String> {
    crate::native::run_spec(spec)
}

#[cfg(test)]
mod tests;
