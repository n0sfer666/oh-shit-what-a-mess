use oswam_core::format::human_bytes;
use oswam_core::manifest::Manifest;
use oswam_core::scan::ScanResult;

fn symbol(risk: oswam_core::risk::RiskLevel) -> &'static str {
    use oswam_core::risk::RiskLevel::*;
    match risk {
        Safe => "✓",
        Caution => "▲",
        Danger => "✗",
        Never => "⛔",
    }
}

pub fn print_scan(result: &ScanResult) {
    for cat in &result.categories {
        println!(
            "\n{} {}  —  {}",
            cat.glyph,
            cat.name,
            human_bytes(cat.total_bytes)
        );
        for entry in &cat.entries {
            println!(
                "  {} {:<8} {:>10}  {}",
                symbol(entry.risk),
                format!("{:?}", entry.risk),
                human_bytes(entry.physical_bytes),
                entry.display
            );
        }
    }
    println!("\nИтого: {}", human_bytes(result.total_bytes));
}

pub fn print_summary(manifest: &Manifest, dry_run: bool, trash: bool) {
    let head = if dry_run {
        "Превью"
    } else {
        "Готово"
    };
    println!(
        "\n{head}: {} элементов, {} физически.",
        manifest.entries.len(),
        human_bytes(manifest.total_bytes())
    );
    if !dry_run && trash {
        println!("Перемещено в Корзину — место освободится после её очистки.");
    }
}
