use serde_json::{json, Value};

fn main() {
    let mut args = std::env::args().skip(1);
    let tag = args.next().expect("usage: inspect <tag> [state-json]");
    let state: Value = args
        .next()
        .map(|value| serde_json::from_str(&value).expect("state must be valid JSON"))
        .unwrap_or_else(|| json!({}));

    let rendered = wokwi_elements_rs::render(&tag, &state)
        .unwrap_or_else(|| panic!("unknown component: {tag}"));
    println!("styles:\n{}\n", rendered.styles);
    println!("html:\n{}", rendered.html);
    println!(
        "\npinInfo:\n{}",
        serde_json::to_string_pretty(
            &wokwi_elements_rs::pin_info(&tag, &state).unwrap_or_default()
        )
        .unwrap()
    );
}
