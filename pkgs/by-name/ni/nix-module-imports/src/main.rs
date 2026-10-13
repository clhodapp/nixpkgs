//! Keep the imports of module files in step with what the modules
//! refer to.
//!
//! A module of the module system in `lib.modules` refers to options: it
//! sets them and reads them. It works on its own only when it imports
//! the modules that declare those options. This tool reads the source
//! of the modules, finds the references that the imports of a module do
//! not cover, and writes the imports that cover them. It evaluates
//! nothing, so what it finds holds for every configuration.

mod algebra;
mod memory;
mod reduce;
mod references;
mod source;
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
use std::time::{Duration, Instant};

use clap::{Args, Parser, Subcommand};

use touches::Touches;

#[derive(Parser)]
#[command(
    version,
    about = "Keep the imports of module files in step with what the modules refer to"
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

/// Which modules are looked at.
#[derive(Args)]
struct Which {
    /// A file that holds the list of the modules that are loaded
    /// always, as `nixos/modules/module-list.nix` does. Imports are
    /// generated only to these modules and to the modules that they
    /// import
    #[arg(long, value_name = "FILE")]
    module_list: Option<PathBuf>,
    /// A directory in which every file that is written as a module is
    /// looked at, also when no module of the list imports it. May be
    /// given more than once
    #[arg(long, value_name = "DIRECTORY")]
    tree: Vec<PathBuf>,
    /// An argument of modules that the caller of the modules gives
    /// without the tool knowing its value, as a caller can give `pkgs`.
    /// Without this, a module that takes an argument needs the module
    /// that sets it. May be given more than once
    #[arg(long, value_name = "NAME")]
    caller_arg: Vec<String>,
    /// About the most time to spend on finding the smallest set of
    /// imports among modules that import each other, as a number with
    /// `s`, `m` or `h` for seconds, minutes or hours, such as `10s` or
    /// `10m`. When the time is up, the tool goes on from what it has
    /// found and writes a set that has no import that could be left
    /// out, and that may not be the smallest. With `0` it does that for
    /// every group at once.
    #[arg(long, value_name = "TIME", default_value = "5m", value_parser = time)]
    search_approximate_after: Duration,
    /// More module files
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,
}

/// Read a time such as `10s`, `10m` or `2h`. A number without a unit is
/// refused, apart from `0`, so that nobody has to guess the unit.
fn time(text: &str) -> Result<Duration, String> {
    if text == "0" {
        return Ok(Duration::ZERO);
    }
    let seconds_in_unit = match text.chars().last() {
        Some('s') => 1,
        Some('m') => 60,
        Some('h') => 60 * 60,
        _ => {
            return Err(format!(
                "`{text}` has no unit: write `s`, `m` or `h` after the number, as in `10s` or `10m`"
            ));
        }
    };
    let number: u64 = text[..text.len() - 1]
        .parse()
        .map_err(|_| format!("`{text}` is not a whole number with `s`, `m` or `h` after it"))?;
    number
        .checked_mul(seconds_in_unit)
        .map(Duration::from_secs)
        .ok_or_else(|| format!("`{text}` is too long a time"))
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
    /// Print, as JSON, what is found for each module: the references
    /// that its imports do not cover, the imports that would cover
    /// them, and what could not be decided
    References {
        #[command(flatten)]
        reading: Reading,
        #[command(flatten)]
        which: Which,
    },
    /// Fail when a module does not have the generated imports that
    /// `fix` would write, or refers to an option that no module
    /// declares
    Check {
        #[command(flatten)]
        reading: Reading,
        #[command(flatten)]
        which: Which,
    },
    /// Write into each module the imports that cover its references.
    /// They go into the `imports` of the module, between two comments,
    /// and replace what was between the comments before
    Fix {
        #[command(flatten)]
        reading: Reading,
        #[command(flatten)]
        which: Which,
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

/// The files below a directory that are written as modules.
fn modules_below(directory: &Path, found: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        fs::read_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|error| format!("{}: {error}", directory.display()))?
            .path();
        if path.is_dir() {
            modules_below(&path, found)?;
        } else if path.extension().is_some_and(|extension| extension == "nix")
            && fs::read_to_string(&path).is_ok_and(|text| source::looks_like_module(&text))
        {
            found.push(path);
        }
    }
    Ok(())
}

/// A path without the `/` in front, which is how `source` takes the
/// place of a file in a tree.
fn in_tree(path: &Path) -> String {
    path.to_string_lossy().trim_start_matches('/').to_string()
}

/// The modules that a command looks at, and what is found for them.
struct Found {
    reports: BTreeMap<PathBuf, references::Report>,
    /// The generated imports that each file has now, as they are
    /// written: a directory where the import names one.
    written: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
    /// The modules that are imported by the name of their directory,
    /// with that name.
    by_directory: BTreeMap<PathBuf, PathBuf>,
    /// The files of a `--tree` that are written like a module and are
    /// not taken as modules of the system, with the reason.
    other: BTreeMap<PathBuf, String>,
}

