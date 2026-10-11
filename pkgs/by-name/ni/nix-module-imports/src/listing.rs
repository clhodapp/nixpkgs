//! Reading from the source of a module file which options it can set.
//!
//! No condition is looked at: a setting under `mkIf`, in a branch of an
//! `if`, or in an `optionalAttrs` counts whatever the condition is. An
//! attribute path is followed down to the first declared option, and
//! what is below that is the value of the option and is not read. Where
//! the settings are not written out, for example where a function
//! builds them, the place is recorded as one that was not read.

use std::collections::{BTreeMap, BTreeSet};

use rnix::ast::{self, HasEntry};
use rowan::ast::AstNode;

use crate::source::{OptionTree, attr_name};

/// What the source of one module file shows.
#[derive(Default)]
pub struct Listing {
    /// The options the file can set.
    pub sets: BTreeSet<String>,
    /// The settings the file can make, each as the whole path that is
    /// written in the source, into the value of the option:
    /// `systemd.services.nginx.wantedBy`. A name that is computed is
    /// written `*`. The flag says that the value is not written out,
    /// for example a call of a function, so that it can hold settings
    /// below the path.
    pub settings: BTreeSet<(String, bool)>,
    /// The paths under which the settings were not read. The empty path
    /// stands for the whole module.
    pub not_read: BTreeSet<String>,
}

/// The attributes of a module that are not settings.
const NOT_SETTINGS: [&str; 8] = [
    "options",
    "imports",
    "meta",
    "disabledModules",
    "_file",
    "key",
    "freeformType",
    "_class",
];

/// Where a path of names leads.
enum Reached<'t> {
    /// To a group of options. The settings are in the value.
    Group(&'t OptionTree),
    /// To an option, and maybe on into its value.
    Option,
    /// To nothing that is declared, or to a place that was not read.
    Nothing,
}

struct Reader<'a> {
    options: &'a OptionTree,
    /// The names that a `let` of the file gives to an expression.
    bound: BTreeMap<String, ast::Expr>,
    listing: Listing,
}

/// The name of the function of a call and its arguments. `lib.mkIf a b`
/// gives `mkIf` and the two arguments.
fn call(expr: &ast::Expr) -> Option<(String, Vec<ast::Expr>)> {
    let mut arguments = Vec::new();
    let mut head = expr.clone();
    while let ast::Expr::Apply(apply) = &head {
        arguments.push(apply.argument()?);
        head = apply.lambda()?;
    }
    arguments.reverse();
    let text = head.syntax().text().to_string();
    let name = text.rsplit('.').next()?.trim().to_string();
    Some((name, arguments))
}

