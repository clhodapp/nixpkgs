//! Which references of a module are not covered by its imports.
//!
//! A module refers to an option when it sets it or reads it. The module
//! system needs the option to be declared: a module that sets an option
//! that no loaded module declares fails, whatever `mkIf` is around the
//! setting, and a module that reads one fails when the read is worked
//! out. A module that is loaded on its own gets the declarations of the
//! modules it imports, and of the modules those import. So a reference
//! is covered when the module itself declares the option, or a module
//! that it reaches through imports does.
//!
//! The input is what each file declares, sets and reads, from `touches`,
//! for the modules of a list and for every module that one of them
//! imports. Nothing is evaluated.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::touches::{Kind, Touches};

/// How many options one reference may meet before it is reported as one
/// whose option is not known, and not as a reference to each of them.
const WIDE: usize = 50;

/// The declared options, as a tree by name. A node is an option, or has
/// the names below it.
#[derive(Default)]
struct Declared {
    /// The files that declare an option here.
    files: BTreeSet<PathBuf>,
    below: BTreeMap<String, Declared>,
}

impl Declared {
    fn declare(&mut self, path: &[String], file: &Path) {
        match path.split_first() {
            None => {
                self.files.insert(file.to_path_buf());
            }
            Some((name, rest)) => self
                .below
                .entry(name.clone())
                .or_default()
                .declare(rest, file),
        }
    }

    /// Every option at or below this node.
    fn all<'a>(&'a self, path: &mut Vec<String>, found: &mut Vec<(Vec<String>, &'a Declared)>) {
        if !self.files.is_empty() {
            found.push((path.clone(), self));
            return;
        }
        for (name, below) in &self.below {
            path.push(name.clone());
            below.all(path, found);
            path.pop();
        }
    }

    /// The options that a reference at `rest` below this node meets.
    /// The first option on the way is the one that is met: what comes
    /// after it in the path is inside the value of that option. A
    /// reference that ends above options meets all of them when it has
    /// everything below it, and none otherwise. A name that is not
    /// known, on either side, stands for any name.
    fn meet<'a>(
        &'a self,
        rest: &[String],
        open: bool,
        path: &mut Vec<String>,
        found: &mut Vec<(Vec<String>, &'a Declared)>,
    ) {
        if !self.files.is_empty() {
            found.push((path.clone(), self));
            return;
        }
        let Some((name, rest)) = rest.split_first() else {
            if open {
                self.all(path, found);
            }
            return;
        };
        for (declared, below) in &self.below {
            if name == "*" || declared == "*" || declared == name {
                path.push(declared.clone());
                below.meet(rest, open, path, found);
                path.pop();
            }
        }
    }
}

/// One reference of a file that its imports do not cover.
#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reference {
    /// `set` or `read`.
    pub kind: Kind,
    /// The option.
    pub option: String,
    /// The files that declare it.
    pub declared_in: Vec<PathBuf>,
}

#[derive(Serialize, Default)]
pub struct Report {
    /// References to an option that a file declares which this file
    /// does not reach through its imports.
    pub not_imported: Vec<Reference>,
    /// The paths that the file sets or reads and that no file of the
    /// input declares, with `set` or `read` in front.
    pub not_declared: BTreeSet<String>,
    /// The paths that meet so many options, or have a name that is not
    /// known so far up, that the option is not known.
    pub not_known: BTreeSet<String>,
    /// Whether the file imports something that is not known, so that
    /// what it reaches is not all known.
    pub imports_unknown: bool,
}

/// The file that an import names: a directory is its `default.nix`.
pub fn module_file(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.join("default.nix")
    } else {
        path.to_path_buf()
    }
}

/// The paths that a file imports, and whether it imports something that
/// is not known.
pub fn imports(touches: &Touches) -> (Vec<PathBuf>, bool) {
    let mut paths = Vec::new();
    let mut unknown = false;
    for touch in &touches.touches {
        if touch.kind != Kind::Import {
            continue;
        }
        match touch.path.first() {
            Some(path) if path != "*" && !touch.open => paths.push(module_file(Path::new(path))),
            _ => unknown = true,
        }
    }
    (paths, unknown)
}

