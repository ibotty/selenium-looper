use std::collections::HashMap;

use anyhow::{anyhow, Context};
use chrono::Utc;
use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::AppError;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Script {
    pub name: String,
    pub creation_date: String,
    pub commands: Vec<Command>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Command {
    pub command: String,
    pub target: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    pub value: String,
    pub description: String,
}

pub fn cmd(command: &str, target: &str, value: &str) -> Command {
    Command {
        command: command.to_string(),
        target: target.to_string(),
        targets: vec![],
        value: value.to_string(),
        description: "".to_string(),
    }
}

pub fn generate_loop_script(
    scripts: HashMap<String, Script>,
    mut data: Vec<Map<String, Value>>,
) -> Result<Script, AppError> {
    if scripts.len() != 1 {
        return Err(AppError::OtherError(anyhow!(
            "only one script supported ATM"
        )));
    }

    // this is safe because of the assertion above.
    let mut script = scripts.into_values().next().unwrap();

    if data.is_empty() {
        return Err(AppError::OtherError(anyhow!("empty input data")));
    }

    // this is safe because of the assertion above.
    let first_element = data.pop().unwrap();
    data.push(first_element.clone());

    let name = format!("looped {}", script.name);
    let creation_date = Utc::now().format("%Y-%d-%m").to_string();
    let data_ident = "data";
    let row_ident = "row";
    let mut commands = vec![
        declare_array(data, data_ident)?,
        foreach(data_ident, row_ident),
        console_log(&format!("processing row ${{{}}}", row_ident)),
    ];
    commands.extend(first_element.keys().map(|k| export_row_variable(k)));
    commands.extend(
        first_element
            .keys()
            .map(|k| echo(&format!("processing row.{} ${{{}}}", k, k))),
    );
    commands.append(&mut script.commands);
    commands.push(end());

    Ok(Script {
        name,
        creation_date,
        commands,
    })
}

pub fn export_row_variable(ident: &str) -> Command {
    let scriptlet = format!("return ${{row}}['{}'];", ident);
    cmd("ExecuteScript", &scriptlet, ident)
}

pub fn declare_array(data: Vec<Map<String, Value>>, ident: &str) -> Result<Command, AppError> {
    let scriptlet = format!(
        "return {};",
        serde_json::to_string(&data).context("Could not serialize data")?,
    );

    Ok(cmd("ExecuteScript", &scriptlet, ident))
}

pub fn foreach(target: &str, value: &str) -> Command {
    cmd("forEach", target, value)
}

pub fn end() -> Command {
    cmd("end", "", "")
}

pub fn console_log(line: &str) -> Command {
    let scriptlet = format!("console.log(`{}`);", line);
    cmd("ExecuteScript", &scriptlet, "")
}

pub fn echo(target: &str) -> Command {
    cmd("echo", target, "")
}
