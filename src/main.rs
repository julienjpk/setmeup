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
mod ansible;
mod sources;
mod config;
mod setup;
mod exec;
mod ui;

use config::Config;
use setup::Setup;
use provision::Provision;
use ui::UI;

use clap::{Arg, App};


/// Set Me Up! entry point
#[cfg(not(tarpaulin_include))]
fn main() {
    let options = App::new("Set Me Up!")
        .version("0.3.0")
        .about("Minimalistic Ansible-based remote provisioning tool")
        .arg(Arg::new("config").short('c').value_name("FILE").takes_value(true))
        .get_matches();

    /* Locate, parse and validate the configuration file */
    let run_config = match Config::locate_and_parse(options) {
        Ok(c) => c,
        Err(e) => UI.exit_with_error(&format!("Failed to parse configuration: {}", e))
    };

    UI.intro();

    /* Prompt the user about the port, username and key */
    let client_config = match Setup::prompt() {
        Ok(s) => s,
        Err(e) => UI.exit_with_error(&format!("Failed to set up the exchange: {}", e))
    };

    UI.next_step();

    /* Prepare and execute provisioning */
    let provisioner = match Provision::prompt(&run_config, &client_config) {
        Ok(p) => p,
        Err(e) => UI.exit_with_error(&format!("Failed to prepare for provisioning: {}", e))
    };

    UI.next_step();
    UI.running();

    match provisioner.execute() {
        Ok(r) => UI.render_ansible_result(&r),
        Err(e) => UI.exit_with_error(&format!("Provisioning error: {}", e))
    }
}
