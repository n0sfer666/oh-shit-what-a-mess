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
                entry.size_label(),
                entry.display
            );
        }
    }
    println!("\nИтого: {}", human_bytes(result.total_bytes));
}

pub fn print_tips(elevated: bool) {
    if elevated {
        println!("\n💡 Режим sudo: + снимки Time Machine (последний сохраняется)");
        println!("   + системные кэши /Library/Caches (риск: временные сбои, желателен рестарт).");
        println!("   Снимки — не замена внешнему бэкапу; система создаёт их не просто так.");
    } else {
        println!("\n💡 Для более глубокой очистки перезапусти с sudo:");
        println!("      sudo oswam        # + снимки Time Machine + системные кэши");
        println!("   Риск: снимки — локальный recovery point (не замена бэкапу);");
        println!(
            "   системные кэши — временные сбои до пересоздания. Последний снимок не трогаем."
        );
    }
}

pub fn summary_lines(manifest: &Manifest, dry_run: bool, failures: usize) -> Vec<String> {
    let head = if dry_run {
        "Превью"
    } else {
        "Готово"
    };
    let mut lines = vec![format!(
        "{head}: {} элементов, {} физически.",
        manifest.entries.len(),
        human_bytes(manifest.total_bytes())
    )];
    let freed = manifest.bytes_with_action("permanent") + manifest.bytes_with_action("native");
    let trashed = manifest.bytes_with_action("trash");
    if freed > 0 {
        lines.push(if dry_run {
            format!("Освободилось бы сразу: {}", human_bytes(freed))
        } else {
            format!("Освобождено сразу: {}", human_bytes(freed))
        });
    }
    if trashed > 0 {
        lines.push(if dry_run {
            format!(
                "Ушло бы в Корзину: {} — место освободится после её очистки.",
                human_bytes(trashed)
            )
        } else {
            format!(
                "Перемещено в Корзину: {} — место освободится после её очистки.",
                human_bytes(trashed)
            )
        });
    }
    if failures > 0 {
        lines.push(if dry_run {
            format!("Не удалось бы: {failures} элементов (нет доступа или занято).")
        } else {
            format!("Не удалось: {failures} элементов (нет доступа или занято).")
        });
    }
    lines
}

pub fn print_summary(manifest: &Manifest, dry_run: bool, failures: usize) {
    println!();
    for line in summary_lines(manifest, dry_run, failures) {
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::summary_lines;
    use oswam_core::manifest::Manifest;
    use std::path::Path;

    fn manifest(action: &str) -> Manifest {
        let mut m = Manifest::default();
        m.record(
            Path::new("/Users/tester/Library/Caches/x"),
            2048,
            action,
            "2026-07-25T00:00:00Z",
        );
        m
    }

    #[test]
    fn a_preview_never_claims_that_anything_already_moved() {
        for action in ["trash", "permanent", "native"] {
            for line in summary_lines(&manifest(action), true, 1) {
                assert!(
                    !line.contains("Перемещено") && !line.contains("Освобождено"),
                    "{line}"
                );
            }
        }
    }

    #[test]
    fn a_real_run_reports_in_the_past_tense() {
        let lines = summary_lines(&manifest("trash"), false, 0).join("\n");
        assert!(lines.contains("Готово"));
        assert!(lines.contains("Перемещено в Корзину"));
    }
}
