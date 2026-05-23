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

// ! command lineparameterparse

/// CLI parameter
#[derive(Debug)]
pub struct CliArgs {
 /***/
 pub command: Command,
 /// Globaloption
 pub global: GlobalOptions,
}

/// Globaloption
#[derive(Debug, Default)]
pub struct GlobalOptions {
 /// fineoutput
 pub verbose: bool,
 /// staticpattern
 pub quiet: bool,
 /// coloroutput
 pub color: bool,
 /// targetArchitecture
 pub target: Option<String>,
 /// workmakedirectory
 pub workdir: Option<String>,
}

/// enum
#[derive(Debug)]
pub enum Command {
 /// packetmanagementadministration
 Pkg(PkgCommand),
 /// build
 Build(BuildCommand),
 /// test
 Test(TestCommand),
 /// run
 Run(RunCommand),
 /// debug
 Debug(DebugCommand),
 /// performanceanalyze
 Profile(ProfileCommand),
 /// format
 Fmt(FmtCommand),
 /// Codecheck
 Lint(LintCommand),
 /// documentationgenerate
 Doc(DocCommand),
 /// clearadministration
 Clean(CleanCommand),
 /// newbuildproject
 New(NewCommand),
 /// initializeproject
 Init(InitCommand),
 /// versioninformation
 Version,
 /// information
 Help,
}

/// packetmanagementadministration
#[derive(Debug)]
pub enum PkgCommand {
 /// installdependency
 Install {
 packages: Vec<String>,
 dev: bool,
 },
 /// uninstalldependency
 Uninstall {
 packages: Vec<String>,
 },
 /// updatedependency
 Update {
 packages: Vec<String>,
 },
 /// searchpacket
 Search {
 query: String,
 },
 /// releasepacket
 Publish {
 dry_run: bool,
 },
 /// columnexitdependency
 List {
 depth: Option<usize>,
 },
 /// lockfixeddependency
 Lock,
}

/// build
#[derive(Debug)]
pub struct BuildCommand {
 /// releasepattern
 pub release: bool,
 /// target
 pub target: Option<String>,
 /// ity
 pub features: Vec<String>,
 /// Paralleltasknumber
 pub jobs: Option<usize>,
}

/// test
#[derive(Debug)]
pub struct TestCommand {
 /// testfilterdevice
 pub filter: Option<String>,
 /// releasepattern
 pub release: bool,
 /// showoutput
 pub nocapture: bool,
 /// Paralleltasknumber
 pub jobs: Option<usize>,
}

/// run
#[derive(Debug)]
pub struct RunCommand {
 /// releasepattern
 pub release: bool,
 /// parameter
 pub args: Vec<String>,
}

/// debug
#[derive(Debug)]
pub struct DebugCommand {
 /// programPath
 pub program: Option<String>,
 /// parameter
 pub args: Vec<String>,
 /// appendPlustoprocess
 pub attach: Option<u32>,
}

/// performanceanalyze
#[derive(Debug)]
pub enum ProfileCommand {
 /// CPU analyze
 Cpu {
 duration: Option<u64>,
 output: Option<String>,
 },
 /// memoryanalyze
 Memory {
 duration: Option<u64>,
 },
 /// generatediagram
 Flamegraph {
 input: String,
 output: String,
 },
}

/// format
#[derive(Debug)]
pub struct FmtCommand {
 /// checkpattern
 pub check: bool,
 /// filelist
 pub files: Vec<String>,
}

/// Codecheck
#[derive(Debug)]
pub struct LintCommand {
 /// fix
 pub fix: bool,
 /// filelist
 pub files: Vec<String>,
}

/// documentation
#[derive(Debug)]
pub struct DocCommand {
 /// opendocumentation
 pub open: bool,
 /// outputdirectory
 pub output: Option<String>,
}

/// clearadministration
#[derive(Debug)]
pub struct CleanCommand {
 /// clearadministrationplacefinite
 pub all: bool,
 /// clearadministrationtarget
 pub target: Option<String>,
}

/// newbuildproject
#[derive(Debug)]
pub struct NewCommand {
 /// projectname
 pub name: String,
 /// projectPath
 pub path: Option<String>,
 /// template
 pub template: Option<String>,
}

/// initializeproject
#[derive(Debug)]
pub struct InitCommand {
 /// projectname
 pub name: Option<String>,
 /// template
 pub template: Option<String>,
}

/// parsecommand lineparameter
pub fn parse_args() -> CliArgs {
 let args: Vec<String> = std::env::args().collect();
 parse_args_from(&args[1..])
}

