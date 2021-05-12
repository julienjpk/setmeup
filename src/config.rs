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

use crate::sources::*;

use std::path::{PathBuf, Path};

use clap::ArgMatches;
use directories::{ProjectDirs, BaseDirs, UserDirs};
use yaml_rust::YamlLoader;


/// Returns a (ordered) vector of possible locations for the configuration file
fn get_default_locations() -> Vec<PathBuf> {
    [
        /* Set from the environment? */
        match std::env::var("SETMEUP_CONF") {
            Ok(env_value) => Some(PathBuf::from(env_value)),
            Err(_) => None
        },

        /* Proper per-app directory in XDG_CONFIG_DIR ? */
        match ProjectDirs::from("me", "jjpk", "setmeup") {
            Some(xdg_dirs) => Some([xdg_dirs.config_dir().to_str().unwrap(), "setmeup.yml"].iter().collect()),
            None => None
        },

        /* Acceptable per-app file in XDG_CONFIG_DIR ? */
        match BaseDirs::new() {
            Some(xdg_dirs) => Some([xdg_dirs.config_dir().to_str().unwrap(), "setmeup.yml"].iter().collect()),
            None => None
        },

        /* Old-school file straight into the home directory? */
        match UserDirs::new() {
            Some(user_dirs) => Some([user_dirs.home_dir().to_str().unwrap(), ".setmeup.yml"].iter().collect()),
            None => None
        },

        /* System-wide configuration in an SMU directory? */
        Some(PathBuf::from("/etc/setmeup/setmeup.yml")),

        /* System-wide configuration directly under /etc ? */
        Some(PathBuf::from("/etc/setmeup.yml"))

    ].iter().flatten().map(|path| path.clone()).collect()
}

/// Guesses the most appropriate location for the configuration file
fn infer_configuration_path(args: ArgMatches) -> Result<PathBuf, ()> {
    match args.value_of("config") {
        Some(option_value) => Ok(PathBuf::from(option_value)),
        None => match get_default_locations().iter().filter(|path| path.exists()).next() {
            Some(inferred_location) => Ok(inferred_location.clone()),
            None => Err(())
        }
    }
}


/// Set Me Up! configuration structure
pub struct Config {
    pub sources: Vec<Source>
}

impl Config {
    /// Gets a path to the configuration file and forwards parsing
    pub fn locate_and_parse(args: ArgMatches) -> Result<Self, String> {
        match infer_configuration_path(args) {
            Ok(path) => Self::parse(path.as_path()),
            Err(_) => Err("no configuration file found".to_string())
        }
    }

    /// Handles top-level YAML > struct Config parsing
    pub fn parse(path: &Path) -> Result<Self, String> {
        let yaml_str = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => return Err(format!("failed to read configuration from {}",
                                         path.to_str().unwrap()))
        };

        let yaml = match YamlLoader::load_from_str(&yaml_str) {
            Ok(y) => match y.len() {
                1 => y,
                _ => return Err("configuration should be a single-document YAML file".to_string())
            },
            Err(e) => return Err(e.to_string())
        };

        Ok(Self {
            sources: match yaml[0]["sources"].as_hash() {
                Some(h) => h.iter().map(|(k, v)| Source::parse(String::from(match k.as_str() {
                    Some(s) => s,
                    None => return Err("expected string as source name".to_string())
                }), &v)).collect::<Result<Vec<Source>, String>>()?,
                None => return Err("missing or empty sources".to_string())
            }
        })
    }
}


