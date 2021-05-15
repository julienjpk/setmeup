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


//! Interacts with the client and actually provisions it


use crate::sources::Source;
use crate::config::Config;
use crate::setup::Setup;
use crate::util;

use osshkeys::cipher::Cipher;
use tempfile::NamedTempFile;

use std::path::PathBuf;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;


/// Handles client interaction and triggers provisioning accordingly
pub struct Provision<'a> {
    setup: &'a Setup,
    source: &'a Source,
    playbook_path: PathBuf
}

#[cfg(not(tarpaulin_include))]
impl<'a> Provision<'a> {
    /// Prompts the client for a source and playbook
    pub fn prompt(config: &'a Config, setup: &'a Setup) -> Result<Self, String> {
        println!("Here are the available provisioning sources:\n");
        let source_index = util::iter_prompt_index(config.sources.iter())?;

        println!("\nPreparing the source...");
        let source = config.sources.get(source_index).unwrap();
        source.update()?;

        let playbooks = source.explore();
        println!("Here are the available playbooks for source {}:\n", source.name);
        let playbook_index = util::iter_prompt_index(playbooks.iter().map(|p| p.as_path().to_str().unwrap()))?;
        let playbook_path = playbooks[playbook_index].clone();

        Ok(Self {
            setup,
            source,
            playbook_path
        })
    }

    /// Runs ansible-playbook and provisions the client
    pub fn execute(&self) -> Result<(), String> {
        /* Put the key on disk */
        let mut keyfile = NamedTempFile::new().map_err(|e| format!("failed to ready the private key: {}", e))?;

        keyfile.path().metadata()
            .map_err(|e| format!("failed to secure the private key file: {}", e))?
            .permissions().set_mode(0o600);

        keyfile.write(
            self.setup.credentials.keypair
                .serialize_openssh(None, Cipher::Null)
                .map_err(|e| format!("failed to serialise the private key: {}", e))?.as_bytes())
            .map_err(|e| format!("failed to write the private key to disk: {}", e))?;

        println!("\nRunning ansible-playbook...");

        /* Call ansible-playbook */
        util::exec(
            match &self.source.ansible.path {
                Some(p) => p.as_path().to_str().unwrap(),
                None => "ansible-playbook"
            },
            vec!(
                "--private-key",
                keyfile.path().to_str().unwrap(),
                "-Ki",
                &format!("127.0.0.1:{},", self.setup.reverse_port),
                "-l", "127.0.0.1",
                "-u",
                &self.setup.credentials.username,
                self.playbook_path.as_path().to_str().unwrap()
            ),
            self.source.path.as_path(),
            Some(&self.source.ansible.env),
            true
        )
    }
}
