use std::process::Command;

use serde_json::Value;

use crate::error::Error;

pub fn json_cmd(command: &str, args: &[&str]) -> Result<Value, Error> {
    let shell_output = match Command::new(command).args(args.to_vec()).output() {
        Ok(output) => output.stdout,
        Err(err) => {
            log::error!("Unable to run command {command} with args {args:?}: {err}");
            return Err(Error::ShellCommand(
                "Unable to run command, check logs for more information",
            ));
        }
    };

    let output_str = match String::from_utf8(shell_output) {
        Ok(value) => value,
        Err(err) => {
            log::error!(
                "Unable to perform string coversion for output of command {command} with args {args:?}: {err}"
            );
            return Err(Error::ShellCommand(
                "Unable to convert command to string, check logs for more information.",
            ));
        }
    };

    match serde_json::from_str(&output_str) {
        Ok(value) => Ok(value),
        Err(err) => {
            log::error!("Unable to parse command {command} {args:?} output as JSON: {err}");
            Err(Error::ShellCommand(
                "Unable to convert command to string, check logs for more information.",
            ))
        }
    }
}
