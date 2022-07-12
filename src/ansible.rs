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


//! Ansible interface


use crate::exec;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io::Write;
use faccess::PathExt;
use yaml_rust::Yaml;
use serde_json;
use serde_json::Value as Json;
use tempfile::NamedTempFile;


/// Parameters to use when invoking ansible-playbook
pub struct AnsibleContext {
    pub path: Option<PathBuf>,
    pub env: HashMap<String, String>
}

/// ansible-playbook task result
pub struct AnsibleTaskResult {
    pub name: String,
    pub success: bool,
    pub changed: bool,
    pub message: String
}

/// Collection of task results
pub type AnsibleResult = Vec<AnsibleTaskResult>;

impl AnsibleContext {
    /// Handles parsing the path to ansible-playbook as well as the args and env we should use
    pub fn parse(yaml: &Yaml) -> Result<AnsibleContext, String> {
        Ok(Self {
            path: match &yaml["path"] {
                Yaml::BadValue => None,
                Yaml::String(s) => {
                    let path = PathBuf::from(s);
                    match path.is_file() && path.executable() {
                        true => Some(path),
                        false => return Err(format!("no executable ansible-playbook at {}", path.to_str().unwrap()))
                    }
                },
                _ => return Err("expected string for the ansible-playbook path".to_string())
            },

            env: match &yaml["env"] {
                Yaml::BadValue => HashMap::new(),
                Yaml::Array(a) => a.iter().map(|i| Ok((
                    match &i["name"] {
                        Yaml::String(s) => String::from(s),
                        Yaml::BadValue => return Err("missing name property for environment variable".to_string()),
                        _ => return Err("non-string name property for environment variable".to_string())
                    },
                    match &i["value"] {
                        Yaml::String(s) => String::from(s),
                        Yaml::BadValue => return Err("missing value property for environment variable".to_string()),
                        _ => return Err("non-string value property for environment variable".to_string())
                    }))).collect::<Result<HashMap<String, String>, String>>()?,
                _ => return Err("expected list for the ansible-playbook environment".to_string())
            }
        })
    }

    /// Runs ansible-playbook for provisioning
    pub fn execute(&self, key_path: &Path, inventory_path: &Path,
                   playbook_path: &Path, source_dir_path: &Path) -> Result<AnsibleResult, String> {
        let mut env = self.env.clone();
        env.insert("ANSIBLE_CALLBACKS_ENABLED".into(), "ansible.posix.json".into());
        env.insert("ANSIBLE_STDOUT_CALLBACK".into(), "ansible.posix.json".into());
        env.insert("ANSIBLE_HOST_KEY_CHECKING".into(), "False".into());

        let playbook_fullpath = source_dir_path.join(playbook_path);
        let mut play_file = NamedTempFile::new().map_err(|e| format!("failed to ready the temporary play: {}", e))?;
        play_file.write(format!(
            concat!(
                "- ansible.builtin.import_playbook: {}\n",
                "- hosts: all\n",
                "  gather_facts: no\n",
                "  tasks:\n",
                "    - name: Closing connection\n",
                "      ansible.builtin.meta: reset_connection\n"
            ),
            playbook_fullpath.to_str().unwrap()
        ).as_bytes()).map_err(|e| format!("failed to write the temporary play: {}", e))?;

        let ansible_run = exec::run(
            match &self.path {
                Some(p) => p.as_path().to_str().unwrap(),
                None => "ansible-playbook"
            },
            vec!(
                "--private-key", key_path.to_str().unwrap(),
                "-i", inventory_path.to_str().unwrap(),
                play_file.path().to_str().unwrap()
            ),
            source_dir_path,
            Some(&env),
            true
        );

        let ansible_run = ansible_run?;
        let ansible_json: Json = serde_json::from_str(&ansible_run).map_err(|e| e.to_string())?;
        let plays = ansible_json["plays"].as_array().ok_or("missing plays array".to_string())?;
        let tasks = plays.iter().map(|p| p["tasks"].as_array().unwrap()).flatten();
        let results = tasks.map(|t| {
            let task_result = &t["hosts"]["provisionee"];
            let success = task_result["failed"].as_bool().unwrap_or(false);
            let unreachable = task_result["unreachable"].as_bool().unwrap_or(false);
            AnsibleTaskResult {
                name: String::from(t["task"]["name"].as_str().unwrap_or("unnamed task")),
                success: unreachable || !success,
                changed: task_result["changed"].as_bool().unwrap_or(false),
                message: String::from(task_result["msg"].as_str().unwrap_or("no details"))
            }
        });

        Ok(results.collect())
    }
}

impl Default for AnsibleContext {
    /// Defaults for when no ansible_playbook block is given
    fn default() -> Self {
        Self {
            path: None,
            env: HashMap::new()
        }
    }
}
