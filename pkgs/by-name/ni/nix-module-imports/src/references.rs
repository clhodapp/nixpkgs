//! Which references of a module its imports cover, and which imports
//! would cover the rest.
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
//! The input is what each file declares, sets and reads, from `touches`.
//! Nothing is evaluated.
//!
//! An option can be declared inside another: `systemd.services` is an
//! option, and each file that declares it with a `submodule` declares
//! options of every service. A reference is one to the innermost option
//! that it names. So a setting of `systemd.services.x.serviceConfig`
//! needs the file that declares `serviceConfig`, and not a file that
//! adds some other option to every service.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::touches::{Kind, Touches};

/// How many options with different files that declare them one
/// reference may meet before it is reported as one whose option is not
/// known, and not as a reference to each of them.
const WIDE: usize = 50;

/// The declared options, as a tree by name.
#[derive(Default)]
struct Declared {
    /// The files that declare an option here.
    files: BTreeSet<PathBuf>,
    /// Those of the files that declare the option in a way that does not
    /// show what options are inside it.
    opaque: BTreeSet<PathBuf>,
    below: BTreeMap<String, Declared>,
}

/// An option that a reference meets.
struct Met<'a> {
    option: Vec<String>,
    node: &'a Declared,
    /// Whether the reference goes on below the option to a name that no
    /// declaration inside the option has.
    further: bool,
}

impl Declared {
    fn declare(&mut self, path: &[String], file: &Path, opaque: bool) {
        match path.split_first() {
            None => {
                self.files.insert(file.to_path_buf());
                if opaque {
                    self.opaque.insert(file.to_path_buf());
                }
            }
            Some((name, rest)) => self
                .below
                .entry(name.clone())
                .or_default()
                .declare(rest, file, opaque),
        }
    }

    /// The outermost options at or below this node.
    fn all<'a>(&'a self, path: &mut Vec<String>, found: &mut Vec<Met<'a>>) {
        if !self.files.is_empty() {
            found.push(Met {
                option: path.clone(),
                node: self,
                further: false,
            });
            return;
        }
        for (name, below) in &self.below {
            path.push(name.clone());
            below.all(path, found);
            path.pop();
        }
    }

    /// How many options are declared below this node by a file.
    fn count(&self, file: &Path) -> usize {
        self.below
            .values()
            .map(|below| usize::from(below.files.contains(file)) + below.count(file))
            .sum()
    }

    /// The options that a reference at `rest` below this node meets:
    /// on each way down the names, the innermost option. `last` is the
    /// innermost option above this node. A reference that ends above
    /// every option meets the outermost options below it when it has
    /// everything below it, and none otherwise. A name that is not
    /// known, on either side, stands for any name.
    fn meet<'a>(
        &'a self,
        rest: &[String],
        open: bool,
        path: &mut Vec<String>,
        last: Option<(Vec<String>, &'a Declared)>,
        found: &mut Vec<Met<'a>>,
    ) {
        let last = if self.files.is_empty() {
            last
        } else {
            Some((path.clone(), self))
        };
        let Some((name, rest)) = rest.split_first() else {
            match last {
                Some((option, node)) => found.push(Met {
                    option,
                    node,
                    further: false,
                }),
                None if open => self.all(path, found),
                None => {}
            }
            return;
        };
        let mut matched = false;
        for (declared, below) in &self.below {
            if name == "*" || declared == "*" || declared == name {
                matched = true;
                path.push(declared.clone());
                below.meet(rest, open, path, last.clone(), found);
                path.pop();
            }
        }
        if !matched && let Some((option, node)) = last {
            found.push(Met {
                option,
                node,
                further: true,
            });
        }
    }
}

/// One reference of a file.
#[derive(Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    /// The imports that cover the references which the imports that a
    /// person wrote do not cover. These are the generated imports that
    /// the file is to have.
    pub needed: BTreeSet<PathBuf>,
    /// The references that the imports of the file do not cover now.
    pub not_imported: Vec<Reference>,
    /// The references to an option that only a module outside the
    /// modules that are loaded always declares. No import is generated
    /// for them: it would load that module wherever this one is loaded.
    pub optional: Vec<Reference>,
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

