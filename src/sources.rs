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

use std::path::PathBuf;

use yaml_rust::Yaml;
use regex::Regex;
use faccess::PathExt;


/// A playbook source
pub struct Source {
    pub name: String,
    pub path: PathBuf,
    pub recurse: bool,
    pub playbook_match: Regex,
    pub pre_provision: Option<String>
}

impl Source {
    /// Parses YAML for a playbook source
    pub fn parse(name: String, yaml: &Yaml) -> Result<Self, String> {
        Ok(Self {
            name,
            path: match &yaml["path"] {
                Yaml::String(s) => {
                    let path = PathBuf::from(s);
                    match path.is_dir() && path.readable() {
                        true => path,
                        false => return Err(format!("failed to read at {}", path.to_str().unwrap()))
                    }
                },
                Yaml::BadValue => return Err("missing path parameter".to_string()),
                _ => return Err("expected string for the path parameter".to_string())
            },

            recurse: match yaml["recurse"] {
                Yaml::Boolean(b) => b,
                Yaml::BadValue => false,
                _ => return Err("expected boolean for the recurse source parameter".to_string())
            },

            playbook_match: match &yaml["playbook_match"] {
                Yaml::String(s) => match Regex::new(&s) {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_string())
                },
                Yaml::BadValue => Regex::new(r#"\.ya?ml$"#).unwrap(),
                _ => return Err("expected string for the playbook_match source parameter".to_string())
            },

            pre_provision: match &yaml["pre_provision"] {
                Yaml::String(s) => Some(s.clone()),
                Yaml::BadValue => None,
                _ => return Err("expected string for the pre_provision source parameter".to_string())
            }
        })
    }
}
