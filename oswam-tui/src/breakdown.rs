use crate::app::App;
use crate::selection::is_bulk_pick;
use oswam_core::risk::RiskLevel;
use oswam_core::select::is_deletable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Share {
    pub risk: RiskLevel,
    pub bytes: u64,
    pub picked: usize,
    pub count: usize,
    pub manual: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Breakdown {
    pub picked_bytes: u64,
    pub reclaimable: u64,
    pub unknown: usize,
    pub shares: Vec<Share>,
    pub info_bytes: u64,
    pub info_count: usize,
    pub blocked_bytes: u64,
    pub blocked_count: usize,
    pub blocked_apps: Vec<String>,
}

impl Breakdown {
    pub fn picked(&self) -> usize {
        self.shares.iter().map(|s| s.picked).sum()
    }

    pub fn count(&self) -> usize {
        self.shares.iter().map(|s| s.count).sum()
    }
}

impl App {
    pub fn reclaimable_total(&self) -> u64 {
        (0..self.result.categories.len())
            .map(|ci| self.breakdown(ci).reclaimable)
            .sum()
    }

    pub fn breakdown(&self, ci: usize) -> Breakdown {
        let mut out = Breakdown::default();
        let Some(cat) = self.result.categories.get(ci) else {
            return out;
        };
        for (ei, e) in cat.entries.iter().enumerate() {
            if e.blocked {
                out.blocked_bytes += e.physical_bytes;
                out.blocked_count += 1;
                if let Some(app) = &e.blocking_app {
                    if !out.blocked_apps.contains(app) {
                        out.blocked_apps.push(app.clone());
                    }
                }
                continue;
            }
            if !is_deletable(e) {
                out.info_bytes += e.physical_bytes;
                out.info_count += 1;
                continue;
            }
            let picked = self.is_selected(ci, ei);
            out.reclaimable += e.physical_bytes;
            if picked {
                out.picked_bytes += e.physical_bytes;
                out.unknown += usize::from(e.size_unknown);
            }
            let at = match out.shares.iter().position(|s| s.risk == e.risk) {
                Some(at) => at,
                None => {
                    out.shares.push(Share {
                        risk: e.risk,
                        bytes: 0,
                        picked: 0,
                        count: 0,
                        manual: 0,
                    });
                    out.shares.len() - 1
                }
            };
            let share = &mut out.shares[at];
            share.bytes += e.physical_bytes;
            share.count += 1;
            share.picked += usize::from(picked);
            share.manual += usize::from(!is_bulk_pick(e));
        }
        out.shares.sort_by_key(|s| s.risk);
        out
    }
}
