//! What a module file touches, by the interpretation in `algebra`.
//!
//! A module is a value of the language: a set, or a function that gives
//! one. The function is applied to its inputs, of which the
//! configuration and the options are tracked and the rest is not known.
//! The result is read as the module system defines a module: `options`
//! holds declarations, `config` holds settings, and a module without
//! both is all settings. A value that `mkIf`, `mkMerge`, `mkOverride` or
//! `mkOrder` made is read as what it holds, so `mkIf` is nothing but its
//! content.

use std::collections::BTreeSet;
use std::path::Path;

use serde::Serialize;

use crate::algebra::{Attrs, Interpreter, Thunk, Value, attrs, string, tracked};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// The module sets something at the path.
    Set,
    /// The module declares an option at the path.
    Declare,
    /// The module reads the path from the configuration or the options.
    Read,
    /// The module imports the file at the path.
    Import,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Touch {
    pub kind: Kind,
    /// The path, with `*` for a name that is not known.
    pub path: Vec<String>,
    /// Whether more can be touched below the path than is listed,
    /// because the value there is not known.
    pub open: bool,
}

#[derive(Serialize, Default)]
pub struct Touches {
    pub touches: BTreeSet<Touch>,
    /// How often a call went too deep to follow.
    pub too_deep: usize,
    /// How often a value was needed while it was being worked out.
    pub cycles: usize,
    /// What stopped the interpretation, when something did.
    pub stopped: Option<String>,
    /// Errors in the module that this reading came across, such as a
    /// builtin that does not exist.
    pub problems: BTreeSet<String>,
}

/// How deep the settings of a module are followed.
const DEPTH: usize = 64;

struct Reader<'a> {
    interpreter: &'a mut Interpreter,
    touches: BTreeSet<Touch>,
}