impl Reader<'_> {
    fn not_read(&mut self, path: &[String]) {
        self.listing.not_read.insert(path.join("."));
    }

    /// Go down `names` from `node`.
    fn descend<'t>(
        &mut self,
        path: &mut Vec<String>,
        mut node: &'t OptionTree,
        names: &[Option<String>],
    ) -> Reached<'t> {
        for (index, name) in names.iter().enumerate() {
            let Some(name) = name else {
                // The name is computed.
                self.not_read(path);
                return Reached::Nothing;
            };
            path.push(name.clone());
            // A path that leads to no option and to no group of options
            // is not a setting of an option of this evaluation.
            let Some(child) = node.child(name) else {
                return Reached::Nothing;
            };
            node = child;
            if let Some(option) = node.option() {
                self.listing.sets.insert(option.to_string());
                // The names that follow lead into the value of the option.
                for name in &names[index + 1..] {
                    path.push(name.clone().unwrap_or_else(|| "*".to_string()));
                }
                return Reached::Option;
            }
        }
        Reached::Group(node)
    }

    fn setting(&mut self, path: &[String], written_out: bool) {
        self.listing.settings.insert((path.join("."), !written_out));
    }

    /// Read the settings inside the value of an option, at `path`.
    fn inside(&mut self, expr: Option<ast::Expr>, path: &[String], depth: usize) {
        let Some(expr) = expr else {
            return self.setting(path, false);
        };
        if depth > 24 {
            return self.setting(path, false);
        }
        match &expr {
            ast::Expr::AttrSet(set) => {
                let mut any = false;
                for entry in set.attrpath_values() {
                    any = true;
                    let Some(attrpath) = entry.attrpath() else {
                        continue;
                    };
                    let mut here = path.to_vec();
                    for attr in attrpath.attrs() {
                        here.push(attr_name(&attr).unwrap_or_else(|| "*".to_string()));
                    }
                    self.inside(entry.value(), &here, depth + 1);
                }
                for inherit in set.inherits() {
                    for attr in inherit.attrs() {
                        any = true;
                        let mut here = path.to_vec();
                        here.push(attr_name(&attr).unwrap_or_else(|| "*".to_string()));
                        self.setting(&here, false);
                    }
                }
                if !any {
                    self.setting(path, true);
                }
            }
            ast::Expr::Paren(paren) => self.inside(paren.expr(), path, depth + 1),
            ast::Expr::LetIn(let_in) => self.inside(let_in.body(), path, depth + 1),
            ast::Expr::With(with) => self.inside(with.body(), path, depth + 1),
            ast::Expr::Assert(assert) => self.inside(assert.body(), path, depth + 1),
            ast::Expr::IfElse(if_else) => {
                self.inside(if_else.body(), path, depth + 1);
                self.inside(if_else.else_body(), path, depth + 1);
            }
            ast::Expr::BinOp(op) if op.operator() == Some(ast::BinOpKind::Update) => {
                self.inside(op.lhs(), path, depth + 1);
                self.inside(op.rhs(), path, depth + 1);
            }
            ast::Expr::Ident(ident) => {
                let name = ident.syntax().text().to_string();
                if matches!(name.as_str(), "true" | "false" | "null") {
                    self.setting(path, true);
                } else {
                    match self.bound.get(&name).cloned() {
                        Some(bound) => self.inside(Some(bound), path, depth + 1),
                        None => self.setting(path, false),
                    }
                }
            }
            ast::Expr::Apply(_) => match call(&expr) {
                Some((name, arguments)) => match (name.as_str(), arguments.as_slice()) {
                    ("mkIf" | "optionalAttrs" | "mkOverride" | "mkOrder", [_, content]) => {
                        self.inside(Some(content.clone()), path, depth + 1)
                    }
                    (
                        "mkDefault"
                        | "mkForce"
                        | "mkBefore"
                        | "mkAfter"
                        | "mkVMOverride"
                        | "mkImageMediaOverride",
                        [content],
                    ) => self.inside(Some(content.clone()), path, depth + 1),
                    ("mkMerge", [ast::Expr::List(list)]) => {
                        for item in list.items() {
                            self.inside(Some(item), path, depth + 1);
                        }
                    }
                    _ => self.setting(path, false),
                },
                None => self.setting(path, false),
            },
            ast::Expr::Str(_) | ast::Expr::List(_) | ast::Expr::Literal(_) | ast::Expr::Path(_) => {
                self.setting(path, true)
            }
            _ => self.setting(path, false),
        }
    }

    fn entries(&mut self, set: &ast::AttrSet, path: &[String], node: &OptionTree, top: bool) {
        for entry in set.attrpath_values() {
            let (Some(attrpath), Some(value)) = (entry.attrpath(), entry.value()) else {
                continue;
            };
            let mut names: Vec<Option<String>> =
                attrpath.attrs().map(|attr| attr_name(&attr)).collect();
            if top {
                match names.first().and_then(|name| name.as_deref()) {
                    Some("config") => {
                        names.remove(0);
                    }
                    Some(name) if NOT_SETTINGS.contains(&name) => continue,
                    _ => {}
                }
            }
            let mut here = path.to_vec();
            match self.descend(&mut here, node, &names) {
                Reached::Group(below) => self.value(value, &here, below, 0),
                Reached::Option => self.inside(Some(value), &here, 0),
                Reached::Nothing => {}
            }
        }
        for inherit in set.inherits() {
            for attr in inherit.attrs() {
                let name = attr_name(&attr);
                if top
                    && name
                        .as_deref()
                        .is_some_and(|name| NOT_SETTINGS.contains(&name))
                {
                    continue;
                }
                let mut here = path.to_vec();
                match self.descend(&mut here, node, &[name]) {
                    // The value comes from elsewhere and is a group of
                    // options.
                    Reached::Group(_) => self.not_read(&here),
                    Reached::Option => self.setting(&here, false),
                    Reached::Nothing => {}
                }
            }
        }
    }

    /// Read the settings in an expression that stands at `path`.
    fn value(&mut self, expr: ast::Expr, path: &[String], node: &OptionTree, depth: usize) {
        if depth > 16 {
            self.not_read(path);
            return;
        }
        match &expr {
            ast::Expr::AttrSet(set) => self.entries(set, path, node, false),
            ast::Expr::Paren(paren) => self.inner(paren.expr(), path, node, depth),
            ast::Expr::LetIn(let_in) => self.inner(let_in.body(), path, node, depth),
            ast::Expr::With(with) => self.inner(with.body(), path, node, depth),
            ast::Expr::Assert(assert) => self.inner(assert.body(), path, node, depth),
            ast::Expr::IfElse(if_else) => {
                self.inner(if_else.body(), path, node, depth);
                self.inner(if_else.else_body(), path, node, depth);
            }
            ast::Expr::BinOp(op) if op.operator() == Some(ast::BinOpKind::Update) => {
                self.inner(op.lhs(), path, node, depth);
                self.inner(op.rhs(), path, node, depth);
            }
            ast::Expr::Ident(ident) => {
                let name = ident.syntax().text().to_string();
                match self.bound.get(&name).cloned() {
                    Some(bound) => self.value(bound, path, node, depth + 1),
                    None => self.not_read(path),
                }
            }
            ast::Expr::Apply(_) => match call(&expr) {
                Some((name, arguments)) => match (name.as_str(), arguments.as_slice()) {
                    ("mkIf" | "optionalAttrs" | "mkOverride" | "mkOrder", [_, content]) => {
                        self.value(content.clone(), path, node, depth + 1)
                    }
                    (
                        "mkDefault"
                        | "mkForce"
                        | "mkBefore"
                        | "mkAfter"
                        | "mkVMOverride"
                        | "mkImageMediaOverride",
                        [content],
                    ) => self.value(content.clone(), path, node, depth + 1),
                    ("mkMerge", [ast::Expr::List(list)]) => {
                        for item in list.items() {
                            self.value(item, path, node, depth + 1);
                        }
                    }
                    _ => self.not_read(path),
                },
                None => self.not_read(path),
            },
            _ => self.not_read(path),
        }
    }

    fn inner(&mut self, expr: Option<ast::Expr>, path: &[String], node: &OptionTree, depth: usize) {
        match expr {
            Some(expr) => self.value(expr, path, node, depth + 1),
            None => self.not_read(path),
        }
    }
}

