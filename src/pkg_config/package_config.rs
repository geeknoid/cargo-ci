use cargo_metadata::Package;
use std::collections::HashMap;

pub fn get_variables(p: &Package) -> Option<HashMap<String, String>> {
    p.metadata.get("ci")?.get("variables").and_then(|v| v.as_object()).map(|table| {
        table
            .iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
            .collect::<HashMap<String, String>>()
    })
}
