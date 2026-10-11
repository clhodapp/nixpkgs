//! Working out, from what the module system reports and from the source
//! of the modules, which imports each module file needs.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::reduce::{reaches, reduce};
use crate::source::{Module, OptionTree, Unsupported, relative_literal};

/// The output of `lib.modules.moduleDependencies`. File names are the
/// ones the evaluation saw.
#[derive(Deserialize, Default)]
pub struct Dependencies {
    pub uses: Vec<Use>,
    pub imports: BTreeMap<String, Vec<String>>,
    pub importable: Vec<String>,
    pub declarations: BTreeMap<String, Declaration>,
}

#[derive(Deserialize)]
pub struct Use {
    pub option: String,
    pub file: String,
}

#[derive(Deserialize, Default)]
pub struct Declaration {
    pub files: Vec<String>,
    #[serde(rename = "withDescription")]
    pub with_description: Vec<String>,
}

impl Dependencies {
    /// Put the dependencies of another evaluation with these. One
    /// evaluation shows only the modules that are in it, so a tree whose
    /// modules are spread over several evaluations needs all of them.
    pub fn merge(&mut self, other: Dependencies) {
        self.uses.extend(other.uses);
        for (file, imports) in other.imports {
            let known = self.imports.entry(file).or_default();
            for import in imports {
                if !known.contains(&import) {
                    known.push(import);
                }
            }
        }
        for file in other.importable {
            if !self.importable.contains(&file) {
                self.importable.push(file);
            }
        }
        for (option, declaration) in other.declarations {
            let known = self.declarations.entry(option).or_default();
            for file in declaration.files {
                if !known.files.contains(&file) {
                    known.files.push(file);
                }
            }
            for file in declaration.with_description {
                if !known.with_description.contains(&file) {
                    known.with_description.push(file);
                }
            }
        }
    }

    /// The file that a module imports to use an option. With several
    /// declaring files it is the one whose declaration has the
    /// description, because the other declarations add to that one.
    fn declaring_file(&self, option: &str) -> Option<&str> {
        let declaration = self.declarations.get(option)?;
        match (
            declaration.files.as_slice(),
            declaration.with_description.as_slice(),
        ) {
            ([only], _) => Some(only),
            (_, [described]) => Some(described),
            _ => None,
        }
    }
}

/// How an option is used by a module.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// The module sets the option. The module system reports this.
    Definition,
    /// The module reads the option. The source of the module shows this.
    Read,
}

/// One import that a module needs.
#[derive(Serialize)]
pub struct Import {
    /// The imported file, relative to the root of the tree.
    pub file: String,
    /// The path as it is written in the importing file.
    pub literal: String,
    /// The options that the module uses of the imported file.
    pub options: BTreeMap<String, Kind>,
}

/// A use of an option that the tool writes no import for.
#[derive(Serialize)]
pub struct Unwritten {
    /// The module that uses the option, as the evaluation names it.
    pub module: String,
    pub option: String,
    /// The file that declares the option, as the evaluation names it.
    /// Empty when no declaring file could be chosen.
    pub declared_in: String,
}

#[derive(Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// The generated imports in the file are the needed ones.
    Current,
    /// The tool can write the needed imports.
    Stale,
    /// The file is not in a shape the tool rewrites, and the module does
    /// not import all it needs.
    Unsupported,
}

#[derive(Serialize)]
pub struct FilePlan {
    /// The module, relative to the root, as the evaluation names it. For
    /// a module that is a directory, this is the directory.
    pub module: String,
    /// The file that holds the source of the module.
    pub source: String,
    pub status: Status,
    /// Why the file cannot be rewritten.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The imports the tool writes into the file.
    pub generated: Vec<Import>,
    #[serde(skip)]
    pub new_text: Option<String>,
}

#[derive(Serialize)]
pub struct Cycles {
    /// The number of groups of modules that reach each other through
    /// imports, once the needed imports are written.
    pub groups: usize,
    /// The number of modules in the largest group.
    pub largest: usize,
    /// The number of groups for which the search for the smallest set
    /// of imports did not finish in the time it was given.
    pub not_searched: usize,
    /// The imports inside the groups that the tool writes.
    pub imports: usize,
    /// No set of imports inside the groups is smaller than this. When
    /// it equals `imports`, the tool writes the smallest set there is.
    pub lower_bound: usize,
}