impl Reader<'_> {
    fn touch(&mut self, kind: Kind, path: &[String], open: bool) {
        self.touches.insert(Touch {
            kind,
            path: path.to_vec(),
            open,
        });
    }

    fn field(&mut self, attrs: &Attrs, name: &str) -> Option<Value> {
        let field = attrs.fields.get(name).cloned()?;
        Some(self.interpreter.force(&field))
    }

    /// The kind of property of the module system that a set is, when it
    /// is one: the value of its `_type`.
    fn property(&mut self, attrs: &Attrs) -> Option<String> {
        match self.field(attrs, "_type")? {
            Value::Str(kind) => Some(kind.to_string()),
            _ => None,
        }
    }

    /// Read settings or declarations in a value at a path.
    fn read(&mut self, kind: Kind, value: Value, path: &mut Vec<String>, depth: usize) {
        if depth > DEPTH {
            return self.touch(kind, path, true);
        }
        match value {
            Value::Either(values) => {
                for value in values.iter().cloned() {
                    self.read(kind, value, path, depth + 1);
                }
            }
            Value::Attrs(attrs) => {
                match self.property(&attrs).as_deref() {
                    Some("if" | "override" | "order") => {
                        // The condition decides nothing, and what it
                        // reads of the configuration is still a read.
                        self.field(&attrs, "condition");
                        let content = self.field(&attrs, "content").unwrap_or(Value::Unknown);
                        return self.read(kind, content, path, depth + 1);
                    }
                    Some("merge") => {
                        let contents = self.field(&attrs, "contents").unwrap_or(Value::Unknown);
                        return match contents {
                            Value::List(items) | Value::Seq(items) => {
                                for item in items.iter() {
                                    let item = self.interpreter.force(item);
                                    self.read(kind, item, path, depth + 1);
                                }
                            }
                            other => self.read(kind, other, path, depth + 1),
                        };
                    }
                    // Any other kind is a value of its own, such as the
                    // declaration of an option.
                    Some(_) => return self.touch(kind, path, false),
                    None => {}
                }
                if attrs.fields.is_empty() && attrs.any.is_none() {
                    return self.touch(kind, path, false);
                }
                for (name, field) in &attrs.fields {
                    let field = self.interpreter.force(field);
                    path.push(name.clone());
                    self.read(kind, field, path, depth + 1);
                    path.pop();
                }
                if let Some(any) = &attrs.any {
                    let any = self.interpreter.force(any);
                    path.push("*".to_string());
                    self.read(kind, any, path, depth + 1);
                    path.pop();
                }
            }
            // What is here is not known, so anything can be below.
            Value::Unknown | Value::Config(_) => self.touch(kind, path, true),
            Value::Str(_)
            | Value::Path(_)
            | Value::List(_)
            | Value::Seq(_)
            | Value::Int(_)
            | Value::Bool(_)
            | Value::Null
            | Value::Lambda(_)
            | Value::Builtin(..) => self.touch(kind, path, false),
        }
    }

    /// Read a module: a set, or a function that is applied to the inputs.
    fn module(&mut self, value: Value, inputs: &Thunk, depth: usize) {
        if depth > 8 {
            return self.touch(Kind::Set, &[], true);
        }
        match value {
            Value::Lambda(_) | Value::Builtin(..) => {
                let result = self.interpreter.apply(value, inputs.clone());
                self.module(result, inputs, depth + 1);
            }
            Value::Either(values) => {
                for value in values.iter().cloned() {
                    self.module(value, inputs, depth + 1);
                }
            }
            Value::Attrs(attrs) => {
                // A property around the whole module, as in
                // `lib.mkIf condition { ... }`, holds settings.
                if self.property(&attrs).is_some() {
                    return self.read(Kind::Set, Value::Attrs(attrs), &mut Vec::new(), 0);
                }
                let explicit =
                    attrs.fields.contains_key("config") || attrs.fields.contains_key("options");
                for (name, field) in &attrs.fields {
                    let field = self.interpreter.force(field);
                    match name.as_str() {
                        "imports" => self.imports(field, inputs, depth),
                        "options" => self.read(Kind::Declare, field, &mut Vec::new(), 0),
                        "config" => self.read(Kind::Set, field, &mut Vec::new(), 0),
                        // The module system puts these into the settings.
                        "meta" => self.read(Kind::Set, field, &mut vec!["meta".to_string()], 0),
                        "freeformType" => self.touch(
                            Kind::Set,
                            &["_module".to_string(), "freeformType".to_string()],
                            false,
                        ),
                        // The parts of a module that are not settings.
                        "_file" | "key" | "_class" | "disabledModules" => {}
                        // A module with `options` or `config` has no
                        // other settings. One without has only settings.
                        _ if explicit => {}
                        _ => self.read(Kind::Set, field, &mut vec![name.clone()], 0),
                    }
                }
                if let Some(any) = &attrs.any {
                    let any = self.interpreter.force(any);
                    self.read(Kind::Set, any, &mut vec!["*".to_string()], 0);
                }
            }
            // The module is not known: it can set anything.
            Value::Unknown | Value::Config(_) => self.touch(Kind::Set, &[], true),
            Value::Path(path) => {
                self.touch(Kind::Import, &[path.to_string_lossy().to_string()], false)
            }
            Value::Str(text) => self.touch(Kind::Import, &[text.to_string()], false),
            // Not a module.
            Value::List(_) | Value::Seq(_) | Value::Int(_) | Value::Bool(_) | Value::Null => {}
        }
    }

    /// The `imports` of a module. A path is another file. Anything else
    /// is a module that is written in this file.
    fn imports(&mut self, value: Value, inputs: &Thunk, depth: usize) {
        match value {
            Value::List(items) | Value::Seq(items) => {
                for item in items.iter() {
                    let item = self.interpreter.force(item);
                    self.module(item, inputs, depth + 1);
                }
            }
            Value::Either(values) => {
                for value in values.iter().cloned() {
                    self.imports(value, inputs, depth);
                }
            }
            // The list is not known, so a module in it can set anything.
            _ => self.touch(Kind::Import, &["*".to_string()], true),
        }
    }
}