/// What the check works on.
pub struct Modules<'a> {
    /// What each file touches.
    pub files: &'a BTreeMap<PathBuf, Touches>,
    /// The files that start what is loaded always, as the list of the
    /// modules of NixOS does. A file that one of them reaches through
    /// imports is loaded always too.
    pub roots: &'a BTreeSet<PathBuf>,
    /// The generated imports that each file has now.
    pub generated: &'a BTreeMap<PathBuf, BTreeSet<PathBuf>>,
    /// The arguments of modules that the caller of the modules gives.
    pub caller_arguments: &'a BTreeSet<String>,
    /// The modules that some module disables with `disabledModules`.
    /// Where one of them is disabled, what it imports is not loaded
    /// through it. So no module counts as reaching a file through one
    /// of them.
    pub can_be_disabled: &'a BTreeSet<PathBuf>,
}

/// The files that `starts` reach through imports, themselves included,
/// and whether one of them imports something that is not known. A file
/// in `not_through` is reached, and what it imports is not followed.
fn reach(
    edges: &BTreeMap<&PathBuf, (Vec<PathBuf>, bool)>,
    starts: impl Iterator<Item = PathBuf>,
    not_through: &BTreeSet<PathBuf>,
) -> (BTreeSet<PathBuf>, bool) {
    let mut seen = BTreeSet::new();
    let mut unknown = false;
    let mut stack: Vec<PathBuf> = starts.collect();
    let starts: BTreeSet<PathBuf> = stack.iter().cloned().collect();
    while let Some(file) = stack.pop() {
        if !seen.insert(file.clone()) {
            continue;
        }
        if not_through.contains(&file) && !starts.contains(&file) {
            continue;
        }
        let Some((paths, some_unknown)) = edges.get(&file) else {
            continue;
        };
        unknown |= some_unknown;
        stack.extend(paths.iter().cloned());
    }
    (seen, unknown)
}

impl Declared {
    /// Whether a file that declares `path`, and the paths in `own`,
    /// can be a module next to the declarations of this tree: it
    /// declares no option above one of them, and none inside one of
    /// them that it does not declare itself.
    fn fits(&self, path: &[String], own: &BTreeSet<&[String]>) -> bool {
        let mut node = self;
        for (index, name) in path.iter().enumerate() {
            if !node.files.is_empty() && !own.contains(&path[..index]) {
                return false;
            }
            match node.below.get(name) {
                Some(below) => node = below,
                None => return true,
            }
        }
        node.below.is_empty() || !node.files.is_empty()
    }
}

/// The files that are modules of one system, of the files that were
/// found in a directory.
///
/// A directory of modules also holds files that are written like a
/// module and are none of this system: a file of helpers that takes
/// the same arguments, and a module of another system, such as the
/// options of a `submodule` that are kept in a file of their own. The
/// modules of the system are the `certain` ones, every file that one
/// of them imports, and every other file that makes settings which all
/// are settings of options that these modules or the file itself
/// declare, and that declares nothing which cannot stand next to their
/// declarations. A file that is taken this way can make another one
/// fit, so this is repeated until no file is added.
///
/// The result is the modules of the system, and for every other file
/// the reason why it is none.
pub fn modules_of_the_system(
    files: &BTreeMap<PathBuf, Touches>,
    certain: &BTreeSet<PathBuf>,
) -> (BTreeSet<PathBuf>, BTreeMap<PathBuf, String>) {
    let edges: BTreeMap<&PathBuf, (Vec<PathBuf>, bool)> = files
        .iter()
        .map(|(file, touches)| (file, imports(touches)))
        .collect();
    let all_the_way = BTreeSet::new();
    let (mut taken, _) = reach(&edges, certain.iter().cloned(), &all_the_way);
    // A file that another one loads as a value is a part of that file.
    let loaded: BTreeSet<&PathBuf> = files.values().flat_map(|touches| &touches.loads).collect();
    loop {
        let mut declared = Declared::default();
        declared.declare(
            &["_module".to_string()],
            Path::new("the module system"),
            true,
        );
        for file in &taken {
            for touch in files.get(file).iter().flat_map(|touches| &touches.touches) {
                if touch.kind == Kind::Declare && !touch.path.is_empty() {
                    declared.declare(&touch.path, file, touch.open);
                }
            }
        }
        let mut added = Vec::new();
        let mut reasons = BTreeMap::new();
        for file in files.keys() {
            if taken.contains(file) {
                continue;
            }
            if loaded.contains(file) {
                reasons.insert(
                    file.clone(),
                    "another file loads it with `import`".to_string(),
                );
                continue;
            }
            // The file is looked at together with what it imports.
            let (with_imports, _) = reach(&edges, std::iter::once(file.clone()), &all_the_way);
            let touches = || {
                with_imports
                    .iter()
                    .filter_map(|file| files.get(file))
                    .flat_map(|touches| &touches.touches)
            };
            let own: BTreeSet<&[String]> = touches()
                .filter(|touch| touch.kind == Kind::Declare)
                .map(|touch| &touch.path[..])
                .collect();
            let mut own_declared = Declared::default();
            for path in &own {
                own_declared.declare(path, file, false);
            }
            let mut settings = 0;
            let mut reason = None;
            for touch in touches() {
                let path = touch.path.join(".");
                match touch.kind {
                    // A declaration at no path is one of options that
                    // are not known, and says nothing either way.
                    Kind::Declare
                        if !touch.path.is_empty() && !declared.fits(&touch.path, &own) =>
                    {
                        reason = Some(format!(
                            "it declares {path}, which cannot stand next to the options of the modules"
                        ));
                    }
                    // A setting at a name that is not known says
                    // nothing either way.
                    Kind::Set if touch.path.first().is_some_and(|name| name != "*") => {
                        let mut found = Vec::new();
                        for tree in [&declared, &own_declared] {
                            tree.meet(&touch.path, true, &mut Vec::new(), None, &mut found);
                        }
                        if found.is_empty() {
                            reason = Some(format!("it sets {path}, which no module declares"));
                        } else {
                            settings += 1;
                        }
                    }
                    _ => {}
                }
                if reason.is_some() {
                    break;
                }
            }
            if reason.is_none() && settings == 0 {
                reason = Some("it makes no setting of an option".to_string());
            }
            match reason {
                Some(reason) => {
                    reasons.insert(file.clone(), reason);
                }
                None => added.push(file.clone()),
            }
        }
        if added.is_empty() {
            return (taken, reasons);
        }
        let (more, _) = reach(&edges, added.into_iter(), &all_the_way);
        taken.extend(more);
    }
}