#[derive(Serialize)]
pub struct Plan {
    pub modules: usize,
    pub modules_in_tree: usize,
    pub definitions: usize,
    pub reads: usize,
    /// The number of imports of files that a module uses and does not
    /// import in its own writing. The tool writes fewer, because it
    /// leaves out a file that a module reaches through another import.
    pub direct: usize,
    pub files: Vec<FilePlan>,
    /// Uses of an option that is declared outside the tree.
    pub declared_outside: Vec<Unwritten>,
    /// Uses of an option whose declaring file is not imported by path.
    pub not_importable: Vec<Unwritten>,
    /// Uses of an option that several files declare, none or several of
    /// them with a description.
    pub no_declaration_to_choose: Vec<Unwritten>,
    pub cycles: Cycles,
}

pub struct Settings<'a> {
    /// The directory that holds the tree to rewrite.
    pub root: &'a Path,
    /// The name of that directory in the evaluation.
    pub evaluated_root: &'a str,
    /// Leave out the options that a module only reads.
    pub definitions_only: bool,
    /// How long the search for the smallest set of imports may take.
    pub search: Duration,
}

impl Settings<'_> {
    /// The path below the root for a file name of the evaluation.
    fn in_tree(&self, evaluated: &str) -> Option<String> {
        let rest = evaluated.strip_prefix(self.evaluated_root)?;
        let rest = rest.strip_prefix('/')?;
        (!rest.is_empty()).then(|| rest.to_string())
    }
}

/// One module of the tree, before the tool decides what to write.
struct Candidate {
    module: String,
    source: String,
    directory: String,
    parsed: Result<Module, String>,
    /// The files the module needs and does not import in its own
    /// writing, with the options it uses of each.
    needed: BTreeMap<String, BTreeMap<String, Kind>>,
    /// The imports the module has whatever the tool writes.
    fixed: BTreeSet<String>,
    /// Why the tool cannot write imports into the file.
    unwritable: Option<String>,
}

/// The directory part of a path below the root.
fn directory_of(path: &str) -> &str {
    path.rfind('/').map_or("", |index| &path[..index])
}

