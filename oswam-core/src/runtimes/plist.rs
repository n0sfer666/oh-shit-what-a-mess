pub fn plist_string(xml: &str, key: &str) -> Option<String> {
    let (_, after) = xml.split_once(&format!("<key>{key}</key>"))?;
    let value = after.trim_start().strip_prefix("<string>")?;
    let (value, _) = value.split_once("</string>")?;
    Some(value.trim().to_string())
}
