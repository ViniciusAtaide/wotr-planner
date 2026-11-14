//! What a tool's response costs an agent: a few fixed calls on the real data, each checked against its tool's output
//! schema (a client checks it too, and refuses a response that breaks it), for an empty list, an empty map or a
//! `null` below the top level (a row leaves out what it doesn't have), and against a size budget. A budget that breaks
//! asks whether the new characters say something new: a fact said twice, or fixed text sent on every call, belongs
//! elsewhere (`CODING_STANDARDS.md`, "Each fact once"). Raise it only for a new fact.

use rmcp::serde_json::{Map, Value, json};

use crate::support::*;

/// A human fighter 8 with a greatsword: the attack line's Build.
fn fighter() -> Value {
    json!({
        "start.race": "human",
        "start.faith": "atheism",
        "start.alignment": "true-neutral",
        "start.score.str": 17,
        "start.score.dex": 13,
        "start.score.con": 14,
        "level.1.class": {"class": "fighter"},
        "level.2.class": {"class": "fighter"},
        "level.3.class": {"class": "fighter"},
        "level.4.class": {"class": "fighter"},
        "level.5.class": {"class": "fighter"},
        "level.6.class": {"class": "fighter"},
        "level.7.class": {"class": "fighter"},
        "level.8.class": {"class": "fighter"},
        "slot.feat.1.class.0": "power-attack",
        "slot.feat.1.race.0": {"key": "weapon-focus", "parameter": "greatsword"},
        "slot.feat.1.general.0": "dodge",
        "slot.feat.2.class.0": "cleave",
        "slot.feat.4.class.0": {"key": "weapon-specialization", "parameter": "greatsword"},
        "level.4.increase": "str",
        "slot.feat.5.general.0": "great-cleave",
        "slot.feat.8.class.0": {"key": "improved-critical", "parameter": "greatsword"},
        "level.8.increase": "str",
    })
}

/// An elf rogue whose Power Attack fails for Strength, with the two feats that need it and a feat the data lacks.
fn failing_rogue() -> Value {
    json!({
        "start.race": "elf",
        "start.faith": "atheism",
        "start.alignment": "true-neutral",
        "start.score.str": 10,
        "start.score.dex": 16,
        "level.1.class": {"class": "rogue"},
        "level.2.class": {"class": "rogue"},
        "level.3.class": {"class": "rogue"},
        "level.4.class": {"class": "rogue"},
        "level.5.class": {"class": "rogue"},
        "slot.feat.1.general.0": "power-attack",
        "slot.feat.3.general.0": "cleave",
        "slot.feat.5.general.0": "great-cleave",
        "slot.feat.4.class.0": "dodge",
    })
}

/// The declared Build's code.
async fn declared(client: &Client, choices: Value) -> String {
    let validated = client.ok("validate", json!({ "choices": choices })).await;
    assert_eq!(validated["status"], "valid", "{validated}");
    text(&validated["build"])
}

/// The fixed calls, by name, with each tool, its size budget in characters and its result.
async fn calls(client: &Client) -> Vec<(&'static str, &'static str, usize, Value)> {
    let fighter = declared(client, fighter()).await;
    let mut results = Vec::new();
    // Each budget is about a tenth above the size when it was set (2026-11-14).
    for (name, tool, budget, arguments) in [
        (
            "validate, failing",
            "validate",
            600,
            json!({ "choices": failing_rogue() }),
        ),
        (
            "sheet, fighter 8",
            "sheet",
            14_500,
            json!({ "build": fighter }),
        ),
        (
            "options, a feat slot",
            "options",
            15_500,
            json!({ "build": fighter, "choice": "slot.feat.7.general.0" }),
        ),
        (
            "describe, fighter",
            "describe",
            6_100,
            json!({ "kind": "class", "key": "fighter" }),
        ),
        (
            "search, power",
            "search",
            4_300,
            json!({ "query": "power" }),
        ),
    ] {
        results.push((name, tool, budget, client.ok(tool, arguments).await));
    }
    results
}