/// Read the options that a module file can set. `None` when the file
/// does not parse.
pub fn read(text: &str, options: &OptionTree) -> Option<Listing> {
    let parse = rnix::Root::parse(text);
    if !parse.errors().is_empty() {
        return None;
    }
    let root = parse.tree();
    let mut bound = BTreeMap::new();
    for node in root.syntax().descendants() {
        if let Some(let_in) = ast::LetIn::cast(node) {
            for binding in let_in.attrpath_values() {
                let (Some(attrpath), Some(value)) = (binding.attrpath(), binding.value()) else {
                    continue;
                };
                let names: Vec<Option<String>> =
                    attrpath.attrs().map(|attr| attr_name(&attr)).collect();
                if let [Some(name)] = names.as_slice() {
                    bound.entry(name.clone()).or_insert(value);
                }
            }
        }
    }
    let mut reader = Reader {
        options,
        bound,
        listing: Listing::default(),
    };
    // The module, behind the function, `let`, `with` and parentheses
    // that may be written around it.
    let mut expr = root.expr();
    while let Some(current) = &expr {
        expr = match current {
            ast::Expr::Lambda(lambda) => lambda.body(),
            ast::Expr::LetIn(let_in) => let_in.body(),
            ast::Expr::With(with) => with.body(),
            ast::Expr::Assert(assert) => assert.body(),
            ast::Expr::Paren(paren) => paren.expr(),
            _ => break,
        };
    }
    let options = reader.options;
    match expr {
        Some(ast::Expr::AttrSet(set)) => reader.entries(&set, &[], options, true),
        // A module that is not written as an attribute set, such as
        // `lib.mkIf condition { ... }`, is all settings.
        Some(other) => reader.value(other, &[], options, 0),
        None => reader.not_read(&[]),
    }
    Some(reader.listing)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing(text: &str) -> (Vec<String>, Vec<String>) {
        let names: Vec<String> = [
            "services.web.enable",
            "services.web.port",
            "systemd.services",
            "environment.systemPackages",
            "networking.firewall.allowedTCPPorts",
            "users.users",
        ]
        .iter()
        .map(|name| name.to_string())
        .collect();
        let options = OptionTree::new(names.iter());
        let listing = read(text, &options).unwrap();
        (
            listing.sets.into_iter().collect(),
            listing.not_read.into_iter().collect(),
        )
    }

    #[test]
    fn reads_settings_whatever_their_condition() {
        let (sets, not_read) = listing(
            r#"{ config, lib, pkgs, ... }:
let
  cfg = config.services.web;
  firewall = { networking.firewall.allowedTCPPorts = [ cfg.port ]; };
in
{
  options.services.web.enable = lib.mkEnableOption "web";
  config = lib.mkIf cfg.enable (lib.mkMerge [
    {
      systemd.services.web = { wantedBy = [ "multi-user.target" ]; };
      environment.systemPackages = lib.optionals cfg.enable [ pkgs.hello ];
    }
    (lib.optionalAttrs pkgs.stdenv.isLinux { users.users.web = { }; })
    (if cfg.port == 80 then firewall else { })
  ]);
}
"#,
        );
        assert_eq!(
            sets,
            [
                "environment.systemPackages",
                "networking.firewall.allowedTCPPorts",
                "systemd.services",
                "users.users"
            ]
        );
        assert!(not_read.is_empty());
    }

    #[test]
    fn reads_into_the_value_of_an_option() {
        let names: Vec<String> = [
            "systemd.services",
            "users.users",
            "environment.systemPackages",
        ]
        .iter()
        .map(|name| name.to_string())
        .collect();
        let options = OptionTree::new(names.iter());
        let listing = read(
            r#"{ config, lib, pkgs, name, ... }:
let
  cfg = config.services.web;
in
{
  systemd.services.web = {
    wantedBy = [ "multi-user.target" ];
    serviceConfig.ExecStart = "web";
    path = lib.mkIf cfg.enable [ pkgs.hello ];
  };
  systemd.services.${name}.enable = true;
  users.users.web.group = cfg.group;
  environment.systemPackages = lib.attrValues cfg.packages;
}
"#,
            &options,
        )
        .unwrap();
        let settings: Vec<(String, bool)> = listing.settings.into_iter().collect();
        assert_eq!(
            settings,
            [
                ("environment.systemPackages".to_string(), true),
                ("systemd.services.*.enable".to_string(), false),
                ("systemd.services.web.path".to_string(), false),
                (
                    "systemd.services.web.serviceConfig.ExecStart".to_string(),
                    false
                ),
                ("systemd.services.web.wantedBy".to_string(), false),
                ("users.users.web.group".to_string(), true),
            ]
        );
    }

    #[test]
    fn records_where_the_settings_are_not_written_out() {
        let (sets, not_read) = listing(
            "{ lib, ... }:\n{\n  systemd = lib.mapAttrs (name: value: value) { };\n  environment.systemPackages = [ ];\n  services.${\"web\"}.port = 1;\n}\n",
        );
        assert_eq!(sets, ["environment.systemPackages"]);
        assert_eq!(not_read, ["services", "systemd"]);
    }

    #[test]
    fn reads_a_module_that_is_not_an_attribute_set() {
        let (sets, not_read) = listing(
            "{ lib, config, ... }:\nlib.mkIf config.services.web.enable {\n  users.users.web = { };\n}\n",
        );
        assert_eq!(sets, ["users.users"]);
        assert!(not_read.is_empty());
    }
}
