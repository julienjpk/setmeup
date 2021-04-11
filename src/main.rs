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

mod config;

use clap::{Arg, App};

#[cfg(not(tarpaulin_include))]
fn main() {
    let options = App::new("Set Me Up!")
        .version("0.0.0")
        .about("Minimalistic Ansible-based remote provisioning tool")
        .arg(Arg::with_name("config").short("c").value_name("FILE").takes_value(true))
        .get_matches();

    let smu_config = match config::Config::locate_and_parse(options) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to parse configuration: {}", e);
            std::process::exit(1);
        }
    };
}