#[tokio::test]
async fn real_data_responses_fit_their_schemas_leave_out_empties_and_stay_in_budget() {
    let client = Client::start().await;
    let mut problems = Vec::new();
    for (name, tool, budget, result) in calls(&client).await {
        let schema = client.output_schema(tool).await;
        conforms(&schema, &schema, &result, name, &mut problems);
        if let Value::Object(fields) = &result {
            for (field, value) in fields {
                empties(value, &format!("{name}: {field}"), false, &mut problems);
            }
        }
        let size = result.to_string().len();
        if size > budget {
            problems.push(format!(
                "{name}: {size} characters, over its budget of {budget}"
            ));
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Each empty list, empty map or `null` inside a top-level field's value (`inside`: below that field itself).
fn empties(value: &Value, path: &str, inside: bool, problems: &mut Vec<String>) {
    match value {
        Value::Null if inside => problems.push(format!("{path} is null")),
        Value::Array(items) if inside && items.is_empty() => {
            problems.push(format!("{path} is []"));
        }
        Value::Object(fields) if inside && fields.is_empty() => {
            problems.push(format!("{path} is {{}}"));
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                empties(item, &format!("{path}[{index}]"), true, problems);
            }
        }
        Value::Object(fields) => {
            for (field, item) in fields {
                empties(item, &format!("{path}.{field}"), true, problems);
            }
        }
        _ => {}
    }
}

/// Whether a value fits a JSON Schema, for the parts schemars writes: `$ref` into `$defs`, `type`, `const`, `enum`,
/// `properties`, `required`, `additionalProperties`, `items`, `oneOf`, `anyOf` and `allOf`. Each misfit goes into
/// `problems` with its path.
fn conforms(root: &Value, schema: &Value, value: &Value, path: &str, problems: &mut Vec<String>) {
    let Value::Object(schema) = schema else {
        return;
    };
    if let Some(Value::String(reference)) = schema.get("$ref") {
        let name = reference.trim_start_matches("#/$defs/");
        conforms(root, &root["$defs"][name], value, path, problems);
    }
    for (keyword, needs_all) in [("allOf", true), ("anyOf", false), ("oneOf", false)] {
        let Some(Value::Array(branches)) = schema.get(keyword) else {
            continue;
        };
        let misfits: Vec<Vec<String>> = branches
            .iter()
            .map(|branch| {
                let mut misfit = Vec::new();
                conforms(root, branch, value, path, &mut misfit);
                misfit
            })
            .collect();
        let fits = misfits.iter().filter(|misfit| misfit.is_empty()).count();
        if needs_all && fits < branches.len() {
            problems.extend(misfits.into_iter().flatten());
        } else if !needs_all && fits == 0 {
            let closest = misfits.into_iter().min_by_key(Vec::len).unwrap_or_default();
            problems.push(format!(
                "{path} fits none of its {keyword}; the closest: {closest:?}"
            ));
        }
    }
    if let Some(types) = schema.get("type") {
        let allowed: Vec<&str> = match types {
            Value::String(one) => vec![one.as_str()],
            Value::Array(many) => many.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        if !allowed.iter().any(|kind| is_of(value, kind)) {
            problems.push(format!("{path} is not {allowed:?}: {value}"));
            return;
        }
    }
    if let Some(constant) = schema.get("const")
        && constant != value
    {
        problems.push(format!("{path} is not {constant}: {value}"));
    }
    if let Some(Value::Array(options)) = schema.get("enum")
        && !options.contains(value)
    {
        problems.push(format!("{path} is none of {options:?}: {value}"));
    }
    match value {
        Value::Object(fields) => object(root, schema, fields, path, problems),
        Value::Array(items) => {
            if let Some(item) = schema.get("items") {
                for (index, value) in items.iter().enumerate() {
                    conforms(root, item, value, &format!("{path}[{index}]"), problems);
                }
            }
        }
        _ => {}
    }
}

/// An object's `required`, `properties` and `additionalProperties`.
fn object(
    root: &Value,
    schema: &Map<String, Value>,
    fields: &Map<String, Value>,
    path: &str,
    problems: &mut Vec<String>,
) {
    if let Some(Value::Array(required)) = schema.get("required") {
        for field in required.iter().filter_map(Value::as_str) {
            if !fields.contains_key(field) {
                problems.push(format!("{path} has no required `{field}`"));
            }
        }
    }
    let properties = schema.get("properties").and_then(Value::as_object);
    for (field, value) in fields {
        let field_path = format!("{path}.{field}");
        match properties.and_then(|properties| properties.get(field)) {
            Some(property) => conforms(root, property, value, &field_path, problems),
            None => match schema.get("additionalProperties") {
                Some(Value::Bool(false)) => {
                    problems.push(format!("{field_path} isn't in the schema"))
                }
                Some(extra @ Value::Object(_)) => {
                    conforms(root, extra, value, &field_path, problems)
                }
                _ => {}
            },
        }
    }
}

/// Whether a value is of a JSON Schema type.
fn is_of(value: &Value, kind: &str) -> bool {
    match kind {
        "null" => value.is_null(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        _ => true,
    }
}