/// secondaryStringArrayparseparameter
fn parse_args_from(args: &[String]) -> CliArgs {
 let mut global = GlobalOptions::default();
 let mut command = Command::Help;
 
 let mut i = 0;
 while i < args.len() {
 match args[i].as_str() {
 "-v" | "--verbose" => global.verbose = true,
 "-q" | "--quiet" => global.quiet = true,
 "--no-color" => global.color = false,
 "--target" => {
 if i + 1 < args.len() {
 global.target = Some(args[i + 1].clone());
 i += 1;
 }
 }
 "-C" | "--workdir" => {
 if i + 1 < args.len() {
 global.workdir = Some(args[i + 1].clone());
 i += 1;
 }
 }
 "pkg" => {
 command = parse_pkg_command(&args[i + 1..]);
 break;
 }
 "build" | "b" => {
 command = parse_build_command(&args[i + 1..]);
 break;
 }
 "test" | "t" => {
 command = parse_test_command(&args[i + 1..]);
 break;
 }
 "run" | "r" => {
 command = parse_run_command(&args[i + 1..]);
 break;
 }
 "debug" => {
 command = parse_debug_command(&args[i + 1..]);
 break;
 }
 "profile" => {
 command = parse_profile_command(&args[i + 1..]);
 break;
 }
 "fmt" => {
 command = parse_fmt_command(&args[i + 1..]);
 break;
 }
 "lint" => {
 command = parse_lint_command(&args[i + 1..]);
 break;
 }
 "doc" => {
 command = parse_doc_command(&args[i + 1..]);
 break;
 }
 "clean" => {
 command = parse_clean_command(&args[i + 1..]);
 break;
 }
 "new" => {
 command = parse_new_command(&args[i + 1..]);
 break;
 }
 "init" => {
 command = parse_init_command(&args[i + 1..]);
 break;
 }
 "version" | "-V" | "--version" => {
 command = Command::Version;
 break;
 }
 "help" | "-h" | "--help" => {
 command = Command::Help;
 break;
 }
 _ => {}
 }
 i += 1;
 }
 
 CliArgs { command, global }
}

fn parse_pkg_command(args: &[String]) -> Command {
 if args.is_empty() {
 return Command::Pkg(PkgCommand::List { depth: None });
 }
 
 match args[0].as_str() {
 "install" | "add" | "i" => {
 let packages: Vec<String> = args[1..].iter()
 .filter(|a| !a.starts_with('-'))
 .cloned()
 .collect();
 let dev = args.iter().any(|a| a == "--dev" || a == "-D");
 Command::Pkg(PkgCommand::Install { packages, dev })
 }
 "uninstall" | "remove" | "rm" => {
 let packages: Vec<String> = args[1..].iter()
 .filter(|a| !a.starts_with('-'))
 .cloned()
 .collect();
 Command::Pkg(PkgCommand::Uninstall { packages })
 }
 "update" | "up" => {
 let packages: Vec<String> = args[1..].iter()
 .filter(|a| !a.starts_with('-'))
 .cloned()
 .collect();
 Command::Pkg(PkgCommand::Update { packages })
 }
 "search" => {
 let query = args.get(1).cloned().unwrap_or_default();
 Command::Pkg(PkgCommand::Search { query })
 }
 "publish" => {
 let dry_run = args.iter().any(|a| a == "--dry-run");
 Command::Pkg(PkgCommand::Publish { dry_run })
 }
 "list" | "ls" => {
 let depth = args.iter()
 .position(|a| a == "--depth")
 .and_then(|i| args.get(i + 1))
 .and_then(|s| s.parse().ok());
 Command::Pkg(PkgCommand::List { depth })
 }
 "lock" => Command::Pkg(PkgCommand::Lock),
 _ => Command::Pkg(PkgCommand::List { depth: None }),
 }
}

fn parse_build_command(args: &[String]) -> Command {
 let release = args.iter().any(|a| a == "--release" || a == "-r");
 let target = args.iter()
 .position(|a| a == "--target")
 .and_then(|i| args.get(i + 1).cloned());
 let features: Vec<String> = args.iter()
 .skip_while(|a| *a != "--features")
 .skip(1)
 .take_while(|a| !a.starts_with('-'))
 .cloned()
 .collect();
 let jobs = args.iter()
 .position(|a| a == "-j" || a == "--jobs")
 .and_then(|i| args.get(i + 1))
 .and_then(|s| s.parse().ok());
 
 Command::Build(BuildCommand { release, target, features, jobs })
}