#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use ctor::*;

    #[ctor]
    fn init() {
        /* Making sure we don't stumble across an actual config file */
        std::env::set_var("XDG_CONFIG_HOME", "/nonexistent");
    }

    use std::path::PathBuf;
    use clap::{App, Arg};

    fn get_test_yaml_file(name: &str) -> PathBuf {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR unset");
        PathBuf::from(manifest_dir + "/tests/" + &name + ".yml")
    }

    fn expected_error_raised(yaml_name: &str, error_substr: &str) -> Result<(), String> {
        match Config::parse(&get_test_yaml_file(yaml_name)) {
            Ok(_) => Err("no error raised".to_string()),
            Err(e) => match e.find(error_substr) {
                Some(_) => Ok(()),
                None => Err(format!("wrong error message: {}", e))
            }
        }
    }

    fn expect_parse_ok(yaml_name: &str) -> Result<Config, String> {
        match Config::parse(&get_test_yaml_file(yaml_name)) {
            Ok(c) => Ok(c),
            Err(e) => Err(format!("unexpected error: {}", e))
        }
    }

    #[test]
    fn test_locate_and_parse_default_ko() -> Result<(), String> {
        let matches = App::new("tests").get_matches_from(vec!["tests"]);
        match Config::locate_and_parse(matches) {
            Ok(_) => Err("parsed configuration with no available configuration file".to_string()),
            Err(e) => match e.find("no configuration file found") {
                Some(_) => Ok(()),
                None => Err(format!("unexpected error message: {}", e))
            }
        }
    }

    #[test]
    fn test_locate_and_parse_default_ok() -> Result<(), String> {
        let matches = App::new("tests")
            .arg(Arg::with_name("config")
                 .short("c")
                 .takes_value(true))
            .get_matches_from(vec!["tests", "-c", get_test_yaml_file("located").to_str().unwrap()]);

        match Config::locate_and_parse(matches) {
            Ok(c) => match c.sources[0].name.as_str() {
                "located" => Ok(()),
                _ => Err(format!("unexpected source name: {}", c.sources[0].name))
            },
            Err(e) => Err(format!("failed to parse: {}", e))
        }
    }

    #[test]
    fn test_not_found_raises_error() -> Result<(), String> {
        expected_error_raised("non_existent", "failed to read")
    }

    #[test]
    fn test_empty_yaml_ko() -> Result<(), String> {
        expected_error_raised("empty", "single-document")
    }

    #[test]
    fn test_invalid_yaml_ko() -> Result<(), String> {
        expected_error_raised("invalid", "") /* not testing yaml-rust, I just want an error */
    }

    #[test]
    fn test_missing_sources_ko() -> Result<(), String> {
        expected_error_raised("no_sources", "missing or empty sources")
    }

    #[test]
    fn test_empty_sources_ko() -> Result<(), String> {
        expected_error_raised("empty_sources", "missing or empty sources")
    }

    #[test]
    fn test_parse_int_source_name_ko() -> Result<(), String> {
        expected_error_raised("int_source_name", "expected string as source name")
    }

    #[test]
    fn test_local_no_path_ko() -> Result<(), String> {
        expected_error_raised("local_no_path", "missing path parameter")
    }

    #[test]
    fn test_local_non_string_path_ko() -> Result<(), String> {
        expected_error_raised("local_non_string_path", "expected string for the path")
    }

    #[test]
    fn test_local_non_dir_path_ko() -> Result<(), String> {
        /* TODO: test unreadable directory */
        expected_error_raised("local_non_dir_path", "failed to read")
    }

    #[test]
    fn test_non_boolean_recurse_ko() -> Result<(), String> {
        expected_error_raised("non_boolean_recurse", "expected boolean for the recurse")
    }

    #[test]
    fn test_non_string_playbook_match_ko() -> Result<(), String> {
        expected_error_raised("non_string_playbook_match", "expected string for the playbook_match")
    }

    #[test]
    fn test_invalid_playbook_match_ko() -> Result<(), String> {
        expected_error_raised("invalid_playbook_match", "") /* not testing regex crate */
    }

    #[test]
    fn test_local_ok() -> Result<(), String> {
        let c = expect_parse_ok("local_ok")?;

        if c.sources.len() != 1 {
            return Err("did not fetch exactly one source".to_string())
        }

        if c.sources[0].name != "foo" {
            return Err("failed to parse source name".to_string())
        }

        if c.sources[0].recurse {
            return Err("recursing although we should not".to_string())
        }

        if ! c.sources[0].playbook_match.is_match("test.yml") {
            return Err("failed to match a .yml file with the default REGEX".to_string())
        }

        if c.sources[0].playbook_match.is_match("test.txt") {
            return Err("matched a .txt file with the default REGEX".to_string())
        }

        if let Some(_) = c.sources[0].pre_provision {
            return Err("unexpected pre_provision command".to_string())
        }

        match c.sources[0].path.to_str().unwrap() == "/tmp" {
            true => Ok(()),
            false => Err("failed to parse /tmp as the path".to_string())
        }
    }

    #[test]
    fn test_recurse_ok() -> Result<(), String> {
        let c = expect_parse_ok("recurse")?;
        match c.sources[0].recurse {
            true => Ok(()),
            false => Err("failed to parse positive recurse parameter".to_string())
        }
    }

    #[test]
    fn test_non_string_pre_provision_ko() -> Result<(), String> {
        expected_error_raised("non_string_pre_provision", "expected string for the pre_provision")
    }

    #[test]
    fn test_pre_provision_ok() -> Result<(), String> {
        let c = expect_parse_ok("pre_provision_ok")?;
        match &c.sources[0].pre_provision {
            Some(s) => if s == "/bin/true" {
                Ok(())
            }
            else {
                Err("wrong pre_provision value".to_string())
            },
            None => Err("failed to parse pre_provision parameter".to_string())
        }
    }

    #[test]
    fn test_empty_ansible_playbook_ok() -> Result<(), String> {
        let c = expect_parse_ok("local_ok")?;

        if let Some(_) = c.sources[0].ansible.path {
            return Err("stored an ansible-playbook as the default".to_string())
        }

        if c.sources[0].ansible.args.len() > 0 {
            return Err("stored ansible-playbook args as defaults".to_string())
        }

        if c.sources[0].ansible.env.len() > 0 {
            return Err("stored environment variables as defaults".to_string())
        }

        Ok(())
    }

    #[test]
    fn test_ansible_playbook_non_string_path_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_non_string_path", "expected string for the ansible-playbook path")
    }

    #[test]
    fn test_ansible_playbook_non_existent_path_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_non_existent_path", "no executable ansible-playbook at")
    }

    #[test]
    fn test_ansible_playbook_non_executable_path_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_non_executable_path", "no executable ansible-playbook at")
    }

    #[test]
    fn test_ansible_playbook_path_ok() -> Result<(), String> {
        let c = expect_parse_ok("ansible_playbook_path")?;
        match &c.sources[0].ansible.path {
            Some(p) => match p.to_str().unwrap() {
                "/bin/true" => Ok(()),
                _ => Err("parsed the wrong ansible-playbook path".to_string())
            },
            None => Err("failed to parse the ansible-playbook path".to_string())
        }
    }

    #[test]
    fn test_ansible_playbook_non_list_args_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_non_list_args", "expected list for the ansible-playbook args")
    }

    #[test]
    fn test_ansible_playbook_non_string_arg_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_non_string_arg", "expected strings as arguments")
    }

    #[test]
    fn test_ansible_playbook_args_ok() -> Result<(), String> {
        let c = expect_parse_ok("ansible_playbook_args")?;
        match c.sources[0].ansible.args.len() {
            2 => match c.sources[0].ansible.args[0] == "arg1" && c.sources[0].ansible.args[1] == "arg2" {
                true => Ok(()),
                false => Err(format!("failed to parse the args, got {} and {}",
                                     c.sources[0].ansible.args[0], c.sources[0].ansible.args[1]))
            },
            _ => Err(format!("parsed {} arg(s) instead of 2", c.sources[0].ansible.args.len()))
        }
    }

    #[test]
    fn test_ansible_playbook_non_list_env_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_non_list_env", "expected list for the ansible-playbook env")
    }

    #[test]
    fn test_ansible_playbook_no_name_env_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_no_name_env", "missing name property")
    }

    #[test]
    fn test_ansible_playbook_non_string_env_name_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_non_string_env_name", "non-string name property")
    }

    #[test]
    fn test_ansible_playbook_no_value_env_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_no_value_env", "missing value property")
    }

    #[test]
    fn test_ansible_playbook_non_string_env_value_ko() -> Result<(), String> {
        expected_error_raised("ansible_playbook_non_string_env_value", "non-string value property")
    }

    #[test]
    fn test_ansible_playbook_env_ok() -> Result<(), String> {
        let c = expect_parse_ok("ansible_playbook_env")?;
        match c.sources[0].ansible.env.len() {
            1 => Ok(()),
            _ => Err(format!("parsed {} environment variables instead of 1", c.sources[0].ansible.env.len()))
        }
    }
}