/// What the module in `file` touches. `library` is the directory of the
/// library that modules get as `lib`.
pub fn of_file(interpreter: &mut Interpreter, file: &Path, library: &Path) -> Touches {
    interpreter.reads.clear();
    interpreter.too_deep = 0;
    interpreter.cycles = 0;
    let lib = interpreter.import_file(library);
    // The inputs of a module. The configuration and the options are
    // tracked. Every other input is one that only the caller of the
    // modules has, such as `pkgs`, and is not known.
    let inputs = Thunk::done(attrs(
        vec![
            ("config", tracked("config")),
            ("options", tracked("options")),
            ("lib", lib.clone()),
            ("pkgs", attrs(vec![("lib", lib)], true)),
            (
                "modulesPath",
                string(
                    &file
                        .parent()
                        .map(|parent| parent.to_string_lossy().to_string())
                        .unwrap_or_default(),
                ),
            ),
        ],
        true,
    ));
    let module = interpreter.import_file(file);
    let mut reader = Reader {
        interpreter,
        touches: BTreeSet::new(),
    };
    reader.module(module, &inputs, 0);
    let mut touches = reader.touches;
    for path in &interpreter.reads {
        touches.insert(Touch {
            kind: Kind::Read,
            path: path.clone(),
            open: false,
        });
    }
    Touches {
        touches,
        too_deep: interpreter.too_deep,
        cycles: interpreter.cycles,
        stopped: interpreter.fatal.clone(),
        problems: std::mem::take(&mut interpreter.problems),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Interpret a module with a small library that is written like the
    /// real one, and give the touches as text.
    fn touches(name: &str, module: &str) -> Vec<String> {
        let root = std::env::temp_dir().join(format!(
            "nix-module-imports-algebra-{}-{name}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("lib")).unwrap();
        fs::write(
            root.join("lib/default.nix"),
            r#"rec {
  mkIf = condition: content: { _type = "if"; inherit condition content; };
  mkMerge = contents: { _type = "merge"; inherit contents; };
  mkOverride = priority: content: { _type = "override"; inherit priority content; };
  mkDefault = mkOverride 1000;
  mkOption = attrs: attrs // { _type = "option"; };
  optionalAttrs = condition: set: if condition then set else { };
  optionals = condition: list: if condition then list else [ ];
  mapAttrs' = f: set: builtins.listToAttrs (map (name: f name set.${name}) (builtins.attrNames set));
  nameValuePair = name: value: { inherit name value; };
  genAttrs = names: f: builtins.listToAttrs (map (name: nameValuePair name (f name)) names);
}
"#,
        )
        .unwrap();
        let file = root.join("module.nix");
        fs::write(&file, module).unwrap();
        let mut interpreter = Interpreter::new();
        let result = of_file(&mut interpreter, &file, &root.join("lib"));
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(result.stopped, None);
        result
            .touches
            .into_iter()
            .map(|touch| {
                format!(
                    "{:?} {}{}",
                    touch.kind,
                    touch.path.join("."),
                    if touch.open { " ..." } else { "" }
                )
            })
            .collect()
    }

    #[test]
    fn a_condition_decides_nothing() {
        assert_eq!(
            touches(
                "conditions",
                r#"{ config, lib, pkgs, ... }:
let
  cfg = config.services.web;
in
{
  options.services.web.enable = lib.mkOption { default = false; };
  config = lib.mkIf cfg.enable (lib.mkMerge [
    { systemd.services.web.wantedBy = [ "multi-user.target" ]; }
    (lib.optionalAttrs pkgs.stdenv.isLinux { users.users.web.group = "web"; })
    (if cfg.port == 80 then { networking.firewall.allowedTCPPorts = [ 80 ]; } else { })
  ]);
}
"#
            ),
            [
                "Set ",
                "Set networking.firewall.allowedTCPPorts",
                "Set systemd.services.web.wantedBy",
                "Set users.users.web.group",
                "Declare services.web.enable",
                "Read config.services",
                "Read config.services.web",
                "Read config.services.web.enable",
                "Read config.services.web.port",
            ]
        );
    }

    #[test]
    fn a_helper_is_read_from_its_source() {
        // Nothing here knows `mapAttrs'` or `genAttrs` by name. The
        // entries that are made from the configuration have a name that
        // is not known, and the ones from a list that is written out
        // have their names.
        assert_eq!(
            touches(
                "helpers",
                r#"{ config, lib, ... }:
{
  systemd.services = lib.mapAttrs' (name: instance:
    lib.nameValuePair "web-${name}" { description = instance.description; }
  ) config.services.web.instances;
  users.users = lib.genAttrs [ "alice" "bob" ] (name: { isNormalUser = true; });
}
"#
            ),
            [
                "Set systemd.services.*.description ...",
                "Set users.users.alice.isNormalUser",
                "Set users.users.bob.isNormalUser",
                "Read config.services",
                "Read config.services.web",
                "Read config.services.web.instances",
                "Read config.services.web.instances.*",
                "Read config.services.web.instances.*.description",
            ]
        );
    }

    #[test]
    fn a_module_inside_imports_belongs_to_the_file() {
        assert_eq!(
            touches(
                "imports",
                "{ lib, ... }:\n{\n  imports = [\n    ./other.nix\n    { meta.maintainers = [ ]; }\n    ({ config, ... }: { warnings = lib.optionals config.old.enable [ \"old\" ]; })\n  ];\n}\n"
            )
            .into_iter()
            .filter(|touch| !touch.starts_with("Import"))
            .collect::<Vec<_>>(),
            ["Set meta.maintainers", "Set warnings", "Read config.old", "Read config.old.enable"]
        );
    }
}