/// For each file, the references that its imports do not cover.
/// `files` holds what each file touches, for every file that is loaded
/// always.
pub fn check(files: &BTreeMap<PathBuf, Touches>) -> BTreeMap<PathBuf, Report> {
    let mut declared = Declared::default();
    // The module system declares the options below `_module` itself,
    // for every module.
    let module_system = PathBuf::from("the module system");
    declared.declare(&["_module".to_string()], &module_system);
    for (file, touches) in files {
        for touch in &touches.touches {
            // A declaration with more below it that is not known, as
            // from options that are put together from parts, declares
            // what is below its path. It is an option for this purpose:
            // a reference at or below it is one to this file.
            if touch.kind == Kind::Declare && !touch.path.is_empty() {
                declared.declare(&touch.path, file);
            }
        }
    }

    // What each file reaches through imports, itself included.
    let direct: BTreeMap<&PathBuf, (Vec<PathBuf>, bool)> = files
        .iter()
        .map(|(file, touches)| (file, imports(touches)))
        .collect();
    let reach = |start: &PathBuf| -> (BTreeSet<PathBuf>, bool) {
        let mut seen = BTreeSet::from([start.clone(), module_system.clone()]);
        let mut unknown = false;
        let mut stack = vec![start.clone()];
        while let Some(file) = stack.pop() {
            let Some((paths, some_unknown)) = direct.get(&file) else {
                continue;
            };
            unknown |= some_unknown;
            for path in paths {
                if seen.insert(path.clone()) {
                    stack.push(path.clone());
                }
            }
        }
        (seen, unknown)
    };

    let mut reports = BTreeMap::new();
    for (file, touches) in files {
        let (reached, imports_unknown) = reach(file);
        let mut report = Report {
            imports_unknown,
            ..Report::default()
        };
        let mut not_imported: BTreeSet<Reference> = BTreeSet::new();
        for touch in &touches.touches {
            // A read has `config` or `options` in front of the path.
            let (label, path) = match touch.kind {
                Kind::Set => ("set", &touch.path[..]),
                Kind::Read => ("read", touch.path.get(1..).unwrap_or(&[])),
                // A path that is only asked for needs no declaration.
                Kind::Declare | Kind::Import | Kind::Ask => continue,
            };
            if path.is_empty() {
                if touch.open {
                    report.not_known.insert(format!("{label} anything"));
                }
                continue;
            }
            let shown = format!(
                "{label} {}{}",
                path.join("."),
                if touch.open { " ..." } else { "" }
            );
            if path[0] == "*" {
                report.not_known.insert(shown);
                continue;
            }
            let mut found = Vec::new();
            declared.meet(path, touch.open, &mut Vec::new(), &mut found);
            if found.len() > WIDE {
                report.not_known.insert(shown);
                continue;
            }
            // A path that goes down to names and meets no option: the
            // file sets or reads something that nothing declares. A
            // path that ends above the options and has nothing below it
            // is a step on the way to another path, and is not one.
            if found.is_empty() {
                let mut above = Vec::new();
                declared.meet(path, true, &mut Vec::new(), &mut above);
                if above.is_empty() {
                    report.not_declared.insert(shown);
                }
                continue;
            }
            for (option, node) in found {
                if node
                    .files
                    .iter()
                    .any(|declaring| reached.contains(declaring))
                {
                    continue;
                }
                not_imported.insert(Reference {
                    kind: touch.kind,
                    option: option.join("."),
                    declared_in: node.files.iter().cloned().collect(),
                });
            }
        }
        report.not_imported = not_imported.into_iter().collect();
        reports.insert(file.clone(), report);
    }
    reports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::touches::Touch;

    fn touches(entries: &[(Kind, &str, bool)]) -> Touches {
        Touches {
            touches: entries
                .iter()
                .map(|(kind, path, open)| Touch {
                    kind: *kind,
                    path: if *kind == Kind::Import {
                        vec![path.to_string()]
                    } else {
                        path.split('.').map(str::to_string).collect()
                    },
                    open: *open,
                })
                .collect(),
            ..Touches::default()
        }
    }

    fn run(files: &[(&str, Touches)]) -> BTreeMap<String, Vec<String>> {
        let files: BTreeMap<PathBuf, Touches> = files
            .iter()
            .map(|(file, touches)| {
                (
                    PathBuf::from(file),
                    Touches {
                        touches: touches.touches.clone(),
                        ..Touches::default()
                    },
                )
            })
            .collect();
        check(&files)
            .into_iter()
            .map(|(file, report)| {
                let mut lines: Vec<String> = report
                    .not_imported
                    .iter()
                    .map(|reference| format!("{:?} {}", reference.kind, reference.option))
                    .collect();
                lines.extend(
                    report
                        .not_declared
                        .iter()
                        .map(|path| format!("not declared: {path}")),
                );
                lines.extend(
                    report
                        .not_known
                        .iter()
                        .map(|path| format!("not known: {path}")),
                );
                (file.display().to_string(), lines)
            })
            .collect()
    }

    #[test]
    fn a_reference_is_covered_by_an_import_of_an_import() {
        let found = run(&[
            (
                "/m/a.nix",
                touches(&[
                    (Kind::Import, "/m/b.nix", false),
                    (Kind::Set, "systemd.services.a.wantedBy", false),
                    (Kind::Read, "config.users.users.a", true),
                ]),
            ),
            (
                "/m/b.nix",
                touches(&[(Kind::Import, "/m/systemd.nix", false)]),
            ),
            (
                "/m/systemd.nix",
                touches(&[(Kind::Declare, "systemd.services", false)]),
            ),
            (
                "/m/users.nix",
                touches(&[(Kind::Declare, "users.users", false)]),
            ),
        ]);
        // The setting is covered through two imports. The read is not.
        assert_eq!(found["/m/a.nix"], ["Read users.users"]);
    }

    #[test]
    fn a_step_on_the_way_is_no_reference_and_a_path_to_nothing_is_one() {
        let found = run(&[
            (
                "/m/a.nix",
                touches(&[
                    (Kind::Read, "config.services", false),
                    (Kind::Read, "config.services.a.enable", true),
                    (Kind::Set, "servics.a.enable", false),
                ]),
            ),
            (
                "/m/b.nix",
                touches(&[(Kind::Declare, "services.a.enable", false)]),
            ),
        ]);
        assert_eq!(
            found["/m/a.nix"],
            [
                "Read services.a.enable",
                "not declared: set servics.a.enable"
            ]
        );
    }
}
