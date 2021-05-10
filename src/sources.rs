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
use walkdir::WalkDir;


/// A playbook source
pub struct Source {
    pub name: String,
    pub path: PathBuf,
    pub recurse: bool,
    pub playbook_match: Regex,
    pub pre_provision: Option<String>
}

const DEFAULT_MATCH: &str = r#"\.ya?ml$"#;

impl Source {
    fn new(name: String, path: PathBuf, recurse: bool,
           playbook_match: Regex, pre_provision: Option<String>) -> Self {
        Self { name, path, recurse, playbook_match, pre_provision }
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
            }
        ))
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

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use array_tool::vec::Intersect;

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
                                    None).explore();

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
                                    None).explore();

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
                                 None);
        expect_playbooks(source, vec!["playbook1.yml", "playbook2.yaml"])
    }

    #[test]
    fn with_depth_no_recurse() -> Result<(), String> {
        let source = Source::new(String::from("with_depth"),
                                 get_source_path("with_depth"),
                                 false,
                                 Regex::new(DEFAULT_MATCH).unwrap(),
                                 None);
        expect_playbooks(source, vec!["playbook1.yml"])
    }

    #[test]
    fn with_depth_recurse() -> Result<(), String> {
        let source = Source::new(String::from("with_depth"),
                                 get_source_path("with_depth"),
                                 true,
                                 Regex::new(DEFAULT_MATCH).unwrap(),
                                 None);
        expect_playbooks(source, vec!["playbook1.yml", "depth1/playbook2.yml", "depth2/depth1/playbook3.yml"])
    }

    #[test]
    fn playbook_match_none_no_recurse() -> Result<(), String> {
        let source = Source::new(String::from("root_only"),
                                 get_source_path("root_only"),
                                 false,
                                 Regex::new(r#"nomatch"#).unwrap(),
                                 None);
        expect_playbooks(source, vec![])
    }

    #[test]
    fn playbook_match_some_no_recurse() -> Result<(), String> {
        let source = Source::new(String::from("root_only"),
                                 get_source_path("root_only"),
                                 false,
                                 Regex::new(r#"\.yml$"#).unwrap(),
                                 None);
        expect_playbooks(source, vec!["playbook1.yml"])
    }

    #[test]
    fn playbook_match_some_recurse() -> Result<(), String> {
        let source = Source::new(String::from("with_depth"),
                                 get_source_path("with_depth"),
                                 true,
                                 Regex::new(r#"playbook{1,3}"#).unwrap(),
                                 None);
        expect_playbooks(source, vec!["playbook1.yml", "depth2/depth1/playbook3.yml"])
    }
}