fn find(reading: &Reading, which: &Which) -> Result<Found, String> {
    let mut roots = BTreeSet::new();
    let mut certain = BTreeSet::new();
    for file in &which.files {
        let file =
            fs::canonicalize(file).map_err(|error| format!("{}: {error}", file.display()))?;
        certain.insert(references::module_file(&file));
    }
    if let Some(list) = &which.module_list {
        // The list is read as a module that imports its entries, so
        // the entries are found like any other import.
        let list =
            fs::canonicalize(list).map_err(|error| format!("{}: {error}", list.display()))?;
        roots.insert(list.clone());
        certain.insert(list);
    }
    let mut files: Vec<PathBuf> = certain.iter().cloned().collect();
    for directory in &which.tree {
        modules_below(directory, &mut files)?;
    }
    if files.is_empty() {
        return Err("no module is given: name files, a `--module-list` or a `--tree`".to_string());
    }
    let mut touches = read(reading, files, true)?;
    let (modules, other) = references::modules_of_the_system(&touches, &certain);
    touches.retain(|file, _| modules.contains(file));
    eprintln!(
        "{} files read, of which {} are modules of the system",
        touches.len() + other.len(),
        touches.len()
    );
    let mut generated = BTreeMap::new();
    let mut written = BTreeMap::new();
    for file in touches.keys() {
        let directory = file.parent().map(in_tree).unwrap_or_default();
        let Ok(text) = fs::read_to_string(file) else {
            continue;
        };
        let Ok(module) = source::Module::parse(text) else {
            continue;
        };
        if let Ok(paths) = module.generated_imports(&directory) {
            let paths: BTreeSet<PathBuf> =
                paths.iter().map(|path| Path::new("/").join(path)).collect();
            if !paths.is_empty() {
                generated.insert(
                    file.clone(),
                    paths
                        .iter()
                        .map(|path| references::module_file(path))
                        .collect(),
                );
                written.insert(file.clone(), paths);
            }
        }
    }
    // The check runs on this thread, with the same limit as the
    // reading of one file.
    memory::limit(reading.memory_per_file * 1024 * 1024);
    let can_be_disabled: BTreeSet<PathBuf> = touches
        .values()
        .flat_map(|found| &found.disables)
        .map(|path| references::module_file(path))
        .collect();
    let mut reports = references::check(&references::Modules {
        files: &touches,
        roots: &roots,
        generated: &generated,
        caller_arguments: &which.caller_arg.iter().cloned().collect(),
        can_be_disabled: &can_be_disabled,
    })?;

    // An import is left out when the module reaches the file through
    // another import. The imports that a person wrote stay as they are.
    // An import of a module that can be disabled, and an import that
    // such a module has, is no way to reach anything, so those are
    // kept out of this and stay.
    let number: BTreeMap<&PathBuf, usize> = touches
        .keys()
        .enumerate()
        .map(|(index, file)| (file, index))
        .collect();
    let mut edges: reduce::Edges = vec![BTreeSet::new(); touches.len()];
    let mut removable = BTreeSet::new();
    let mut stays: BTreeMap<&PathBuf, BTreeSet<PathBuf>> = BTreeMap::new();
    for (file, found) in &touches {
        let from = number[file];
        let (imported, _) = references::imports(found);
        let written = generated.get(file);
        let one_way =
            |target: &PathBuf| can_be_disabled.contains(file) || can_be_disabled.contains(target);
        for target in &imported {
            if let Some(&to) = number.get(target)
                && !written.is_some_and(|written| written.contains(target))
                && !one_way(target)
            {
                edges[from].insert(to);
            }
        }
        for target in &reports[file].needed {
            if one_way(target) {
                stays.entry(file).or_default().insert(target.clone());
            } else if let Some(&to) = number.get(target)
                && edges[from].insert(to)
            {
                removable.insert((from, to));
            }
        }
    }
    let groups = reduce::reduce(
        &mut edges,
        &removable,
        Instant::now() + which.search_approximate_after,
    );
    if let Some(largest) = groups.iter().max_by_key(|group| group.modules) {
        eprintln!(
            "{} groups of modules import each other. The largest has {} modules and keeps {} \
             generated imports inside it; no set of fewer than {} would do.",
            groups.len(),
            largest.modules,
            largest.kept,
            largest.lower_bound
        );
    }
    let approximated = groups.iter().filter(|group| !group.smallest).count();
    if approximated > 0 {
        eprintln!(
            "the time for finding the smallest set was up: {approximated} of {} groups of \
             modules that import each other got a set that may not be the smallest",
            groups.len()
        );
    }
    let files: Vec<&PathBuf> = touches.keys().collect();
    for (file, report) in &mut reports {
        let from = number[file];
        report.needed = edges[from]
            .iter()
            .filter(|to| removable.contains(&(from, **to)))
            .map(|to| files[*to].clone())
            .chain(stays.get(file).into_iter().flatten().cloned())
            .collect();
    }
    // The modules that some module imports by the name of their
    // directory.
    let mut by_directory = BTreeMap::new();
    for found in touches.values() {
        for touch in &found.touches {
            if touch.kind == touches::Kind::Import
                && let Some(written) = touch.path.first()
            {
                let written = PathBuf::from(written);
                let file = references::module_file(&written);
                if file != written {
                    by_directory.insert(file, written);
                }
            }
        }
    }
    Ok(Found {
        reports,
        written,
        by_directory,
        other,
    })
}