pub fn plan(dependencies: &Dependencies, settings: &Settings) -> Plan {
    let importable: BTreeSet<&str> = dependencies.importable.iter().map(String::as_str).collect();
    let option_tree = OptionTree::new(dependencies.declarations.keys());

    // What each module defines of other modules.
    let mut defined: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for used in &dependencies.uses {
        defined.entry(&used.file).or_default().insert(&used.option);
    }
    let modules: BTreeSet<&str> = dependencies
        .imports
        .keys()
        .map(String::as_str)
        .chain(defined.keys().copied())
        .collect();

    let mut plan = Plan {
        modules: modules.len(),
        modules_in_tree: 0,
        definitions: dependencies.uses.len(),
        reads: 0,
        direct: 0,
        files: Vec::new(),
        declared_outside: Vec::new(),
        not_importable: Vec::new(),
        no_declaration_to_choose: Vec::new(),
        cycles: Cycles {
            groups: 0,
            largest: 0,
            not_searched: 0,
            imports: 0,
            lower_bound: 0,
        },
    };
    // First every module is read, to learn what it needs and which
    // imports it has whatever the tool writes.
    let mut candidates: Vec<Candidate> = Vec::new();

    for evaluated in modules {
        let Some(module) = settings.in_tree(evaluated) else {
            continue;
        };
        plan.modules_in_tree += 1;
        let on_disk = settings.root.join(&module);
        let (source, directory) = if on_disk.is_dir() {
            (format!("{module}/default.nix"), module.clone())
        } else {
            (module.clone(), directory_of(&module).to_string())
        };
        let parsed = fs::read_to_string(settings.root.join(&source))
            .map_err(|error| format!("the file cannot be read: {error}"))
            .and_then(|text| Module::parse(text).map_err(|reason| reason.to_string()));

        let mut used: BTreeMap<String, Kind> = defined
            .get(evaluated)
            .into_iter()
            .flatten()
            .map(|option| (option.to_string(), Kind::Definition))
            .collect();
        if let (Ok(parsed), false) = (&parsed, settings.definitions_only) {
            for option in parsed.reads(&option_tree) {
                let declares_it =
                    dependencies
                        .declarations
                        .get(&option)
                        .is_some_and(|declaration| {
                            declaration.files.iter().any(|file| file == evaluated)
                        });
                if !declares_it && !used.contains_key(&option) {
                    used.insert(option, Kind::Read);
                    plan.reads += 1;
                }
            }
        }

        // The files the module needs, with the options it uses of each.
        let mut needed: BTreeMap<String, BTreeMap<String, Kind>> = BTreeMap::new();
        for (option, kind) in used {
            let unwritten = |declared_in: &str| Unwritten {
                module: evaluated.to_string(),
                option: option.clone(),
                declared_in: declared_in.to_string(),
            };
            let Some(declaring) = dependencies.declaring_file(&option) else {
                plan.no_declaration_to_choose.push(unwritten(""));
                continue;
            };
            if declaring == evaluated {
                continue;
            }
            match settings.in_tree(declaring) {
                None => plan.declared_outside.push(unwritten(declaring)),
                Some(_) if !importable.contains(declaring) => {
                    plan.not_importable.push(unwritten(declaring))
                }
                Some(file) => {
                    needed.entry(file).or_default().insert(option, kind);
                }
            }
        }

        let manual = match &parsed {
            Ok(parsed) => parsed.manual_imports(&directory).unwrap_or_default(),
            Err(_) => BTreeSet::new(),
        };
        needed.retain(|file, _| !manual.contains(file));
        plan.direct += needed.len();

        // Whether the tool can write imports into the file is found by
        // trying it with all of them.
        let literals: Vec<String> = needed
            .keys()
            .map(|file| relative_literal(&directory, file))
            .collect();
        let unwritable = match &parsed {
            Err(reason) => Some(reason.clone()),
            Ok(parsed) => match parsed.with_generated_imports(&literals) {
                Err(reason) if !literals.is_empty() => Some(reason.to_string()),
                _ => None,
            },
        };
        let fixed = if unwritable.is_some() {
            // A person writes the imports of such a file by hand, in
            // whatever way the file needs. The evaluation shows them.
            dependencies
                .imports
                .get(evaluated)
                .into_iter()
                .flatten()
                .filter_map(|file| settings.in_tree(file))
                .collect()
        } else {
            manual
        };
        candidates.push(Candidate {
            module,
            source,
            directory,
            parsed,
            needed,
            fixed,
            unwritable,
        });
    }

    // The imports between the modules of the tree: the ones that stay
    // whatever the tool writes, and at first every import that a module
    // needs.
    let mut names: BTreeMap<String, usize> = BTreeMap::new();
    let mut edges: Vec<BTreeSet<usize>> = Vec::new();
    let mut node = |name: &str, edges: &mut Vec<BTreeSet<usize>>| match names.get(name) {
        Some(index) => *index,
        None => {
            names.insert(name.to_string(), edges.len());
            edges.push(BTreeSet::new());
            edges.len() - 1
        }
    };
    for candidate in &candidates {
        let from = node(&candidate.module, &mut edges);
        for file in &candidate.fixed {
            let to = node(file, &mut edges);
            edges[from].insert(to);
        }
        if candidate.unwritable.is_none() {
            for file in candidate.needed.keys() {
                let to = node(file, &mut edges);
                edges[from].insert(to);
            }
        }
    }

    // An import is left out when the module reaches the file through
    // its other imports.
    let removable: BTreeSet<(usize, usize)> = candidates
        .iter()
        .filter(|candidate| candidate.unwritable.is_none())
        .flat_map(|candidate| {
            let from = names[&candidate.module];
            candidate
                .needed
                .keys()
                .filter(|file| !candidate.fixed.contains(*file))
                .map(|file| (from, names[file]))
                .collect::<Vec<_>>()
        })
        .collect();
    let groups = reduce(&mut edges, &removable, Instant::now() + settings.search);
    plan.cycles = Cycles {
        groups: groups.len(),
        not_searched: groups.iter().filter(|group| !group.searched).count(),
        largest: groups.iter().map(|group| group.modules).max().unwrap_or(0),
        imports: groups.iter().map(|group| group.kept).sum(),
        lower_bound: groups.iter().map(|group| group.lower_bound).sum(),
    };

    for candidate in candidates {
        let from = names[&candidate.module];
        let Candidate {
            module,
            source,
            directory,
            parsed,
            needed,
            unwritable,
            ..
        } = candidate;
        let mut generated: Vec<Import> = needed
            .into_iter()
            .filter(|(file, _)| match unwritable {
                // What the tool writes.
                None => edges[from].contains(&names[file]),
                // What a person still has to write.
                Some(_) => !reaches(&edges, from, names[file]),
            })
            .map(|(file, options)| Import {
                literal: relative_literal(&directory, &file),
                file,
                options,
            })
            .collect();
        generated.sort_by(|left, right| left.literal.cmp(&right.literal));
        let literals: Vec<String> = generated
            .iter()
            .map(|import| import.literal.clone())
            .collect();

        let (status, reason, new_text) = match (unwritable, parsed) {
            (Some(_), _) if generated.is_empty() => (Status::Current, None, None),
            (Some(reason), _) => (Status::Unsupported, Some(reason), None),
            (None, Err(reason)) => (Status::Unsupported, Some(reason), None),
            (None, Ok(parsed)) => match parsed.with_generated_imports(&literals) {
                // A file in a shape the tool does not rewrite is left
                // alone when it needs nothing written.
                Err(
                    Unsupported::BodyIsNotAnAttributeSet
                    | Unsupported::ImportsIsNotAList
                    | Unsupported::ImportsIsInherited,
                ) if literals.is_empty() => (Status::Current, None, None),
                Err(reason) => (Status::Unsupported, Some(reason.to_string()), None),
                Ok(text) => {
                    let current = fs::read_to_string(settings.root.join(&source)).ok();
                    if Some(&text) == current.as_ref() {
                        (Status::Current, None, None)
                    } else {
                        (Status::Stale, None, Some(text))
                    }
                }
            },
        };
        if status == Status::Current && generated.is_empty() {
            continue;
        }
        plan.files.push(FilePlan {
            module,
            source,
            status,
            reason,
            generated,
            new_text,
        });
    }

    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan for a tree of modules `a.nix`, `b.nix` and `c.nix`, each of
    /// which declares the option of its name. `uses` says which module
    /// sets the option of which other module.
    fn plan_for(name: &str, uses: &[(&str, &str)]) -> Vec<(String, Vec<String>)> {
        let root = std::env::temp_dir().join(format!(
            "nix-module-imports-test-{}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let evaluated_root = root.to_str().unwrap().to_string();
        let mut dependencies = Dependencies::default();
        for module in ["a", "b", "c"] {
            fs::write(root.join(format!("{module}.nix")), "{\n  x = 1;\n}\n").unwrap();
            let file = format!("{evaluated_root}/{module}.nix");
            dependencies.imports.insert(file.clone(), Vec::new());
            dependencies.importable.push(file.clone());
            dependencies.declarations.insert(
                module.to_string(),
                Declaration {
                    files: vec![file.clone()],
                    with_description: vec![file],
                },
            );
        }
        for (module, option) in uses {
            dependencies.uses.push(Use {
                option: option.to_string(),
                file: format!("{evaluated_root}/{module}.nix"),
            });
        }
        let plan = plan(
            &dependencies,
            &Settings {
                root: &root,
                evaluated_root: &evaluated_root,
                definitions_only: true,
                search: Duration::from_secs(60),
            },
        );
        fs::remove_dir_all(&root).unwrap();
        plan.files
            .into_iter()
            .map(|file| {
                (
                    file.module,
                    file.generated
                        .into_iter()
                        .map(|import| import.literal)
                        .collect(),
                )
            })
            .collect()
    }

    #[test]
    fn leaves_out_a_file_that_another_import_reaches() {
        let files = plan_for("chain", &[("a", "b"), ("a", "c"), ("b", "c")]);
        assert_eq!(
            files,
            [
                ("a.nix".to_string(), vec!["./b.nix".to_string()]),
                ("b.nix".to_string(), vec!["./c.nix".to_string()]),
            ]
        );
    }

    #[test]
    fn keeps_a_file_in_reach_of_modules_that_import_each_other() {
        // `a` and `b` use each other and both use `c`. Each could leave
        // `c` to the other, and one of them has to keep it.
        let files = plan_for("cycle", &[("a", "b"), ("a", "c"), ("b", "a"), ("b", "c")]);
        assert_eq!(
            files,
            [
                (
                    "a.nix".to_string(),
                    vec!["./b.nix".to_string(), "./c.nix".to_string()]
                ),
                ("b.nix".to_string(), vec!["./a.nix".to_string()]),
            ]
        );
    }

    #[test]
    fn chooses_the_declaration_with_the_description() {
        let dependencies: Dependencies = serde_json::from_str(
            r#"{
                "uses": [],
                "imports": {},
                "importable": [],
                "declarations": {
                    "one": { "files": ["/t/a.nix"], "withDescription": [] },
                    "two": { "files": ["/t/a.nix", "/t/b.nix"], "withDescription": ["/t/b.nix"] },
                    "three": { "files": ["/t/a.nix", "/t/b.nix"], "withDescription": [] }
                }
            }"#,
        )
        .unwrap();
        assert_eq!(dependencies.declaring_file("one"), Some("/t/a.nix"));
        assert_eq!(dependencies.declaring_file("two"), Some("/t/b.nix"));
        assert_eq!(dependencies.declaring_file("three"), None);
    }
}
