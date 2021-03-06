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


use crate::ansible::AnsibleResult;
use crate::sources::Source;
use crate::config::Config;
use crate::setup::Setup;
use crate::ui::UI;

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
        let source_index = UI.prompt_from_vec(
            "Here are the available provisioning sources:",
            &config.sources.iter().map(|s| s.name.clone()).collect()
        );

        let source = config.sources.get(source_index).unwrap();
        source.update()?;

        let playbooks = source.explore();
        let playbook_index = UI.prompt_from_vec(
            "Here are the available playbooks:",
            &playbooks.iter().map(|p| String::from(p.as_path().to_str().unwrap())).collect()
        );
        let playbook_path = playbooks[playbook_index].clone();

        Ok(Self {
            setup,
            source,
            playbook_path
        })
    }

    /// Runs ansible-playbook and provisions the client
    pub fn execute(&self) -> Result<AnsibleResult, String> {
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

        /* Create the inventory file */
        let mut inventory = NamedTempFile::new().map_err(|e| format!("failed to ready the inventory file: {}", e))?;
        inventory.write(
            format!("provisionee ansible_host=127.0.0.1 ansible_port={} ansible_user={}",
                    self.setup.reverse_port,
                    self.setup.credentials.username).as_bytes()
        ).map_err(|e| format!("failed to write the inventory: {}", e))?;

        /* Call ansible-playbook */
        self.source.ansible.execute(
            keyfile.path(),
            inventory.path(),
            self.playbook_path.as_path(),
            self.source.path.as_path()
        )
    }
}
