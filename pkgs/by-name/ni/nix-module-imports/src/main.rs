//! Find the references of module files that their imports do not cover.
//!
//! A module of the module system in `lib.modules` refers to options: it
//! sets them and reads them. It works on its own only when it imports
//! the modules that declare those options. This tool reads the source
//! of the modules and reports the references that the imports of a
//! module do not cover. It evaluates nothing, so what it reports holds
//! for every configuration.

mod algebra;
mod memory;
mod references;
mod touches;

#[global_allocator]
static ALLOCATOR: memory::Counting = memory::Counting;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use clap::{Args, Parser, Subcommand};

use touches::Touches;

#[derive(Parser)]
#[command(
    version,
    about = "Find the references of module files that their imports do not cover"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// How the module files are read.
#[derive(Args)]
struct Reading {
    /// The directory of the library that the modules get as `lib`
    #[arg(long, value_name = "DIRECTORY")]
    library: PathBuf,
    /// An argument that the caller of the modules gives by name, as
    /// NixOS gives `modulesPath`. May be given more than once. An
    /// argument that is not given is read from the configuration at
    /// `_module.args`
    #[arg(long, value_name = "NAME=TEXT")]
    special_arg: Vec<String>,
    /// A directory whose files are not read when a module imports one
    /// of them. What the file gives is then not known, and what a
    /// module passes to it counts as read in full. May be given more
    /// than once
    #[arg(long, value_name = "DIRECTORY")]
    unread: Vec<PathBuf>,
    /// How many files are read at a time. The default is the number of
    /// processors, and at most eight. The run takes at most this number
    /// times `--memory-per-file`
    #[arg(long, value_name = "NUMBER")]
    jobs: Option<usize>,
    /// How much memory the reading of one file may take, in megabytes.
    /// A file that needs more is not read to the end: it is reported
    /// with that as the reason and listed as one that can touch
    /// anything. Read it again with a larger number, and fewer `--jobs`
    #[arg(long, value_name = "MEGABYTES", default_value_t = 2048)]
    memory_per_file: usize,
}

#[derive(Subcommand)]
enum Command {
    /// Print, as JSON, what each of the given module files can touch:
    /// the options it can declare, the settings it can make, what it
    /// can read of the configuration and the files it imports
    Touches {
        #[command(flatten)]
        reading: Reading,
        /// The module files
        #[arg(required = true, value_name = "FILE")]
        files: Vec<PathBuf>,
    },
    /// Print, as JSON, the references of each module that its imports
    /// do not cover. The modules are the ones of the list and the given
    /// files, and every module that one of them imports. These are the
    /// modules that are loaded always
    References {
        #[command(flatten)]
        reading: Reading,
        /// A file that holds the list of the modules, as
        /// `nixos/modules/module-list.nix` does
        #[arg(long, value_name = "FILE")]
        module_list: Option<PathBuf>,
        /// More module files
        #[arg(value_name = "FILE")]
        files: Vec<PathBuf>,
    },
}

