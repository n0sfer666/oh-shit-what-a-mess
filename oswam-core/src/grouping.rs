use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const GROUPED_EXTENSIONS: &[&str] = &["avd", "ini"];

pub struct Group {
    pub primary: PathBuf,
    pub companions: Vec<PathBuf>,
}

pub fn group_by_stem<P>(children: &[PathBuf], extensions: &[&str], is_primary: P) -> Vec<Group>
where
    P: Fn(&Path) -> bool,
{
    let mut order: Vec<String> = Vec::new();
    let mut buckets: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for child in children {
        let key = stem_of(child, extensions);
        if !buckets.contains_key(&key) {
            order.push(key.clone());
        }
        buckets.entry(key).or_default().push(child.clone());
    }
    order
        .into_iter()
        .filter_map(|key| buckets.remove(&key))
        .map(|members| split_primary(members, &is_primary))
        .collect()
}

fn stem_of(path: &Path, extensions: &[&str]) -> String {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());
    let groupable = path
        .extension()
        .map(|e| extensions.iter().any(|known| e == *known))
        .unwrap_or(false);
    if !groupable {
        return name;
    }
    match name.rsplit_once('.') {
        Some((stem, _)) => stem.to_string(),
        None => name,
    }
}

fn split_primary<P>(mut members: Vec<PathBuf>, is_primary: &P) -> Group
where
    P: Fn(&Path) -> bool,
{
    let idx = members.iter().position(|p| is_primary(p)).unwrap_or(0);
    let primary = members.remove(idx);
    Group {
        primary,
        companions: members,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    fn dirs(names: &[&str]) -> impl Fn(&Path) -> bool {
        let owned: Vec<String> = names.iter().map(|n| n.to_string()).collect();
        move |p: &Path| {
            owned
                .iter()
                .any(|n| p.file_name().is_some_and(|f| f == n.as_str()))
        }
    }

    #[test]
    fn pairs_avd_dir_with_its_ini() {
        let children = paths(&["/avd/Pixel_8a.ini", "/avd/Pixel_8a.avd"]);
        let groups = group_by_stem(&children, GROUPED_EXTENSIONS, dirs(&["Pixel_8a.avd"]));
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].primary, PathBuf::from("/avd/Pixel_8a.avd"));
        assert_eq!(
            groups[0].companions,
            vec![PathBuf::from("/avd/Pixel_8a.ini")]
        );
    }

    #[test]
    fn distinct_stems_stay_separate() {
        let children = paths(&["/avd/A.avd", "/avd/A.ini", "/avd/B.avd", "/avd/B.ini"]);
        let groups = group_by_stem(&children, GROUPED_EXTENSIONS, dirs(&["A.avd", "B.avd"]));
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|g| g.companions.len() == 1));
    }

    #[test]
    fn lone_child_has_no_companions() {
        let children = paths(&["/avd/Solo.avd"]);
        let groups = group_by_stem(&children, GROUPED_EXTENSIONS, dirs(&["Solo.avd"]));
        assert_eq!(groups.len(), 1);
        assert!(groups[0].companions.is_empty());
    }

    #[test]
    fn falls_back_to_first_member_when_no_primary() {
        let children = paths(&["/avd/X.ini", "/avd/X.lock"]);
        let groups = group_by_stem(&children, &["ini", "lock"], |_| false);
        assert_eq!(groups[0].primary, PathBuf::from("/avd/X.ini"));
        assert_eq!(groups[0].companions, vec![PathBuf::from("/avd/X.lock")]);
    }

    #[test]
    fn dotted_names_are_not_collapsed_into_one_group() {
        let children = paths(&["/avd/com.example.a", "/avd/com.example.b"]);
        let groups = group_by_stem(&children, GROUPED_EXTENSIONS, |_| false);
        assert_eq!(groups.len(), 2);
        assert!(groups.iter().all(|g| g.companions.is_empty()));
    }

    #[test]
    fn preserves_input_order_of_groups() {
        let children = paths(&["/avd/Z.avd", "/avd/A.avd"]);
        let groups = group_by_stem(&children, GROUPED_EXTENSIONS, dirs(&["Z.avd", "A.avd"]));
        let names: Vec<PathBuf> = groups.into_iter().map(|g| g.primary).collect();
        assert_eq!(names, paths(&["/avd/Z.avd", "/avd/A.avd"]));
    }
}
