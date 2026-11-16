use serde_json::{Map, Value};

const CONTRACTS_JSON: &str = include_str!("../reference/contracts/components.json");

pub fn contracts_json() -> &'static str {
    CONTRACTS_JSON
}

pub fn contract(tag: &str) -> Option<Value> {
    let contracts: Value = serde_json::from_str(CONTRACTS_JSON).ok()?;
    contracts
        .as_array()?
        .iter()
        .find(|component| component.get("tag").and_then(Value::as_str) == Some(tag))
        .cloned()
}

pub fn default_state(tag: &str) -> Option<Value> {
    let contract = contract(tag)?;
    let properties = contract.get("properties")?.as_object()?;
    let mut state = Map::new();

    for (name, descriptor) in properties {
        let default = descriptor.get("default")?;
        let value = match default.get("kind").and_then(Value::as_str) {
            Some("zeros") => {
                let length = default.get("length").and_then(Value::as_u64).unwrap_or(0);
                Value::Array((0..length).map(|_| Value::from(0)).collect())
            }
            _ => default.get("value").cloned().unwrap_or(Value::Null),
        };
        let value = numeric_object_to_array(value);
        if tag == "wokwi-hx711" && name == "type" && value.is_null() {
            continue;
        }
        state.insert(name.clone(), value);
    }

    Some(Value::Object(state))
}

fn numeric_object_to_array(value: Value) -> Value {
    let Some(object) = value.as_object() else {
        return value;
    };
    if object.is_empty() {
        return value;
    }
    let mut indexed = Vec::with_capacity(object.len());
    for (key, entry) in object {
        let Ok(index) = key.parse::<usize>() else {
            return value;
        };
        indexed.push((index, entry.clone()));
    }
    indexed.sort_by_key(|(index, _)| *index);
    if indexed
        .iter()
        .enumerate()
        .all(|(expected, (actual, _))| expected == *actual)
    {
        Value::Array(indexed.into_iter().map(|(_, entry)| entry).collect())
    } else {
        value
    }
}
