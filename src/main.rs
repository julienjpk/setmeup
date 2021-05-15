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


//! Set Me Up! (SMU) is a simple implementation of an Ansible-based provisioning server.


mod provision;
mod sources;
mod config;
mod setup;
mod util;

use clap::{Arg, App};
use atty::Stream;


/// Set Me Up! entry point
#[cfg(not(tarpaulin_include))]
fn main() {
    let options = App::new("Set Me Up!")
        .version("0.0.0")
        .about("Minimalistic Ansible-based remote provisioning tool")
        .arg(Arg::with_name("config").short("c").value_name("FILE").takes_value(true))
        .get_matches();

    /* Locate, parse and validate the configuration file */
    let run_config = match config::Config::locate_and_parse(options) {
        Ok(c) => c,
        Err(e) => {
            util::error(&format!("Failed to parse configuration: {}", e));
            std::process::exit(1);
        }
    };

    util::success("Welcome to Set Me Up!");
    if ! atty::is(Stream::Stdin) {
        util::error("Set Me Up! is running without a TTY!\n\
                     This will make it impossible for ansible-playbook to hide your become password as you type it.\n\
                     Make sure you use -t when connecting to the SMU server");
    }

    /* Prompt the user about the port, username and key */
    let client_config = match setup::Setup::prompt() {
        Ok(s) => s,
        Err(e) => {
            util::error(&format!("Failed to set up the exchange: {}", e));
            std::process::exit(1);
        }
    };

    /* Prepare and execute provisioning */
    let provisioner = match provision::Provision::prompt(&run_config, &client_config) {
        Ok(p) => p,
        Err(e) => {
            util::error(&format!("Failed to prepare for provisioning: {}", e));
            std::process::exit(1);
        }
    };

    match provisioner.execute() {
        Ok(_) => util::success("Provisioning complete!"),
        Err(e) => {
            util::error(&format!("Provisioning error: {}", e));
            std::process::exit(1);
        }
    }
}
