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


//! UI logic


use crate::ansible::AnsibleResult;

use std::io::Write;

use atty;
use termion::{clear, color, style, cursor};
use lazy_static::lazy_static;


pub trait UserInterface: Sync {
    fn intro(&self);
    fn error(&self, message: &str);
    fn next_step(&self);
    fn present_pubkey(&self, username: &str, pubkey: &str);
    fn prompt_from_vec(&self, message: &str, choices: &Vec<String>) -> usize;
    fn render_ansible_result(&self, result: &AnsibleResult);

    fn intro_pubkey(&self, username: &str) {
        self.next_step();
        println!("SetMeUp will be using an ECDSA keypair to authenticate with your machine.");
        println!("Please make sure user {} has the following public key in their ~/.ssh/authorized_keys file:\n", username);
    }

    fn running(&self) {
        print!("Running Ansible (this may take a while)... ");
        std::io::stdout().flush().ok();
    }

    fn prompt(&self, message: &str) -> String {
        print!("{} ", message);

        if let Err(e) = std::io::stdout().flush() {
            self.exit_with_error(&format!("failed to flush stdout: {}", e))
        }

        let mut buffer = String::new();
        if let Err(e) = std::io::stdin().read_line(&mut buffer) {
            self.exit_with_error(&format!("failed to read from stdin: {}", e))
        }

        buffer.truncate(buffer.trim_end().len());
        buffer
    }

    fn prompt_index_in_range(&self, length: usize) -> usize {
        let mut index_1 = 0;
        while index_1 <= 0 || index_1 > length {
            let index_input = self.prompt(&format!("Select by index (1-{}) :", length));
            index_1 = index_input.parse::<usize>().unwrap_or(0);
        }
        index_1 - 1
    }

    fn exit_with_error(&self, message: &str) -> ! {
        self.error(message);
        std::process::exit(1);
    }
}


pub struct BasicInterface;

impl UserInterface for BasicInterface {
    fn intro(&self) {
        println!("=== Welcome to SetMeUp! ===");
        println!("Basic UI mode: connect with `ssh -t` for something slightly fancier\n");
    }

    fn error(&self, message: &str) {
        println!("/!\\ {}", message);
    }

    fn next_step(&self) {
        println!();
    }

    fn present_pubkey(&self, username: &str, pubkey: &str) {
        self.intro_pubkey(username);
        println!("---\n{}\n---\n", pubkey);
    }

    fn prompt_from_vec(&self, message: &str, choices: &Vec<String>) -> usize {
        println!("{}\n", message);
        choices.iter().enumerate().for_each(|(i, c)| println!("    {}. {}", i + 1, c));
        println!();

        self.prompt_index_in_range(choices.len())
    }

    fn render_ansible_result(&self, result: &AnsibleResult) {
        println!("done!");
        for task in result {
            println!(
                "`- [{}]{} {}",
                if task.success { "OK" } else { "KO" },
                if task.changed { "" } else { " (change)" },
                task.name);
            if !task.success {
                println!("        Task error message: {}", task.message);
            }
        }
    }
}


pub struct TTYInterface;

impl TTYInterface {
    fn clear(&self) {
        print!("{}{}", clear::All, cursor::Goto(1, 1));
    }
}

impl UserInterface for TTYInterface {
    fn intro(&self) {
        println!("{}{}Welcome to SetMeUp!{}\n",
                 style::Bold,
                 color::Fg(color::Cyan),
                 style::Reset);
    }

    fn error(&self, message: &str) {
        println!("{}{}{}{}",
                 style::Bold,
                 color::Fg(color::Red),
                 message,
                 style::Reset);
    }

    fn next_step(&self) {
        self.clear();
    }

    fn present_pubkey(&self, username: &str, pubkey: &str) {
        self.intro_pubkey(username);
        println!("{}{}{}{}\n",
                 style::Bold,
                 color::Fg(color::Blue),
                 pubkey,
                 style::Reset);
    }

    fn prompt_from_vec(&self, message: &str, choices: &Vec<String>) -> usize {
        println!("{}\n", message);
        for (i, c) in choices.iter().enumerate() {
            println!("    {}{}{}.{} {}",
                     style::Bold,
                     color::Fg(color::Cyan),
                     i + 1,
                     style::Reset,
                     c);
        }
        println!();

        self.prompt_index_in_range(choices.len())
    }

    fn render_ansible_result(&self, result: &AnsibleResult) {
        println!("{}{}done!{}", color::Fg(color::Cyan), style::Bold, style::Reset);

        let ok = format!("{}{}✓{}", color::Fg(color::Green), style::Bold, style::Reset);
        let ko = format!("{}{}x{}", color::Fg(color::Red), style::Bold, style::Reset);
        let change = format!(" ({}{}change{})", color::Fg(color::Yellow), style::Bold, style::Reset);

        for task in result {
            println!(
                "`- [{}]{} {}",
                if task.success { &ok } else { &ko },
                if task.changed { &change } else { "" },
                task.name);
            if !task.success {
                println!("       {}{}Task error message:{} {}",
                         color::Fg(color::Red), style::Bold, style::Reset,
                         task.message);
            }
        }
    }
}


pub type BoxedInterface = Box<dyn UserInterface>;
lazy_static! {
    pub static ref UI: BoxedInterface = match atty::is(atty::Stream::Stdin) {
        true => Box::new(BasicInterface {}),
        false => Box::new(TTYInterface {})
    };
}
