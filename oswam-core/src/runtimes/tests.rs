use super::*;
use crate::config::Config;
use crate::fsops::fake::FakeFs;
use crate::guard::Guard;
use crate::risk::RiskLevel;
use crate::select::is_deletable;

const LISTING: &str = "== Disk Images ==
-- iOS --
iOS 27.0 (24A434) - 87D69AF7
    State: Ready
    Image Path: /System/Library/AssetsV2/com_apple_MobileAsset_iOSSimulatorRuntime/new.asset/AssetData
    Size: 7.5G

Total Disk Images: 1 (7.5G)
";

const HOME: &str = "/Users/tester";

const PLIST: &str = "<plist><dict>
\t<key>MobileAssetProperties</key>
\t<dict>
\t\t<key>Build</key>
\t\t<string>22G86</string>
\t\t<key>SimulatorVersion</key>
\t\t<string>18.6</string>
\t</dict>
</dict></plist>";

fn assets(names: &[&str]) -> FakeFs {
    let mut fs = FakeFs::default();
    let paths: Vec<String> = names.iter().map(|n| format!("{ASSETS}/{n}")).collect();
    let refs: Vec<&str> = paths.iter().map(String::as_str).collect();
    fs.dir(ASSETS, &refs);
    for path in &paths {
        if path.ends_with(".asset") {
            let data = format!("{path}/AssetData");
            fs.dir(path, &[data.as_str()]);
            fs.file(&data, 100);
        } else {
            fs.file(path, 1);
        }
    }
    fs
}

#[test]
fn only_an_asset_simctl_does_not_know_is_an_orphan() {
    let fs = assets(&["new.asset", "old.asset", "catalog.xml"]);
    let registered = registered_images(LISTING).expect("listing");
    let orphans = orphan_assets(&fs, Path::new(ASSETS), &registered);
    assert_eq!(orphans, vec![PathBuf::from(format!("{ASSETS}/old.asset"))]);
}

#[test]
fn an_image_reported_through_the_data_volume_is_still_registered() {
    let fs = assets(&["new.asset"]);
    let listing = LISTING.replace("/System/Library", "/System/Volumes/Data/System/Library");
    let registered = registered_images(&listing).expect("listing");
    assert!(orphan_assets(&fs, Path::new(ASSETS), &registered).is_empty());
}

#[test]
fn an_unrecognised_listing_claims_no_orphans() {
    assert!(registered_images("").is_none());
    assert!(registered_images("xcrun: error: unable to find utility").is_none());
}

#[test]
fn a_listing_whose_paths_do_not_add_up_claims_no_orphans() {
    let unparsed = LISTING.replace("Image Path:", "Image Location:");
    assert!(registered_images(&unparsed).is_none());
    let untotalled = LISTING.replace("Total Disk Images: 1 (7.5G)", "");
    assert!(registered_images(&untotalled).is_none());
}

#[test]
fn an_orphan_is_named_by_its_ios_version() {
    let asset = Path::new("/x/old.asset");
    assert_eq!(
        label(Some(PLIST), asset),
        "iOS 18.6 (22G86) — осиротевший рантайм"
    );
    assert_eq!(label(None, asset), "old.asset — осиротевший рантайм");
}

#[test]
fn a_key_without_a_string_value_is_not_read_from_its_neighbour() {
    let xml = "<key>SimulatorVersion</key><dict/><key>Build</key><string>22G86</string>";
    assert_eq!(plist_string(xml, "SimulatorVersion"), None);
}

#[test]
fn an_orphan_is_weighed_but_can_never_be_selected() {
    let fs = assets(&["old.asset"]);
    let asset = PathBuf::from(format!("{ASSETS}/old.asset"));
    let config = Config::default();
    let guard = Guard::new(&fs, &config, Path::new(HOME));
    let entry = orphan_entry(&fs, &guard, &asset, "x".into()).expect("entry");
    assert!(entry.physical_bytes >= 100 * 512);
    assert_eq!(entry.risk, RiskLevel::Never);
    assert!(!is_deletable(&entry));
}

fn found(fs: &FakeFs, config: &Config, xcode: bool, listing: &str) -> Vec<crate::scan::ScanEntry> {
    let guard = Guard::new(fs, config, Path::new(HOME));
    let run = |argv: &[&str]| match argv.first() {
        Some(&"xcode-select") => xcode.then(|| "/Library/Developer".to_string()),
        _ => Some(listing.to_string()),
    };
    find(fs, &guard, run, |_| Some(PLIST.to_string()))
}

#[test]
fn an_orphan_is_found_and_named_from_its_info_plist() {
    let fs = assets(&["new.asset", "old.asset"]);
    let entries = found(&fs, &Config::default(), true, LISTING);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].display, "iOS 18.6 (22G86) — осиротевший рантайм");
}

#[test]
fn without_developer_tools_nothing_is_called_an_orphan() {
    let fs = assets(&["new.asset", "old.asset"]);
    assert!(found(&fs, &Config::default(), false, LISTING).is_empty());
    assert!(found(&fs, &Config::default(), true, "garbage").is_empty());
}

#[test]
fn an_ignored_orphan_stays_hidden() {
    let fs = assets(&["new.asset", "old.asset"]);
    let config = Config {
        ignore_globs: vec!["**/old.asset".into()],
        ..Config::default()
    };
    assert!(found(&fs, &config, true, LISTING).is_empty());
}
