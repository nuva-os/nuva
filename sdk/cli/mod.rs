/*
 * Nuva OS
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// ! CLI enterportmodule

pub mod commands;
pub mod args;

use crate::NuvaSdk;
use crate::error::SdkError;

/// run CLI
pub fn run(sdk: &mut NuvaSdk) -> Result<(), SdkError> {
 let args = args::parse_args();
 execute_command(sdk, args)
}

/// execute
fn execute_command(sdk: &mut NuvaSdk, args: args::CliArgs) -> Result<(), SdkError> {
 match args.command {
 args::Command::Pkg(cmd) => commands::pkg::execute(sdk, cmd),
 args::Command::Build(cmd) => commands::build::execute(sdk, cmd),
 args::Command::Test(cmd) => commands::test::execute(sdk, cmd),
 args::Command::Run(cmd) => commands::run::execute(sdk, cmd),
 args::Command::Debug(cmd) => commands::debug::execute(sdk, cmd),
 args::Command::Profile(cmd) => commands::profile::execute(sdk, cmd),
 args::Command::Fmt(cmd) => commands::fmt::execute(sdk, cmd),
 args::Command::Lint(cmd) => commands::lint::execute(sdk, cmd),
 args::Command::Doc(cmd) => commands::doc::execute(sdk, cmd),
 args::Command::Clean(cmd) => commands::clean::execute(sdk, cmd),
 args::Command::New(cmd) => commands::new::execute(sdk, cmd),
 args::Command::Init(cmd) => commands::init::execute(sdk, cmd),
 args::Command::Version => {
 println!("Nuva SDK v{}", crate::version());
 Ok(())
 }
 args::Command::Help => {
 print_help();
 Ok(())
 }
 }
}

/// printinformation
fn print_help() {
 println!(r#"
Nuva SDK - Development toolkit for Nuva OS

USAGE:
 nuva <command> [options]

COMMANDS:
 pkg Package management
 build Build the project
 test Run tests
 run Run the project
 debug Debug the project
 profile Profile the project
 fmt Format source code
 lint Lint source code
 doc Generate documentation
 clean Clean build artifacts
 new Create a new project
 init Initialize a new project
 version Show version information
 help Show this help message

OPTIONS:
 -h, --help Show help for a command
 -v, --verbose Enable verbose output
 -q, --quiet Suppress output

For more information about a command, run:
 nuva <command> --help
"#);
}

/// CLI output
pub mod output {
 use std::io::Write;

 /// outputlevelcategory
 #[derive(Debug, Clone, Copy)]
 pub enum Level {
 Info,
 Success,
 Warning,
 Error,
 Debug,
 }

 /// printmessage
 pub fn print(level: Level, message: &str) {
 let prefix = match level {
 Level::Info => "\x1b[34m[i]\x1b[0m", // blue
 Level::Success => "\x1b[32m[+]\x1b[0m", // green
 Level::Warning => "\x1b[33m[!]\x1b[0m", // yellow
 Level::Error => "\x1b[31m[-]\x1b[0m", // red
 Level::Debug => "\x1b[35m[d]\x1b[0m", // purple
 };
 
 println!("{} {}", prefix, message);
 }

 /// printinformation
 pub fn info(message: &str) {
 print(Level::Info, message);
 }

 /// printsuccess
 pub fn success(message: &str) {
 print(Level::Success, message);
 }

 /// printwarning
 pub fn warning(message: &str) {
 print(Level::Warning, message);
 }

 /// printerror
 pub fn error(message: &str) {
 print(Level::Error, message);
 }

 /// printdebug
 pub fn debug(message: &str) {
 print(Level::Debug, message);
 }

 /// printentermeasurement
 pub fn progress(current: usize, total: usize, message: &str) {
 let percent = if total > 0 { (current * 100) / total } else { 0 };
 print!("\r\x1b[34m[{}%]\x1b[0m {}", percent, message);
 std::io::stdout().flush().ok();
 }

 /// clearcurrentrow
 pub fn clear_line() {
 print!("\r\x1b[2K");
 std::io::stdout().flush().ok();
 }
}