/// For each file, the references that its imports do not cover and the
/// imports that would cover them.
///
/// The check stops with an error when the thread passes its limit of
/// memory. See `memory::limit`.
pub fn check(modules: &Modules) -> Result<BTreeMap<PathBuf, Report>, String> {
    let files = modules.files;
    let mut declared = Declared::default();
    // The module system declares the options below `_module` itself,
    // for every module.
    let module_system = PathBuf::from("the module system");
    declared.declare(&["_module".to_string()], &module_system, true);
    // The files that set an argument of modules, by the argument.
    let mut arguments: BTreeMap<&str, BTreeSet<&PathBuf>> = BTreeMap::new();
    for (file, touches) in files {
        for touch in &touches.touches {
            match touch.kind {
                // A declaration with more below it that is not known,
                // as from options that are put together from parts,
                // declares what is below its path.
                Kind::Declare if !touch.path.is_empty() => {
                    declared.declare(&touch.path, file, touch.open);
                }
                Kind::Set => {
                    if let [first, second, name, ..] = &touch.path[..]
                        && first == "_module"
                        && second == "args"
                        && name != "*"
                    {
                        arguments.entry(name).or_default().insert(file);
                    }
                }
                _ => {}
            }
        }
    }

    // The imports of each file: all of them, and those that a person
    // wrote. A reference counts as covered by hand only through the
    // second kind, in every file on the way, so that the generated
    // imports of one file do not depend on those of another.
    let all: BTreeMap<&PathBuf, (Vec<PathBuf>, bool)> = files
        .iter()
        .map(|(file, touches)| (file, imports(touches)))
        .collect();
    let by_hand: BTreeMap<&PathBuf, (Vec<PathBuf>, bool)> = all
        .iter()
        .map(|(file, (paths, unknown))| {
            let generated = modules.generated.get(*file);
            let paths = paths
                .iter()
                .filter(|path| !generated.is_some_and(|generated| generated.contains(*path)))
                .cloned()
                .collect();
            (*file, (paths, *unknown))
        })
        .collect();
    let (always, _) = reach(&all, modules.roots.iter().cloned(), &BTreeSet::new());

    // The file to import for an option that several files declare: one
    // that shows nothing of what is inside the option when the
    // reference goes on to a name that no other declares, then the one
    // that declares the most inside the option, then the first by name.
    let choose = |met: &Met, candidates: &[&PathBuf]| -> PathBuf {
        let mut ranked: Vec<&PathBuf> = candidates.to_vec();
        ranked.sort_by_key(|file| {
            (
                !(met.further && met.node.opaque.contains(*file)),
                std::cmp::Reverse(met.node.count(file)),
                (*file).clone(),
            )
        });
        ranked[0].clone()
    };

    let mut reports = BTreeMap::new();
    for (file, touches) in files {
        if crate::memory::over() {
            return Err(format!(
                "checking the references takes more memory than it may, at {}",
                file.display()
            ));
        }
        let start = || std::iter::once(file.clone());
        let (mut reached_by_hand, _) = reach(&by_hand, start(), modules.can_be_disabled);
        reached_by_hand.insert(module_system.clone());
        let (mut reached, imports_unknown) = reach(&all, start(), modules.can_be_disabled);
        reached.insert(module_system.clone());
        let mut report = Report {
            imports_unknown,
            ..Report::default()
        };
        // Each reference that an import by hand does not cover, with
        // the files that can cover it.
        let mut open: Vec<(Reference, Vec<PathBuf>, Option<PathBuf>)> = Vec::new();
        let mut not_imported: BTreeSet<Reference> = BTreeSet::new();
        let mut optional: BTreeSet<Reference> = BTreeSet::new();
        let mut refer = |reference: Reference, preferred: Option<PathBuf>| {
            let providers = &reference.declared_in;
            if providers.iter().any(|file| reached_by_hand.contains(file)) {
                return;
            }
            let candidates: Vec<PathBuf> = providers
                .iter()
                .filter(|file| always.contains(*file))
                .cloned()
                .collect();
            if candidates.is_empty() {
                if !providers.iter().any(|file| reached.contains(file)) {
                    optional.insert(reference);
                }
                return;
            }
            if !providers.iter().any(|file| reached.contains(file)) {
                not_imported.insert(reference.clone());
            }
            open.push((reference, candidates, preferred));
        };

        for option in &touches.enums {
            // The reading of the file took the value of this option to
            // be one of the values that the file declares for it. A
            // declaration in another file can add values, and what the
            // file selects by such a value is then not known.
            let mut found = Vec::new();
            declared.meet(option, false, &mut Vec::new(), None, &mut found);
            if found
                .iter()
                .any(|met| met.node.files.iter().any(|declaring| declaring != file))
            {
                report.not_known.insert(format!(
                    "read by the value of {}, which another file declares too",
                    option.join(".")
                ));
            }
        }
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
            // An argument of modules is a value that some module sets,
            // unless the caller of the modules gives it.
            if touch.kind == Kind::Read
                && let [first, second, name, ..] = path
                && first == "_module"
                && second == "args"
            {
                // An argument that several modules set is one that the
                // caller chooses a module for, as `pkgs` is set by the
                // module of nixpkgs or by its read-only one. No import
                // decides that for the caller.
                if let Some(setting) = arguments.get(name.as_str())
                    && setting.len() == 1
                    && !modules.caller_arguments.contains(name)
                {
                    refer(
                        Reference {
                            kind: touch.kind,
                            option: format!("_module.args.{name}"),
                            declared_in: setting.iter().map(|file| (*file).clone()).collect(),
                        },
                        None,
                    );
                }
                continue;
            }
            let mut found = Vec::new();
            declared.meet(path, touch.open, &mut Vec::new(), None, &mut found);
            // Options that the same files declare are one case: which
            // of them the reference names does not matter for what
            // covers it.
            let mut cases: BTreeSet<&BTreeSet<PathBuf>> = BTreeSet::new();
            found.retain(|met| cases.insert(&met.node.files));
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
                declared.meet(path, true, &mut Vec::new(), None, &mut above);
                if above.is_empty() {
                    report.not_declared.insert(shown);
                }
                continue;
            }
            for met in found {
                let providers: Vec<PathBuf> = met.node.files.iter().cloned().collect();
                let candidates: Vec<&PathBuf> = providers
                    .iter()
                    .filter(|file| always.contains(*file))
                    .collect();
                let preferred = (!candidates.is_empty()).then(|| choose(&met, &candidates));
                refer(
                    Reference {
                        kind: touch.kind,
                        option: met.option.join("."),
                        declared_in: providers,
                    },
                    preferred,
                );
            }
        }
        // A reference that one file can cover needs that file. Any
        // other reference is covered by a file that is needed already
        // when there is one, and by the preferred file otherwise.
        for (_, candidates, _) in &open {
            if let [only] = &candidates[..] {
                report.needed.insert(only.clone());
            }
        }
        for (_, candidates, preferred) in &open {
            if !candidates.iter().any(|file| report.needed.contains(file)) {
                let chosen = preferred.clone().unwrap_or_else(|| candidates[0].clone());
                report.needed.insert(chosen);
            }
        }
        report.needed.remove(file);
        report.not_imported = not_imported.into_iter().collect();
        report.optional = optional.into_iter().collect();
        reports.insert(file.clone(), report);
    }
    Ok(reports)
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

    /// Check files of which those below `/m` are loaded always. The
    /// result has, for each file, what is not covered and then the
    /// imports that are needed.
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
        let roots: BTreeSet<PathBuf> = files
            .keys()
            .filter(|file| file.starts_with("/m"))
            .cloned()
            .collect();
        check(&Modules {
            files: &files,
            roots: &roots,
            generated: &BTreeMap::new(),
            caller_arguments: &BTreeSet::from(["lib".to_string()]),
            can_be_disabled: &BTreeSet::new(),
        })
        .unwrap()
        .into_iter()
        .map(|(file, report)| {
            let mut lines: Vec<String> = report
                .not_imported
                .iter()
                .map(|reference| format!("{:?} {}", reference.kind, reference.option))
                .collect();
            lines.extend(
                report
                    .optional
                    .iter()
                    .map(|reference| format!("optional: {}", reference.option)),
            );
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
            lines.extend(
                report
                    .needed
                    .iter()
                    .map(|path| format!("needs {}", path.display())),
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
        assert_eq!(
            found["/m/a.nix"],
            ["Read users.users", "needs /m/users.nix"]
        );
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
                "not declared: set servics.a.enable",
                "needs /m/b.nix"
            ]
        );
    }

    #[test]
    fn a_reference_into_an_option_needs_the_file_that_declares_the_inner_option() {
        let found = run(&[
            (
                "/m/a.nix",
                touches(&[
                    (Kind::Set, "systemd.services.a.serviceConfig.User", false),
                    (Kind::Set, "systemd.services.a.confinement.enable", false),
                    (Kind::Set, "systemd.services.a.other", false),
                ]),
            ),
            (
                "/m/confinement.nix",
                touches(&[
                    (Kind::Declare, "systemd.services", false),
                    (
                        Kind::Declare,
                        "systemd.services.*.confinement.enable",
                        false,
                    ),
                ]),
            ),
            (
                "/m/runner.nix",
                touches(&[
                    (Kind::Declare, "systemd.services", false),
                    (Kind::Declare, "systemd.services.*.runner", false),
                ]),
            ),
            (
                "/m/systemd.nix",
                touches(&[
                    (Kind::Declare, "systemd.services", false),
                    (Kind::Declare, "systemd.services.*.serviceConfig", false),
                    (Kind::Declare, "systemd.services.*.wantedBy", false),
                ]),
            ),
        ]);
        // `other` is declared by none of them inside the option, and is
        // covered by a file that is needed already.
        assert_eq!(
            found["/m/a.nix"],
            [
                "Set systemd.services",
                "Set systemd.services.*.confinement.enable",
                "Set systemd.services.*.serviceConfig",
                "needs /m/confinement.nix",
                "needs /m/systemd.nix"
            ]
        );
    }

    #[test]
    fn a_module_outside_those_that_are_loaded_always_is_not_imported() {
        let found = run(&[
            (
                "/m/a.nix",
                touches(&[
                    (Kind::Set, "virtualisation.memorySize", false),
                    (Kind::Read, "config._module.args.pkgs", true),
                    (Kind::Read, "config._module.args.lib", true),
                    (Kind::Read, "config._module.args.utils", true),
                ]),
            ),
            (
                "/m/nixpkgs.nix",
                touches(&[
                    (Kind::Set, "_module.args.pkgs", true),
                    (Kind::Set, "_module.args.lib", true),
                ]),
            ),
            (
                "/optional/vm.nix",
                touches(&[
                    (Kind::Declare, "virtualisation.memorySize", false),
                    (Kind::Set, "boot.kernelModules", false),
                ]),
            ),
            (
                "/m/kernel.nix",
                touches(&[(Kind::Declare, "boot.kernelModules", false)]),
            ),
        ]);
        // `pkgs` is set by a module, `lib` is given by the caller, and
        // nothing is known about where `utils` comes from.
        assert_eq!(
            found["/m/a.nix"],
            [
                "Read _module.args.pkgs",
                "optional: virtualisation.memorySize",
                "needs /m/nixpkgs.nix"
            ]
        );
        assert_eq!(
            found["/optional/vm.nix"],
            ["Set boot.kernelModules", "needs /m/kernel.nix"]
        );
    }
}
