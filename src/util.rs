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

use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[cfg(not(tarpaulin_include))]
pub fn prompt(invite: &str, buffer: &mut String) -> Result<(), String> {
    print!("{} ", invite);
    std::io::stdout().flush().map_err(|e| format!("failed to converse: {}", e))?;
    std::io::stdin().read_line(buffer).map_err(|e| format!("failed to read input: {}", e))?;
    buffer.truncate(buffer.trim_end().len());
    Ok(())
}

#[cfg(not(tarpaulin_include))]
fn highlight(msg: &str, color: Color) {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true)).ok();
    println!("\n    {}\n", msg);
    stdout.reset().ok();
}

#[cfg(not(tarpaulin_include))]
pub fn error(msg: &str) {
    highlight(msg, Color::Red);
}

#[cfg(not(tarpaulin_include))]
pub fn important(msg: &str) {
    highlight(msg, Color::Cyan);
}