/// Print to the standard output. A reader that has gone away, as `head`
/// does, is no error.
fn print(text: &str) {
    let _ = writeln!(std::io::stdout(), "{text}");
}

/// The path literal that names `target` in the file `from`.
fn literal(from: &Path, target: &Path) -> String {
    let directory = from.parent().map(in_tree).unwrap_or_default();
    source::relative_literal(&directory, &in_tree(target))
}

/// The generated imports that a file is to have, as they are written.
///
/// A module that is a `default.nix` is named by its directory where
/// the modules name it so already. The module system tells modules
/// apart by the path they are imported with, and would load the module
/// a second time under the other name.
fn wanted(found: &Found, report: &references::Report) -> BTreeSet<PathBuf> {
    report
        .needed
        .iter()
        .map(|target| found.by_directory.get(target).unwrap_or(target).clone())
        .collect()
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        Command::Touches { reading, files } => {
            let touches = read(&reading, files, false)?;
            print(&serde_json::to_string_pretty(&touches).map_err(|error| error.to_string())?);
            Ok(ExitCode::SUCCESS)
        }
        Command::References { reading, which } => {
            let found = find(&reading, &which)?;
            let report = serde_json::json!({
                "modules": found.reports,
                "not_modules": found.other,
            });
            print(&serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?);
            Ok(ExitCode::SUCCESS)
        }
        Command::Check { reading, which } => {
            let found = find(&reading, &which)?;
            let none = BTreeSet::new();
            let mut failed = 0;
            let (mut undecided, mut optional) = (0, 0);
            for (file, report) in &found.reports {
                let now = found.written.get(file).unwrap_or(&none);
                let wanted = wanted(&found, report);
                let mut lines = Vec::new();
                for missing in wanted.difference(now) {
                    let declaring = references::module_file(missing);
                    let options: Vec<&str> = report
                        .not_imported
                        .iter()
                        .filter(|reference| reference.declared_in.contains(&declaring))
                        .map(|reference| reference.option.as_str())
                        .collect();
                    lines.push(format!(
                        "  needs the import {}{}",
                        literal(file, missing),
                        if options.is_empty() {
                            String::new()
                        } else {
                            format!(", for {}", options.join(", "))
                        }
                    ));
                }
                for stale in now.difference(&wanted) {
                    lines.push(format!(
                        "  has the generated import {}, which is not needed there",
                        literal(file, stale)
                    ));
                }
                for path in &report.not_declared {
                    lines.push(format!("  can {path}, which no module declares"));
                }
                undecided += usize::from(!report.not_known.is_empty());
                optional += usize::from(!report.optional.is_empty());
                if !lines.is_empty() {
                    failed += 1;
                    print(&format!("{}:", file.display()));
                    print(&lines.join("\n"));
                }
            }
            eprintln!(
                "{} modules checked, {failed} with a finding. {undecided} have a reference \
                 whose option is not known, and {optional} refer to an option of a module that \
                 is not loaded always; `references` lists both.",
                found.reports.len()
            );
            if failed > 0 {
                eprintln!("`nix-module-imports fix` writes the imports.");
                return Ok(ExitCode::FAILURE);
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Fix { reading, which } => {
            let found = find(&reading, &which)?;
            let none = BTreeSet::new();
            let (mut written, mut refused) = (0, 0);
            for (file, report) in &found.reports {
                let wanted = wanted(&found, report);
                if &wanted == found.written.get(file).unwrap_or(&none) {
                    continue;
                }
                let mut paths: Vec<String> =
                    wanted.iter().map(|target| literal(file, target)).collect();
                paths.sort();
                let text = fs::read_to_string(file)
                    .map_err(|error| format!("{}: {error}", file.display()))?;
                let rewritten = source::Module::parse(text)
                    .and_then(|module| module.with_generated_imports(&paths));
                match rewritten {
                    Ok(text) => {
                        fs::write(file, text)
                            .map_err(|error| format!("{}: {error}", file.display()))?;
                        written += 1;
                    }
                    Err(why) => {
                        refused += 1;
                        print(&format!(
                            "{}: {why}; it needs the imports {}",
                            file.display(),
                            paths.join(" ")
                        ));
                    }
                }
            }
            eprintln!("{written} files written, {refused} that could not be written");
            Ok(if refused > 0 {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            })
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
