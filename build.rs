#[cfg(feature = "native-pack")]
#[allow(dead_code)]
#[path = "src/contract.rs"]
mod contract;
#[cfg(feature = "native-pack")]
#[path = "src/metadata.rs"]
mod metadata;

fn main() {
    #[cfg(feature = "native-pack")]
    write_native_catalog();
}

#[cfg(feature = "native-pack")]
fn write_native_catalog() {
    use serde_json::json;
    use std::path::PathBuf;

    println!("cargo:rerun-if-changed=reference/contracts/components.json");
    println!("cargo:rerun-if-changed=src/contract.rs");
    println!("cargo:rerun-if-changed=src/metadata.rs");
    let entries: Vec<_> = metadata::tag_names()
        .iter()
        .map(|&tag| {
            let default_state = contract::default_state(tag)
                .unwrap_or_else(|| panic!("missing contract for {tag}"));
            json!({ "tag": tag, "default_state": default_state })
        })
        .collect();
    let path = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"))
        .join("native-catalog.json");
    std::fs::write(path, serde_json::to_vec(&entries).expect("catalog JSON"))
        .expect("write native catalog");

    println!("cargo:rerun-if-changed=src/generated/defaults.json");
    let defaults: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("src/generated/defaults.json"))
            .expect("generated render defaults");
    let mut source =
        String::from("fn native_default(tag: &str) -> Option<&'static str> { match tag {\n");
    for entry in defaults {
        let tag = entry["tag"].as_str().expect("default render tag");
        source.push_str(&format!("{tag:?} => Some({:?}),\n", entry.to_string()));
    }
    source.push_str("_ => None,\n} }\n");
    let path = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"))
        .join("native-defaults.rs");
    std::fs::write(path, source).expect("write native render defaults");
}
