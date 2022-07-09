/* Set Me Up, a minimalistic Ansible-based remote provisioning tool
 * Copyright (C) 2021 Julien JPK (jjpk.me)

 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.

 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.

 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>. */


//! Utility functions to run external processes


use std::path::Path;
use std::collections::HashMap;
use std::process::{Command, Stdio};


/// Executes the given program as an external process
pub fn run(program: &str, args: Vec<&str>, working_dir: &Path,
           env: Option<&HashMap<String, String>>,
           is_ansible: bool) -> Result<String, String> {
    let mut command = Command::new(program);
    if let Some(e) = env {
        command.envs(e);
    }

    command.args(args)
        .current_dir(working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    match command.output() {
        Ok(o) => match o.status.success() {
            true => Ok(String::from_utf8_lossy(&o.stdout).to_string()),
            false => match is_ansible {
                true => {
                    let output = String::from_utf8_lossy(&o.stdout).to_string();
                    match output.starts_with("{") {
                        true => Ok(output),
                        false => Err(format!("{}\n{}", output, String::from_utf8_lossy(&o.stderr).to_string()))
                    }
                },
                false => Err(format!("{}", String::from_utf8_lossy(&o.stderr).to_string()))
            }
        },
        Err(e) => Err(format!("{}", e))
    }
}

/// Executes the given command-line through a shell in a new process
pub fn shell(cmdline: &String, working_dir: &Path,
             env: Option<&HashMap<String, String>>) -> Result<String, String> {
    run("sh", vec!["-c", &cmdline], working_dir, env, false)
}
