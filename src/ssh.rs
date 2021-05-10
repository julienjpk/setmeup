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

use ssh2::Session;
use osshkeys::KeyPair;

#[cfg(not(tarpaulin_include))]
pub fn test_credentials(local_port: u16, username: &String, keypair: &KeyPair) -> Result<(), String> {
    let tcp = std::net::TcpStream::connect(format!("127.0.0.1:{}", local_port))
        .map_err(|e| format!("failed to connect via local port {}: {}", local_port, e))?;
    let mut session = Session::new().map_err(|e| format!("failed to open session: {}", e))?;
    session.set_tcp_stream(tcp);
    session.handshake().map_err(|e| format!("handshake failed: {}", e))?;

    let pem_privkey = keypair.serialize_pem(None)
        .map_err(|e| format!("failed to encode private key: {}", e))?;

    let result = session.userauth_pubkey_memory(
        username,
        None,
        &pem_privkey,
        None
    ).map(|_| ()).map_err(|e| format!("{}", e));

    session.disconnect(None, "setmeup authentication test complete", None).ok();
    result
}
