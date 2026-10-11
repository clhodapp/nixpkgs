//! Find, write and check the imports that the modules of a tree need.
//!
//! The input is the output of `lib.modules.moduleDependencies` for one or
//! more evaluations of the modules, as JSON. The tool knows nothing about
//! the project the modules belong to.

mod plan;
mod reduce;
mod source;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use plan::{Dependencies, Plan, Settings, Status, Unwritten};

/// Print a line. The reader may stop reading early, as `head` does, and
/// that is not an error.
macro_rules! say {
    ($($argument:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(std::io::stdout(), $($argument)*);
    }};
}

#[derive(Parser)]
#[command(
    version,
    about = "Find, write and check the imports that the modules of a tree need"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Describe the imports that the modules need
    Report {
        #[command(flatten)]
        input: Input,
        /// Print everything the tool worked out, as JSON
        #[arg(long)]
        json: bool,
    },
    /// Write the needed imports into the module files
    Fix {
        #[command(flatten)]
        input: Input,
    },
    /// Fail when a module file does not have the imports it needs
    Check {
        #[command(flatten)]
        input: Input,
    },
}

#[derive(Args)]
struct Input {
    /// A file with the output of `lib.modules.moduleDependencies`, as
    /// JSON. Give the option once for each evaluation of the modules.
    #[arg(long, required = true, value_name = "FILE")]
    dependencies: Vec<PathBuf>,
    /// The directory with the modules to rewrite
    #[arg(long, value_name = "DIRECTORY")]
    root: PathBuf,
    /// The name of that directory in the evaluation, when the evaluation
    /// read the modules from another place, such as the Nix store
    #[arg(long, value_name = "PATH")]
    evaluated_root: Option<String>,
    /// Write imports only for the options a module sets, and not for the
    /// options it reads
    #[arg(long)]
    definitions_only: bool,
}

fn load(input: &Input) -> Result<Plan, String> {
    let mut dependencies = Dependencies::default();
    for path in &input.dependencies {
        let text =
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let more: Dependencies =
            serde_json::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))?;
        dependencies.merge(more);
    }
    let root = fs::canonicalize(&input.root)
        .map_err(|error| format!("{}: {error}", input.root.display()))?;
    let evaluated_root = match &input.evaluated_root {
        Some(evaluated_root) => evaluated_root.trim_end_matches('/').to_string(),
        None => root
            .to_str()
            .ok_or("the name of the root directory is not UTF-8")?
            .to_string(),
    };
    Ok(plan::plan(
        &dependencies,
        &Settings {
            root: &root,
            evaluated_root: &evaluated_root,
            definitions_only: input.definitions_only,
        },
    ))
}

fn count(plan: &Plan, status: Status) -> usize {
    plan.files
        .iter()
        .filter(|file| file.status == status)
        .count()
}

/// The lines for the uses that the tool writes no import for.
fn print_unwritten(title: &str, unwritten: &[Unwritten]) {
    if unwritten.is_empty() {
        return;
    }
    let mut by_file: BTreeMap<&str, usize> = BTreeMap::new();
    for entry in unwritten {
        let key = if entry.declared_in.is_empty() {
            entry.option.as_str()
        } else {
            entry.declared_in.as_str()
        };
        *by_file.entry(key).or_default() += 1;
    }
    say!("{title}: {} uses", unwritten.len());
    for (name, uses) in by_file {
        say!("  {name}: {uses}");
    }
}

fn print_unsupported(plan: &Plan) {
    for file in plan
        .files
        .iter()
        .filter(|file| file.status == Status::Unsupported)
    {
        say!(
            "  {}: {}",
            file.source,
            file.reason.as_deref().unwrap_or("unknown reason")
        );
        for import in &file.generated {
            say!("    needs {}", import.literal);
        }
    }
}

fn report(plan: &Plan) {
    say!(
        "modules in the evaluation: {} ({} in the tree)",
        plan.modules,
        plan.modules_in_tree
    );
    say!(
        "uses of the options of other modules: {} definitions, {} reads",
        plan.definitions,
        plan.reads
    );
    say!(
        "imports of files that a module uses and does not import in its own writing: {}",
        plan.direct
    );
    let generated: usize = plan
        .files
        .iter()
        .filter(|file| file.status != Status::Unsupported)
        .map(|file| file.generated.len())
        .sum();
    say!(
        "imports the tool writes: {generated} in {} files",
        plan.files
            .iter()
            .filter(|file| !file.generated.is_empty())
            .count()
    );
    say!("files to rewrite: {}", count(plan, Status::Stale));
    say!(
        "groups of modules that import each other, with those imports: {} (the largest has {} modules)",
        plan.cycles.groups,
        plan.cycles.largest
    );
    say!(
        "imports the tool writes inside those groups: {} (no set is smaller than {})",
        plan.cycles.imports,
        plan.cycles.lower_bound
    );

    let mut imported: BTreeMap<&str, usize> = BTreeMap::new();
    for import in plan.files.iter().flat_map(|file| &file.generated) {
        *imported.entry(&import.file).or_default() += 1;
    }
    let mut imported: Vec<(&str, usize)> = imported.into_iter().collect();
    imported.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
    if !imported.is_empty() {
        say!("files imported most often:");
        for (file, times) in imported.iter().take(10) {
            say!("  {file}: {times}");
        }
    }

    let unsupported = count(plan, Status::Unsupported);
    if unsupported > 0 {
        say!("files the tool cannot rewrite: {unsupported}");
        print_unsupported(plan);
    }
    print_unwritten(
        "options declared outside the tree, no import written",
        &plan.declared_outside,
    );
    print_unwritten(
        "options declared in a file that is not imported by its path, no import written",
        &plan.not_importable,
    );
    print_unwritten(
        "options with several declarations and no single description, no import written",
        &plan.no_declaration_to_choose,
    );
}

fn run(cli: Cli) -> Result<ExitCode, String> {
    match cli.command {
        Command::Report { input, json } => {
            let plan = load(&input)?;
            if json {
                let text =
                    serde_json::to_string_pretty(&plan).map_err(|error| error.to_string())?;
                say!("{text}");
            } else {
                report(&plan);
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Fix { input } => {
            let plan = load(&input)?;
            let root = &input.root;
            let mut written = 0;
            for file in &plan.files {
                if let Some(text) = &file.new_text {
                    let path = root.join(&file.source);
                    fs::write(&path, text)
                        .map_err(|error| format!("{}: {error}", path.display()))?;
                    written += 1;
                }
            }
            say!("rewrote {written} files");
            let unsupported = count(&plan, Status::Unsupported);
            if unsupported == 0 {
                return Ok(ExitCode::SUCCESS);
            }
            say!("{unsupported} files need their imports written by hand:");
            print_unsupported(&plan);
            Ok(ExitCode::FAILURE)
        }
        Command::Check { input } => {
            let plan = load(&input)?;
            let stale = count(&plan, Status::Stale);
            let unsupported = count(&plan, Status::Unsupported);
            if stale == 0 && unsupported == 0 {
                return Ok(ExitCode::SUCCESS);
            }
            if stale > 0 {
                say!("{stale} files do not have the generated imports they need:");
                for file in plan
                    .files
                    .iter()
                    .filter(|file| file.status == Status::Stale)
                {
                    say!("  {}", file.source);
                }
                say!("`nix-module-imports fix` with the same arguments rewrites them.");
            }
            if unsupported > 0 {
                say!("{unsupported} files need their imports written by hand:");
                print_unsupported(&plan);
            }
            Ok(ExitCode::FAILURE)
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
