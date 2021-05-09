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

use crate::ssh;
use crate::util;

use std::net::TcpListener;

use osshkeys::{KeyPair, KeyType};

pub struct SSHCredentials {
    pub username: String,
    pub keypair: KeyPair
}

pub struct Setup {
    pub reverse_port: u16,
    pub credentials: SSHCredentials
}

impl Setup {
    pub fn prompt() -> Result<Self, String> {
        let reverse_port = Self::prompt_port()?;
        let credentials = Self::key_setup(reverse_port)?;
        Ok(Self { reverse_port, credentials })
    }

    fn port_is_bound(port: u16) -> bool {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(_) => false,
            Err(e) => match e.kind() {
                std::io::ErrorKind::AddrInUse => true,
                _ => false
            }
        }
    }

    fn prompt_port() -> Result<u16, String> {
        loop {
            let mut input = String::new();
            util::prompt("Which port did ssh bind to for remote forwarding?", &mut input)?;
            let input_port = input.parse::<u16>();

            match input_port {
                Ok(p) => match Self::port_is_bound(p) {
                    true => return Ok(p),
                    false => util::error(&format!("Port is not bound locally: {}", p))
                }
                Err(e) => util::error(&format!("Invalid port specification: {}: {}", input, e))
            }
        }
    }

    fn key_setup(port: u16) -> Result<SSHCredentials, String> {
        let keypair = KeyPair::generate(KeyType::ECDSA, 0).map_err(|e| format!("failed to generate keypair: {}", e))?;
        let keypair_str = keypair.serialize_publickey().map_err(|e| format!("failed to serialise keypair: {}", e))?.to_string();

        let mut username = String::new();
        let mut dummy = String::new();

        loop {
            while username.is_empty() {
                util::prompt("Which username should SetMeUp use to reach you over SSH?", &mut username)?;
                if username.is_empty() {
                    util::error(&format!("The username cannot be empty"));
                }
            }

            println!("\nSetMeUp will be using an ECDSA keypair to authenticate with your machine.");
            println!("Please make sure user {} has the following public key in their ~/.ssh/authorized_keys file:", username);
            util::important(&keypair_str);
            util::prompt("Press the Enter key where you are done:", &mut dummy)?;

            match ssh::test_credentials(port, &username, &keypair) {
                Ok(_) => return Ok(SSHCredentials { username, keypair }),
                Err(e) => {
                    util::error(&format!("Authentication test failed: {}", e));
                    username.clear();
                }
            }
        }
    }
}