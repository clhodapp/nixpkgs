//! Working out, from what the module system reports and from the source
//! of the modules, which imports each module file needs.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

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
}

#[derive(Serialize)]
pub struct Plan {
    pub modules: usize,
    pub modules_in_tree: usize,
    pub definitions: usize,
    pub reads: usize,
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
}

impl Settings<'_> {
    /// The path below the root for a file name of the evaluation.
    fn in_tree(&self, evaluated: &str) -> Option<String> {
        let rest = evaluated.strip_prefix(self.evaluated_root)?;
        let rest = rest.strip_prefix('/')?;
        (!rest.is_empty()).then(|| rest.to_string())
    }
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
        files: Vec::new(),
        declared_outside: Vec::new(),
        not_importable: Vec::new(),
        no_declaration_to_choose: Vec::new(),
        cycles: Cycles {
            groups: 0,
            largest: 0,
        },
    };
    // The imports between the modules of the tree, with the ones the
    // tool writes.
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

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

        let edges = graph.entry(module.clone()).or_default();
        for imported in dependencies.imports.get(evaluated).into_iter().flatten() {
            if let Some(imported) = settings.in_tree(imported) {
                edges.insert(imported);
            }
        }
        edges.extend(needed.keys().cloned());

        let manual = match &parsed {
            Ok(parsed) => parsed.manual_imports(&directory).unwrap_or_default(),
            Err(_) => BTreeSet::new(),
        };
        let mut generated: Vec<Import> = needed
            .into_iter()
            .filter(|(file, _)| !manual.contains(file))
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

        let rewritten = parsed.and_then(|parsed| {
            let old = parsed.with_generated_imports(&literals);
            match old {
                // A file in a shape the tool does not rewrite is left
                // alone when it needs nothing written.
                Err(
                    Unsupported::BodyIsNotAnAttributeSet
                    | Unsupported::ImportsIsNotAList
                    | Unsupported::ImportsIsInherited,
                ) if literals.is_empty() => Ok(None),
                Err(reason) => Err(reason.to_string()),
                Ok(text) => Ok(Some(text)),
            }
        });
        let current = fs::read_to_string(settings.root.join(&source)).ok();
        let (status, reason, new_text) = match rewritten {
            Ok(None) => (Status::Current, None, None),
            Ok(Some(text)) if Some(&text) == current.as_ref() => (Status::Current, None, None),
            Ok(Some(text)) => (Status::Stale, None, Some(text)),
            Err(reason) => {
                // A person can write the imports of such a file by
                // hand, in whatever way the file needs. The evaluation
                // then shows them.
                let imported: BTreeSet<String> = dependencies
                    .imports
                    .get(evaluated)
                    .into_iter()
                    .flatten()
                    .filter_map(|file| settings.in_tree(file))
                    .collect();
                if generated
                    .iter()
                    .all(|import| imported.contains(&import.file))
                {
                    (Status::Current, None, None)
                } else {
                    (Status::Unsupported, Some(reason), None)
                }
            }
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

    plan.cycles = cycles(&graph);
    plan
}

/// Count the groups of modules that reach each other, with the
/// algorithm of Tarjan for strongly connected components.
fn cycles(graph: &BTreeMap<String, BTreeSet<String>>) -> Cycles {
    struct State<'a> {
        graph: &'a BTreeMap<String, BTreeSet<String>>,
        index: BTreeMap<&'a str, usize>,
        low: BTreeMap<&'a str, usize>,
        on_stack: BTreeSet<&'a str>,
        stack: Vec<&'a str>,
        next: usize,
        result: Cycles,
    }
    // The walk keeps its own stack, because a chain of imports can be
    // as long as the tree has modules.
    type Work<'a> = Vec<(&'a str, Vec<&'a str>)>;
    fn enter<'a>(state: &mut State<'a>, work: &mut Work<'a>, node: &'a str) {
        state.index.insert(node, state.next);
        state.low.insert(node, state.next);
        state.next += 1;
        state.stack.push(node);
        state.on_stack.insert(node);
        let graph = state.graph;
        let successors: Vec<&'a str> = graph
            .get(node)
            .into_iter()
            .flatten()
            .map(String::as_str)
            .filter(|successor| graph.contains_key(*successor))
            .collect();
        work.push((node, successors));
    }
    fn visit<'a>(state: &mut State<'a>, root: &'a str) {
        let mut work: Work<'a> = Vec::new();
        enter(state, &mut work, root);
        while let Some((node, successors)) = work.last_mut() {
            let node = *node;
            if let Some(successor) = successors.pop() {
                if !state.index.contains_key(successor) {
                    enter(state, &mut work, successor);
                } else if state.on_stack.contains(successor) {
                    let low = state.low[node].min(state.index[successor]);
                    state.low.insert(node, low);
                }
                continue;
            }
            work.pop();
            if let Some((parent, _)) = work.last() {
                let low = state.low[*parent].min(state.low[node]);
                state.low.insert(*parent, low);
            }
            if state.low[node] == state.index[node] {
                let mut size = 0;
                while let Some(member) = state.stack.pop() {
                    state.on_stack.remove(member);
                    size += 1;
                    if member == node {
                        break;
                    }
                }
                if size > 1 {
                    state.result.groups += 1;
                    state.result.largest = state.result.largest.max(size);
                }
            }
        }
    }
    let mut state = State {
        graph,
        index: BTreeMap::new(),
        low: BTreeMap::new(),
        on_stack: BTreeSet::new(),
        stack: Vec::new(),
        next: 0,
        result: Cycles {
            groups: 0,
            largest: 0,
        },
    };
    for node in graph.keys() {
        if !state.index.contains_key(node.as_str()) {
            visit(&mut state, node);
        }
    }
    state.result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(edges: &[(&str, &[&str])]) -> BTreeMap<String, BTreeSet<String>> {
        edges
            .iter()
            .map(|(from, to)| {
                (
                    from.to_string(),
                    to.iter().map(|to| to.to_string()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn counts_the_groups_of_modules_that_reach_each_other() {
        let found = cycles(&graph(&[
            ("a", &["b"]),
            ("b", &["c"]),
            ("c", &["a", "d"]),
            ("d", &["e"]),
            ("e", &["d"]),
            ("f", &["a"]),
        ]));
        assert_eq!(found.groups, 2);
        assert_eq!(found.largest, 3);
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
