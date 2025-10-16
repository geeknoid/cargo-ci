use cargo_metadata::Package;

pub fn variables(p: &Package) -> impl Iterator<Item = (&str, &str)> {
    p.metadata
        .get("ci")
        .and_then(|ci| ci.get("variables"))
        .and_then(|v| v.as_object())
        .into_iter()
        .flat_map(|table| table.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.as_str(), s))))
}