fn parse_test_command(args: &[String]) -> Command {
 let filter = args.iter()
 .find(|a| !a.starts_with('-'))
 .cloned();
 let release = args.iter().any(|a| a == "--release");
 let nocapture = args.iter().any(|a| a == "--nocapture" || a == "--show-output");
 let jobs = args.iter()
 .position(|a| a == "-j" || a == "--jobs")
 .and_then(|i| args.get(i + 1))
 .and_then(|s| s.parse().ok());
 
 Command::Test(TestCommand { filter, release, nocapture, jobs })
}

fn parse_run_command(args: &[String]) -> Command {
 let release = args.iter().any(|a| a == "--release");
 let run_args: Vec<String> = args.iter()
 .skip_while(|a| *a != "--")
 .skip(1)
 .cloned()
 .collect();
 
 Command::Run(RunCommand { release, args: run_args })
}

fn parse_debug_command(args: &[String]) -> Command {
 let program = args.iter()
 .find(|a| !a.starts_with('-'))
 .cloned();
 let attach = args.iter()
 .position(|a| a == "--attach" || a == "-p")
 .and_then(|i| args.get(i + 1))
 .and_then(|s| s.parse().ok());
 let debug_args: Vec<String> = args.iter()
 .skip_while(|a| *a != "--")
 .skip(1)
 .cloned()
 .collect();
 
 Command::Debug(DebugCommand { program, args: debug_args, attach })
}

fn parse_profile_command(args: &[String]) -> Command {
 if args.is_empty() {
 return Command::Profile(ProfileCommand::Cpu { duration: None, output: None });
 }
 
 match args[0].as_str() {
 "cpu" => {
 let duration = args.iter()
 .position(|a| a == "--duration" || a == "-d")
 .and_then(|i| args.get(i + 1))
 .and_then(|s| s.parse().ok());
 let output = args.iter()
 .position(|a| a == "--output" || a == "-o")
 .and_then(|i| args.get(i + 1).cloned());
 Command::Profile(ProfileCommand::Cpu { duration, output })
 }
 "mem" | "memory" => {
 let duration = args.iter()
 .position(|a| a == "--duration" || a == "-d")
 .and_then(|i| args.get(i + 1))
 .and_then(|s| s.parse().ok());
 Command::Profile(ProfileCommand::Memory { duration })
 }
 "flame" | "flamegraph" => {
 let input = args.get(1).cloned().unwrap_or_default();
 let output = args.get(2).cloned().unwrap_or_default();
 Command::Profile(ProfileCommand::Flamegraph { input, output })
 }
 _ => Command::Profile(ProfileCommand::Cpu { duration: None, output: None }),
 }
}

fn parse_fmt_command(args: &[String]) -> Command {
 let check = args.iter().any(|a| a == "--check");
 let files: Vec<String> = args.iter()
 .filter(|a| !a.starts_with('-'))
 .cloned()
 .collect();
 
 Command::Fmt(FmtCommand { check, files })
}

fn parse_lint_command(args: &[String]) -> Command {
 let fix = args.iter().any(|a| a == "--fix");
 let files: Vec<String> = args.iter()
 .filter(|a| !a.starts_with('-'))
 .cloned()
 .collect();
 
 Command::Lint(LintCommand { fix, files })
}

fn parse_doc_command(args: &[String]) -> Command {
 let open = args.iter().any(|a| a == "--open");
 let output = args.iter()
 .position(|a| a == "--output" || a == "-o")
 .and_then(|i| args.get(i + 1).cloned());
 
 Command::Doc(DocCommand { open, output })
}

fn parse_clean_command(args: &[String]) -> Command {
 let all = args.iter().any(|a| a == "--all");
 let target = args.iter()
 .position(|a| a == "--target")
 .and_then(|i| args.get(i + 1).cloned());
 
 Command::Clean(CleanCommand { all, target })
}

fn parse_new_command(args: &[String]) -> Command {
 let name = args.first().cloned().unwrap_or_else(|| "new-project".to_string());
 let path = args.iter()
 .position(|a| a == "--path")
 .and_then(|i| args.get(i + 1).cloned());
 let template = args.iter()
 .position(|a| a == "--template" || a == "-t")
 .and_then(|i| args.get(i + 1).cloned());
 
 Command::New(NewCommand { name, path, template })
}

fn parse_init_command(args: &[String]) -> Command {
 let name = args.iter()
 .find(|a| !a.starts_with('-'))
 .cloned();
 let template = args.iter()
 .position(|a| a == "--template" || a == "-t")
 .and_then(|i| args.get(i + 1).cloned());
 
 Command::Init(InitCommand { name, template })
}