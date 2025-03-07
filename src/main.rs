use std::process::Command;

use json::{object::Object, JsonValue};

fn json_cmd(command: &str, args: &[&str]) -> JsonValue {
    let output_str = String::from_utf8(
        Command::new(command)
            .args(args.to_vec())
            .output()
            .expect(&format!(
                "Unable to run command {command} with args {args:?}"
            ))
            .stdout,
    )
    .expect(&format!(
        "Unable to convert command {command} {args:?} output to string"
    ));

    json::parse(&output_str).expect("Unable to parse command {command} {args:?} output as JSON")
}

fn main() {
    let active_workspace_data: Object = match json_cmd("hyprctl", &["activeworkspace", "-j"]) {
        JsonValue::Object(object) => object,
        _ => panic!("hyprctl activeworkspace response is not a single JSON object"),
    };

    let active_workspace_id: i64 = match active_workspace_data
        .get("id")
        .expect("No field id in active workspace data")
    {
        JsonValue::Number(number) => number
            .as_fixed_point_i64(0)
            .expect("Unable to cast workspace id to i64"),
        _ => panic!("Active workspace id is not a number"),
    };

    println!("{active_workspace_id}")
}
