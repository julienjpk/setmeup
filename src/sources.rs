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

use crate::util;

use std::fmt::Display;
use std::path::PathBuf;
use std::collections::HashMap;

use yaml_rust::Yaml;
use regex::Regex;
use faccess::PathExt;
use walkdir::WalkDir;


/// Parameters to use when invoking ansible-playbook
pub struct AnsibleContext {
    pub path: Option<PathBuf>,
    pub env: HashMap<String, String>
}

impl AnsibleContext {
    /// Handles parsing the path to ansible-playbook as well as the args and env we should use
    fn parse(yaml: &Yaml) -> Result<AnsibleContext, String> {
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


/// A playbook source
pub struct Source {
    pub name: String,
    pub path: PathBuf,
    pub recurse: bool,
    pub playbook_match: Regex,
    pub pre_provision: Option<String>,
    pub ansible: AnsibleContext
}

const DEFAULT_MATCH: &str = r#"\.ya?ml$"#;

impl Source {
    fn new(name: String, path: PathBuf, recurse: bool,
           playbook_match: Regex, pre_provision: Option<String>,
           ansible: AnsibleContext) -> Self {
        Self { name, path, recurse, playbook_match, pre_provision, ansible }
    }

    /// Parses YAML for a playbook source
    pub fn parse(name: String, yaml: &Yaml) -> Result<Self, String> {
        Ok(Self::new(
            name,
            match &yaml["path"] {
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

            match yaml["recurse"] {
                Yaml::Boolean(b) => b,
                Yaml::BadValue => false,
                _ => return Err("expected boolean for the recurse source parameter".to_string())
            },

            match &yaml["playbook_match"] {
                Yaml::String(s) => match Regex::new(&s) {
                    Ok(r) => r,
                    Err(e) => return Err(e.to_string())
                },
                Yaml::BadValue => Regex::new(DEFAULT_MATCH).unwrap(),
                _ => return Err("expected string for the playbook_match source parameter".to_string())
            },

            match &yaml["pre_provision"] {
                Yaml::String(s) => Some(s.clone()),
                Yaml::BadValue => None,
                _ => return Err("expected string for the pre_provision source parameter".to_string())
            },

            match &yaml["ansible_playbook"].as_hash() {
                Some(_) => match AnsibleContext::parse(&yaml["ansible_playbook"]) {
                    Ok(a) => a,
                    Err(e) => return Err(e)
                },
                None => AnsibleContext::default()
            }
        ))
    }

    pub fn update(&self) -> Result<(), String> {
        match &self.pre_provision {
            Some(c) => util::shell(&c, self.path.as_path(), None),
            None => Ok(())
        }
    }

    pub fn explore(&self) -> Vec<PathBuf> {
        let walker = WalkDir::new(&self.path);
        let walker = match self.recurse {
            true => walker,
            false => walker.max_depth(1)
        };

        walker.into_iter()
            .flatten()
            .filter(|entry| self.playbook_match.is_match(entry.path().to_str().unwrap()))
            .map(|entry| PathBuf::from(entry.path().strip_prefix(&self.path).unwrap()))
            .collect()
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}


#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use array_tool::vec::Intersect;
    use std::io::ErrorKind;

    fn get_source_path(name: &str) -> PathBuf {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR unset");
        PathBuf::from(manifest_dir + "/tests/sources/" + &name)
    }

    fn expect_playbooks(source: Source, expected: Vec<&str>) -> Result<(), String> {
        let playbooks = source.explore();
        let actual: Vec<&str> = playbooks.iter().filter_map(|p| p.to_str()).collect();
        let expected_len = expected.len();

        match actual.intersect(expected).len() == expected_len {
            true => Ok(()),
            false => Err(format!("wrong playbook paths returned: {:?}", actual))
        }
    }

    #[test]
    fn non_existent_dir_empty() -> Result<(), String> {
        let playbooks = Source::new(String::from("nonexistent"),
                                    get_source_path("nonexistent"),
                                    false,
                                    Regex::new(DEFAULT_MATCH).unwrap(),
                                    None,
                                    AnsibleContext::default()).explore();

        match playbooks.len() {
            0 => Ok(()),
            n => Err(format!("expected no file match, got {}", n))
        }
    }

    #[test]
    fn existent_empty_ok() -> Result<(), String> {
        let playbooks = Source::new(String::from("empty"),
                                    get_source_path("empty"),
                                    false,
                                    Regex::new(DEFAULT_MATCH).unwrap(),
                                    None,
                                    AnsibleContext::default()).explore();

        match playbooks.len() {
            0 => Ok(()),
            n => Err(format!("expected no file match, got {}", n))
        }
    }

    #[test]
    fn root_only() -> Result<(), String> {
        let source = Source::new(String::from("root_only"),
                                 get_source_path("root_only"),
                                 false,
                                 Regex::new(DEFAULT_MATCH).unwrap(),
                                 None,
                                 AnsibleContext::default());
        expect_playbooks(source, vec!["playbook1.yml", "playbook2.yaml"])
    }

    #[test]
    fn with_depth_no_recurse() -> Result<(), String> {
        let source = Source::new(String::from("with_depth"),
                                 get_source_path("with_depth"),
                                 false,
                                 Regex::new(DEFAULT_MATCH).unwrap(),
                                 None,
                                 AnsibleContext::default());
        expect_playbooks(source, vec!["playbook1.yml"])
    }

    #[test]
    fn with_depth_recurse() -> Result<(), String> {
        let source = Source::new(String::from("with_depth"),
                                 get_source_path("with_depth"),
                                 true,
                                 Regex::new(DEFAULT_MATCH).unwrap(),
                                 None,
                                 AnsibleContext::default());
        expect_playbooks(source, vec!["playbook1.yml", "depth1/playbook2.yml", "depth2/depth1/playbook3.yml"])
    }

    #[test]
    fn playbook_match_none_no_recurse() -> Result<(), String> {
        let source = Source::new(String::from("root_only"),
                                 get_source_path("root_only"),
                                 false,
                                 Regex::new(r#"nomatch"#).unwrap(),
                                 None,
                                 AnsibleContext::default());
        expect_playbooks(source, vec![])
    }

    #[test]
    fn playbook_match_some_no_recurse() -> Result<(), String> {
        let source = Source::new(String::from("root_only"),
                                 get_source_path("root_only"),
                                 false,
                                 Regex::new(r#"\.yml$"#).unwrap(),
                                 None,
                                 AnsibleContext::default());
        expect_playbooks(source, vec!["playbook1.yml"])
    }

    #[test]
    fn playbook_match_some_recurse() -> Result<(), String> {
        let source = Source::new(String::from("with_depth"),
                                 get_source_path("with_depth"),
                                 true,
                                 Regex::new(r#"playbook{1,3}"#).unwrap(),
                                 None,
                                 AnsibleContext::default());
        expect_playbooks(source, vec!["playbook1.yml", "depth2/depth1/playbook3.yml"])
    }

    #[test]
    fn pre_provision_none() -> Result<(), String> {
        let source = Source::new(String::from("root_only"),
                                 get_source_path("root_only"),
                                 false,
                                 Regex::new(DEFAULT_MATCH).unwrap(),
                                 None,
                                 AnsibleContext::default());

        source.update().map_err(|e| format!("unexpected error when nothing should have happened: {}", e))
    }

    #[test]
    fn pre_provision_wrong_command() -> Result<(), String> {
        let source = Source::new(String::from("root_only"),
                                 get_source_path("root_only"),
                                 false,
                                 Regex::new(DEFAULT_MATCH).unwrap(),
                                 Some(String::from("nonexistent")),
                                 AnsibleContext::default());

        match source.update() {
            Ok(_) => Err("update succeeded with a non-existent command".to_string()),
            Err(_) => Ok(())
        }
    }

    #[test]
    fn pre_provision_failing_command() -> Result<(), String> {
        let source = Source::new(String::from("root_only"),
                                 get_source_path("root_only"),
                                 false,
                                 Regex::new(DEFAULT_MATCH).unwrap(),
                                 Some(String::from("/bin/false")),
                                 AnsibleContext::default());

        match source.update() {
            Ok(_) => Err("update succeeded with a failing command".to_string()),
            Err(_) => Ok(())
        }
    }

    #[test]
    fn pre_provision_ok() -> Result<(), String> {
        let mut temp_path = std::env::temp_dir();
        temp_path.push("setmeup_test_pre_provision_ok");

        if let Err(e) = std::fs::remove_file(temp_path.as_path()) {
            match e.kind() {
                ErrorKind::NotFound => (),
                _ => return Err(format!("failed to remove the temporary file before the test: {}", e))
            }
        };

        let source = Source::new(String::from("root_only"),
                                 get_source_path("root_only"),
                                 false,
                                 Regex::new(DEFAULT_MATCH).unwrap(),
                                 Some(format!("> {}", temp_path.to_str().unwrap())),
                                 AnsibleContext::default());

        match source.update() {
            Ok(_) => std::fs::remove_file(temp_path)
                .map_err(|e| format!("failed to remove the temporary after the test: {}", e)),
            Err(e) => Err(format!("failed to update the source: {}", e))
        }
    }
}
