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

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::rc::Rc;

use serde::Serialize;

use crate::algebra::{ARGUMENTS, Attrs, Interpreter, Thunk, Value, string, tracked};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// The module sets something at the path.
    Set,
    /// The module declares an option at the path.
    Declare,
    /// The module reads the path from the configuration or the options.
    Read,
    /// The module asks for the path in a way that can do without it:
    /// `a.b or c`, `a ? b`, or a name that is looked for in a `with`.
    /// Nothing fails when the path is not there.
    Ask,
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
    ///
    /// The `_type` can be one of several values, as in
    /// `mkIf c { ... } // other`, where `other` may bring its own. Then
    /// the set is read as each kind. `None` among the kinds is a set
    /// that is no property.
    fn properties(&mut self, attrs: &Attrs) -> Vec<Option<String>> {
        let kinds = match self.field(attrs, "_type") {
            Some(Value::Either(values)) => values.as_ref().clone(),
            Some(value) => vec![value],
            None => return vec![None],
        };
        let mut found: Vec<Option<String>> = Vec::new();
        for kind in kinds {
            let kind = match kind {
                Value::Str(kind) => Some(kind.to_string()),
                _ => None,
            };
            if !found.contains(&kind) {
                found.push(kind);
            }
        }
        found
    }

    /// Read settings or declarations in a value at a path.
    fn read(&mut self, kind: Kind, value: Value, path: &mut Vec<String>, depth: usize) {
        if self.interpreter.stopped() {
            return;
        }
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
                let mut plain = false;
                for property in self.properties(&attrs) {
                    match property.as_deref() {
                        Some("if" | "override" | "order") => {
                            // The condition decides nothing, and what it
                            // reads of the configuration is still a read.
                            self.field(&attrs, "condition");
                            let content = self.field(&attrs, "content").unwrap_or(Value::Unknown);
                            self.read(kind, content, path, depth + 1);
                        }
                        Some("merge") => {
                            let contents = self.field(&attrs, "contents").unwrap_or(Value::Unknown);
                            match contents {
                                Value::List(items) | Value::Seq(items) => {
                                    for item in items.iter() {
                                        let item = self.interpreter.force(item);
                                        self.read(kind, item, path, depth + 1);
                                    }
                                }
                                other => self.read(kind, other, path, depth + 1),
                            }
                        }
                        // Any other kind is a value of its own, such as
                        // the declaration of an option.
                        Some(_) => self.touch(kind, path, false),
                        None => plain = true,
                    }
                }
                if !plain {
                    return;
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
        if self.interpreter.stopped() {
            return;
        }
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
                // The module system works out every part of a module,
                // whatever the reading below makes of it. So all that
                // the module holds is escaped: an element of a list
                // that is a setting, a function that is the default of
                // an option, the condition of a `mkIf`.
                self.interpreter.escape(&Value::Attrs(attrs.clone()));
                // A property around the whole module, as in
                // `lib.mkIf condition { ... }`, holds settings.
                let properties = self.properties(&attrs);
                if properties.iter().any(Option::is_some) {
                    self.read(Kind::Set, Value::Attrs(attrs.clone()), &mut Vec::new(), 0);
                }
                if !properties.contains(&None) {
                    return;
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
            // The module is not known: it can set and declare anything.
            Value::Unknown | Value::Config(_) => {
                self.interpreter.escape(&value);
                self.touch(Kind::Set, &[], true);
                self.touch(Kind::Declare, &[], true);
                self.touch(Kind::Import, &["*".to_string()], true);
            }
            Value::Path(path) => {
                self.touch(Kind::Import, &[path.to_string_lossy().to_string()], false)
            }
            Value::Str(text) => self.touch(Kind::Import, &[text.to_string()], false),
            // A file that holds a list of modules, as
            // `nixos/modules/module-list.nix` does, is read as a module
            // that imports them. A module in the list that is not a
            // path is written in that file and belongs to it.
            Value::List(_) | Value::Seq(_) if depth == 0 => self.imports(value, inputs, depth),
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
///
/// `special` holds the arguments that the caller of the modules gives
/// by name, as NixOS gives `modulesPath`. An argument that is not in it
/// is read from the configuration at `_module.args`, which is where the
/// module system takes `pkgs` and `utils` from.
pub fn of_file(
    interpreter: &mut Interpreter,
    file: &Path,
    library: &Path,
    special: &BTreeMap<String, String>,
) -> Touches {
    interpreter.begin();
    interpreter.library = Some(library.to_path_buf());
    interpreter.module = Some(file.to_path_buf());
    let lib = interpreter.import_file(library);
    interpreter.arguments = BTreeMap::from([
        ("config".to_string(), tracked("config")),
        ("options".to_string(), tracked("options")),
        ("lib".to_string(), lib),
    ]);
    for (name, text) in special {
        interpreter.arguments.insert(name.clone(), string(text));
    }
    let inputs = Thunk::done(Value::Config(Rc::new(
        ARGUMENTS.iter().map(|part| part.to_string()).collect(),
    )));
    let module = interpreter.import_file(file);
    let mut reader = Reader {
        interpreter,
        touches: BTreeSet::new(),
    };
    reader.module(module, &inputs, 0);
    let mut touches = reader.touches;
    interpreter.settle();
    let mut cuts: Vec<(&String, &usize)> = interpreter.cuts.iter().collect();
    cuts.sort_by_key(|(_, count)| std::cmp::Reverse(**count));
    for (index, (place, count)) in cuts.iter().enumerate() {
        if index < 25 || place.starts_with("all of") {
            eprintln!("{count} {place}");
        }
    }
    if interpreter.fatal.is_some() {
        // The interpretation stopped, so what it found is not all there
        // is. The file counts as one that can touch anything.
        touches.clear();
        interpreter.reads.clear();
        interpreter.deep_reads = BTreeSet::from([Vec::new()]);
        for kind in [Kind::Set, Kind::Declare] {
            touches.insert(Touch {
                kind,
                path: Vec::new(),
                open: true,
            });
        }
        touches.insert(Touch {
            kind: Kind::Import,
            path: vec!["*".to_string()],
            open: true,
        });
    }
    for path in &interpreter.reads {
        if !interpreter.deep_reads.contains(path) {
            touches.insert(Touch {
                kind: Kind::Read,
                path: path.clone(),
                open: false,
            });
        }
    }
    for path in &interpreter.deep_reads {
        // A value that was asked for and is then used in full is still
        // one that the module can do without, unless the module also
        // reads the path in a way that needs it.
        let asked = !interpreter.reads.contains(path)
            && interpreter
                .asked
                .iter()
                .any(|asked| path.starts_with(asked));
        touches.insert(Touch {
            kind: if asked { Kind::Ask } else { Kind::Read },
            path: path.clone(),
            open: true,
        });
    }
    for path in &interpreter.asked {
        if !interpreter.reads.contains(path) && !interpreter.deep_reads.contains(path) {
            touches.insert(Touch {
                kind: Kind::Ask,
                path: path.clone(),
                open: false,
            });
        }
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
  mapAttrsToList = f: set: map (name: f name set.${name}) (builtins.attrNames set);
  recursiveUpdate = lhs: rhs:
    let
      f = builtins.zipAttrsWith (name: values:
        if builtins.length values == 1 || !(builtins.isAttrs (builtins.elemAt values 1) && builtins.isAttrs (builtins.head values))
        then builtins.head values
        else f values);
    in
    f [ rhs lhs ];
  setAttrByPath = path: value:
    let
      go = n: if n == builtins.length path then value else { ${builtins.elemAt path n} = go (n + 1); };
    in
    go 0;
}
"#,
        )
        .unwrap();
        let file = root.join("module.nix");
        fs::write(&file, module).unwrap();
        let mut interpreter = Interpreter::new();
        let result = of_file(&mut interpreter, &file, &root.join("lib"), &BTreeMap::new());
        // Read it again and again, and see that the memory in use does
        // not grow. The first readings are let go by, because they also
        // make what the thread keeps for itself.
        let again = |interpreter: Interpreter| {
            drop(interpreter);
            crate::algebra::release();
            let mut interpreter = Interpreter::new();
            of_file(&mut interpreter, &file, &root.join("lib"), &BTreeMap::new());
            interpreter
        };
        let mut interpreter = interpreter;
        for _ in 0..5 {
            interpreter = again(interpreter);
        }
        drop(interpreter);
        crate::algebra::release();
        let before = crate::memory::in_use();
        let mut interpreter = Interpreter::new();
        for _ in 0..50 {
            interpreter = again(interpreter);
        }
        drop(interpreter);
        crate::algebra::release();
        let grown = crate::memory::in_use() - before;
        assert!(
            grown.abs() < 4096,
            "reading the module fifty times left {grown} bytes in use"
        );
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
                "Read config._module.args.pkgs",
                "Read config._module.args.pkgs.stdenv",
                "Read config._module.args.pkgs.stdenv.isLinux ...",
                "Read config.services",
                "Read config.services.web",
                "Read config.services.web.enable ...",
                "Read config.services.web.port ...",
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
                "Read config.services.web.instances.*.description ...",
            ]
        );
    }

    #[test]
    fn a_recursion_that_is_cut_stops_at_a_parent() {
        // The helper calls itself once for each name of the path. The
        // calls are followed as far as allowed, and then the setting is
        // one that can have anything below it. What the call that is
        // cut would have done with the value is not lost: the value is
        // read with everything below it.
        assert_eq!(
            touches(
                "recursion",
                r#"{ config, lib, ... }:
{
  config = lib.setAttrByPath [ "a" "b" "c" "d" "e" "f" "g" "h" "i" "j" "k" "l" ] config.x.y;
}
"#
            ),
            [
                "Set a.b.c.d.e.f.g.h.i ...",
                "Set a.b.c.d.e.f.g.h.i.* ...",
                "Read config.x",
                "Read config.x.y ..."
            ]
        );
    }

    #[test]
    fn sets_that_are_written_out_are_put_together_name_by_name() {
        // `recursiveUpdate` of the library is written with
        // `zipAttrsWith` and asks how many sets have a name. With sets
        // that are written out that is known, and the declarations are
        // the two that are written.
        assert_eq!(
            touches(
                "zip",
                r#"{ lib, ... }:
{
  options = builtins.foldl' lib.recursiveUpdate { } [
    { services.old.address = lib.mkOption { }; }
    { services.old.port = lib.mkOption { }; }
  ];
}
"#
            ),
            ["Declare services.old.address", "Declare services.old.port"]
        );
    }

    #[test]
    fn a_set_that_is_not_known_can_replace_a_name_that_is_written() {
        // The name of the setting comes from `recursiveUpdate` of a set
        // that is written out and a part of the configuration. The
        // configuration can replace `store.path`, so the name is not
        // only the one that is written.
        let found = touches(
            "replace",
            r#"{ config, lib, ... }:
let
  merged = lib.recursiveUpdate { store.path = "/written"; } config.program.settings;
in
{
  files.${merged.store.path}.mode = "0700";
}
"#,
        );
        assert!(
            found.iter().any(|touch| touch.starts_with("Set files.*")),
            "{found:?}"
        );
        assert!(
            found
                .iter()
                .any(|touch| touch.starts_with("Read config.program.settings.store.path")),
            "{found:?}"
        );
    }

    #[test]
    fn a_path_that_is_split_from_a_text_is_known() {
        // The names come from a text that is written out and is split
        // at the dots, with the separator escaped like the library
        // does it.
        assert_eq!(
            touches(
                "split",
                r#"{ lib, ... }:
let
  escaped = builtins.replaceStrings [ "." "*" ] [ "\\." "\\*" ] ".";
  path = builtins.filter builtins.isString (builtins.split escaped "services.x.old");
in
{
  config = lib.setAttrByPath path true;
}
"#
            ),
            ["Set services.x.old"]
        );
    }

    #[test]
    fn a_helper_of_the_module_that_calls_itself_reads_what_it_is_given() {
        // The helper walks a part of the configuration and calls itself
        // for each set inside it. It is written in the module, so it
        // can see `config`. Its recursion ends with what it was given
        // read in full, and not with all of the configuration. It takes
        // its arguments one at a time, so each level makes the function
        // for the second argument anew.
        let found = touches(
            "walk",
            r#"{ config, lib, ... }:
let
  walk = prefix: set: builtins.concatLists (lib.mapAttrsToList (name: value:
    if builtins.isAttrs value then walk (prefix + name + ".") value
    else [ { name = prefix + name; inherit value; } ]) set);
in
{
  environment.etc.x.text = builtins.toJSON (walk "" config.program.settings);
}
"#,
        );
        assert!(
            found.contains(&"Read config.program.settings.* ...".to_string()),
            "{found:?}"
        );
        assert!(!found.contains(&"Read config ...".to_string()), "{found:?}");
    }

    #[test]
    fn a_helper_that_carries_a_path_along_reads_what_it_is_given() {
        // Like the helper above, but what it carries from level to
        // level is the list of the names so far, which is one name
        // longer each time and never the same twice.
        let found = touches(
            "path",
            r#"{ config, lib, ... }:
let
  walk = path: value:
    if builtins.isAttrs value
    then builtins.concatLists (lib.mapAttrsToList (name: walk (path ++ [ name ])) value)
    else [ { inherit path value; } ];
in
{
  environment.etc.x.text = builtins.toJSON (walk [ ] config.program.settings);
}
"#,
        );
        assert!(
            found.contains(&"Read config.program.settings ...".to_string())
                || found.contains(&"Read config.program.settings.* ...".to_string()),
            "{found:?}"
        );
        assert!(!found.contains(&"Read config ...".to_string()), "{found:?}");
    }

    #[test]
    fn inherit_in_a_rec_set_takes_the_name_from_around_the_set() {
        // `inherit config;` in a `rec` set is the `config` of the scope
        // around the set, and not the entry that it makes.
        assert_eq!(
            touches(
                "inherit",
                r#"{ config, ... }:
let
  inner = rec {
    inherit config;
    value = config.one.two;
  };
in
{
  config.first = inner.value;
  config.second = inner.config.three.four;
}
"#
            ),
            [
                "Set first ...",
                "Set second ...",
                "Read config.one",
                "Read config.one.two ...",
                "Read config.three",
                "Read config.three.four ...",
            ]
        );
    }

    #[test]
    fn what_leaves_the_interpretation_is_read_in_full() {
        // A value that goes to a function that is not known, into a
        // text or a comparison, or into a list that is a setting, is
        // read with everything below it. An argument of the module that
        // the caller does not give by name is read from `_module.args`.
        assert_eq!(
            touches(
                "leaves",
                r#"{ config, pkgs, utils, ... }:
assert config.a.ok;
{
  environment.systemPackages = [ config.b.package ];
  systemd.services.x.script = pkgs.writeText "x" (builtins.toJSON config.c);
  boot.kernelModules = builtins.filter (name: name != config.d.skip) [ "kvm" ];
  users.groups = utils.groups config.e;
}
"#
            ),
            [
                "Set boot.kernelModules",
                "Set environment.systemPackages",
                "Set systemd.services.x.script ...",
                "Set users.groups ...",
                "Read config._module.args.pkgs",
                "Read config._module.args.pkgs.writeText ...",
                "Read config._module.args.utils",
                "Read config._module.args.utils.groups ...",
                "Read config.a",
                "Read config.a.ok ...",
                "Read config.b",
                "Read config.b.package ...",
                "Read config.c ...",
                "Read config.d",
                "Read config.d.skip ...",
                "Read config.e ...",
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
            [
                "Set meta.maintainers",
                "Set warnings",
                "Read config.old",
                "Read config.old.enable ..."
            ]
        );
    }
}
