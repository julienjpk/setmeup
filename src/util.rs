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
use std::ops::Deref;
use std::fmt::Display;
use std::collections::HashMap;
use std::path::Path;
use std::process::{Command, Stdio};

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
fn highlight(msg: &str, color: Option<Color>) {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_fg(color).set_bold(true)).ok();
    println!("{}", msg);
    stdout.reset().ok();
}

#[cfg(not(tarpaulin_include))]
pub fn error(msg: &str) {
    print!("\n/!\\ ");
    highlight(msg, Some(Color::Red));
    print!("\n");
}

#[cfg(not(tarpaulin_include))]
pub fn important(msg: &str) {
    print!("\n");
    highlight(msg, Some(Color::Cyan));
    print!("\n");
}

#[cfg(not(tarpaulin_include))]
pub fn success(msg: &str) {
    highlight(msg, Some(Color::Green));
}

#[cfg(not(tarpaulin_include))]
pub fn iter_prompt_index<I: Iterator<Item=impl Display>>(iter: I) -> Result<usize, String> {
    let length = iter.enumerate()
        .inspect(|(i, item)| highlight(&format!("[{}] {}", i + 1, item), None))
        .count();

    print!("\n");

    let mut index_1 = 0;
    while index_1 <= 0 || index_1 > length {
        let mut index_input = String::new();
        prompt(&format!("Select by index (1-{}) :", length), &mut index_input)?;
        index_1 = index_input.parse::<usize>().unwrap_or(0);
    }

    Ok(index_1 - 1)
}


pub fn exec(program: &str, args: Vec<&str>, working_dir: &Path,
            env: Option<&HashMap<String, String>>, tty: bool) -> Result<(), String> {
    let mut command = Command::new(program);

    command.args(args).current_dir(working_dir);

    if let Some(e) = env {
        command.envs(e);
    }

    if tty {
        match command.status() {
            Ok(s) => match s.success() {
                true => Ok(()),
                false => Err(format!("{} exited with non-zero status code {}", program, s))
            },
            Err(e) => Err(format!("failed to spawn process: {}", e))
        }
    }
    else {
        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

        match command.output() {
            Ok(o) => match o.status.success() {
                true => Ok(()),
                false => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    let report = format!(
                        "{}\n\n{}",
                        match stdout.len() {
                            0 => "<nothing on stdout>",
                            _ => stdout.deref()
                        },
                        match stderr.len() {
                            0 => "<nothing on stderr>",
                            _ => stderr.deref()
                        }
                    );

                    Err(format!("failed to run {}:\n\n{}", program, report))
                }
            },
            Err(e) => Err(format!("failed to spawn process: {}", e))
        }
    }
}

pub fn shell(cmdline: &String, working_dir: &Path,
             env: Option<&HashMap<String, String>>) -> Result<(), String> {
    let executable = std::env::var("SHELL").unwrap_or(String::from("/bin/sh"));
    let args = vec!["-c", &cmdline];
    exec(&executable, args, working_dir, env, false)
}
