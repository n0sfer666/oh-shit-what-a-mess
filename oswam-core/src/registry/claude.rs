use super::browsers::CACHE_DIRS;
use crate::category::{Category, CleanupKind::*, Target};
use crate::risk::RiskLevel::*;

const APP: &str = "Claude";
const ROOT: &str = "~/Library/Application Support/Claude";

pub fn category() -> Category {
    Category {
        id: "claude",
        name: "Claude Desktop",
        glyph: "✳",
        targets: vec![
            Target::discovered(ROOT, CACHE_DIRS.to_vec(), 0, DeletePath, Caution)
                .blocked_while_running(APP),
            Target::enumerated(&format!("{ROOT}/claude-code"), DeletePath, Caution)
                .versioned()
                .blocked_while_running(APP),
            Target::enumerated(&format!("{ROOT}/claude-code-vm"), DeletePath, Caution)
                .versioned()
                .blocked_while_running(APP),
            Target::new(
                &format!("{ROOT}/vm_bundles/claudevm.bundle/rootfs.img"),
                DeletePath,
                Caution,
            )
            .with_companions(&[".rootfs.img.origin"])
            .blocked_while_running(APP),
        ],
    }
}