/// What each of the files touches. With `follow`, also what each file
/// touches that one of them imports, and so on.
fn read(
    reading: &Reading,
    files: Vec<PathBuf>,
    follow: bool,
) -> Result<BTreeMap<PathBuf, Touches>, String> {
    let library = fs::canonicalize(&reading.library)
        .map_err(|error| format!("{}: {error}", reading.library.display()))?;
    let mut special = BTreeMap::new();
    for argument in &reading.special_arg {
        let (name, text) = argument
            .split_once('=')
            .ok_or_else(|| format!("`{argument}` is not of the form NAME=TEXT"))?;
        special.insert(name.to_string(), text.to_string());
    }
    let mut unread = Vec::new();
    for directory in &reading.unread {
        unread.push(
            fs::canonicalize(directory)
                .map_err(|error| format!("{}: {error}", directory.display()))?,
        );
    }
    let jobs = reading
        .jobs
        .or_else(|| {
            std::thread::available_parallelism()
                .ok()
                .map(|processors| usize::from(processors).min(8))
        })
        .unwrap_or(1)
        .max(1);

    let mut done: BTreeMap<PathBuf, Touches> = BTreeMap::new();
    let mut waiting: Vec<PathBuf> = Vec::new();
    for file in files {
        let path =
            fs::canonicalize(&file).map_err(|error| format!("{}: {error}", file.display()))?;
        waiting.push(references::module_file(&path));
    }
    while !waiting.is_empty() {
        waiting.sort();
        waiting.dedup();
        waiting.retain(|file| !done.contains_key(file));
        let next = AtomicUsize::new(0);
        let found: Mutex<Vec<(PathBuf, Touches)>> = Mutex::new(Vec::new());
        std::thread::scope(|scope| {
            for _ in 0..jobs.min(waiting.len()) {
                // The interpretation goes deep into itself, so each
                // thread gets a large stack.
                let spawned = std::thread::Builder::new()
                    .stack_size(256 * 1024 * 1024)
                    .spawn_scoped(scope, || {
                        loop {
                            let index = next.fetch_add(1, Ordering::Relaxed);
                            let Some(file) = waiting.get(index) else {
                                break;
                            };
                            // Each file is read by an interpreter of its
                            // own, so that nothing that was worked out
                            // for one file is taken over for another.
                            let mut interpreter = algebra::Interpreter::new();
                            interpreter.unread = unread.clone();
                            interpreter.limit_memory(reading.memory_per_file * 1024 * 1024);
                            let touches =
                                touches::of_file(&mut interpreter, file, &library, &special);
                            // What the file made is given back before
                            // the next file is read.
                            drop(interpreter);
                            algebra::release();
                            found.lock().unwrap().push((file.clone(), touches));
                        }
                    });
                spawned.expect("a thread to read files could not be started");
            }
        });
        waiting.clear();
        for (file, touches) in found.into_inner().unwrap() {
            if let Some(why) = &touches.stopped {
                eprintln!("nix-module-imports: {}: {why}", file.display());
            }
            if follow {
                let (imported, _) = references::imports(&touches);
                waiting.extend(imported.into_iter().filter(|path| path.is_file()));
            }
            done.insert(file, touches);
        }
    }
    Ok(done)
}

/// Print to the standard output. A reader that has gone away, as `head`
/// does, is no error.
fn print(text: &str) {
    let _ = writeln!(std::io::stdout(), "{text}");
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        Command::Touches { reading, files } => {
            let touches = read(&reading, files, false)?;
            print(&serde_json::to_string_pretty(&touches).map_err(|error| error.to_string())?);
            Ok(ExitCode::SUCCESS)
        }
        Command::References {
            reading,
            module_list,
            mut files,
        } => {
            // The list is read as a module that imports its entries, so
            // the entries are found like any other import.
            files.extend(module_list);
            if files.is_empty() {
                return Err("no module is given: name files or a `--module-list`".to_string());
            }
            let touches = read(&reading, files, true)?;
            eprintln!("{} files read", touches.len());
            // The check runs on this thread, with the same limit as the
            // reading of one file.
            memory::limit(reading.memory_per_file * 1024 * 1024);
            let reports = references::check(&touches)?;
            let with_findings: BTreeMap<&Path, &references::Report> = reports
                .iter()
                .filter(|(_, report)| {
                    !report.not_imported.is_empty()
                        || !report.not_declared.is_empty()
                        || !report.not_known.is_empty()
                })
                .map(|(file, report)| (file.as_path(), report))
                .collect();
            print(
                &serde_json::to_string_pretty(&with_findings).map_err(|error| error.to_string())?,
            );
            let options: BTreeSet<&str> = reports
                .values()
                .flat_map(|report| {
                    report
                        .not_imported
                        .iter()
                        .map(|found| found.option.as_str())
                })
                .collect();
            eprintln!(
                "{} modules read; {} have a reference that their imports do not cover, to {} options; \
                 {} refer to something that no module declares; {} have a reference whose option is not known",
                reports.len(),
                reports
                    .values()
                    .filter(|report| !report.not_imported.is_empty())
                    .count(),
                options.len(),
                reports
                    .values()
                    .filter(|report| !report.not_declared.is_empty())
                    .count(),
                reports
                    .values()
                    .filter(|report| !report.not_known.is_empty())
                    .count(),
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("nix-module-imports: {message}");
            ExitCode::from(2)
        }
    }
}
