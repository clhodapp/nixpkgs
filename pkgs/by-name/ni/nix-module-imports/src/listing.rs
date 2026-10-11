//! Reading from the source of a module file what the file writes.
//!
//! This is the first of two steps, and it decides nothing. It needs the
//! source of one file and nothing else: no evaluation, no list of the
//! options that exist. It gives one finding for every attribute path
//! that the module writes, whatever the path is. Whether a finding
//! counts, and for what, is decided afterwards, where every finding that
//! is set aside gets a reason.
//!
//! No condition is looked at: what is written under `mkIf`, in a branch
//! of an `if`, or in an `optionalAttrs` is found whatever the condition
//! is. A path is followed as far as it is written out, so a finding
//! names `systemd.services.nginx.wantedBy` and not only
//! `systemd.services`.

use std::collections::{BTreeMap, BTreeSet};

use rnix::ast::{self, HasEntry};
use rowan::ast::AstNode;
use serde::Serialize;

use crate::source::attr_name;

/// Where in the module a path is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Place {
    /// Under `options`.
    Options,
    /// Under `config`.
    Config,
    /// At the top of the module, under neither. In a module without
    /// `options` and `config` these are its settings. The attributes
    /// that are parts of a module, such as `imports`, are found here as
    /// well, and are told apart afterwards.
    Top,
}

/// One attribute path that a module writes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Finding {
    pub place: Place,
    /// The path, with `*` for a name that is computed.
    pub path: Vec<String>,
    /// Whether the value at the path is written out. It is not when a
    /// function builds it or when it comes from elsewhere, and then the
    /// value can hold more below the path than the source shows here.
    pub written_out: bool,
}

struct Reader {
    /// The names that a `let` of the file gives to an expression.
    bound: BTreeMap<String, ast::Expr>,
    findings: BTreeSet<Finding>,
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

fn name_of(attr: &ast::Attr) -> String {
    attr_name(attr).unwrap_or_else(|| "*".to_string())
}

impl Reader {
    fn found(&mut self, place: Place, path: &[String], written_out: bool) {
        self.findings.insert(Finding {
            place,
            path: path.to_vec(),
            written_out,
        });
    }

    /// Read what is written in the value at `path`.
    fn value(&mut self, expr: Option<ast::Expr>, place: Place, path: &[String], depth: usize) {
        let Some(expr) = expr else {
            return self.found(place, path, false);
        };
        if depth > 24 {
            return self.found(place, path, false);
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
                    here.extend(attrpath.attrs().map(|attr| name_of(&attr)));
                    self.value(entry.value(), place, &here, depth + 1);
                }
                for inherit in set.inherits() {
                    for attr in inherit.attrs() {
                        any = true;
                        let mut here = path.to_vec();
                        here.push(name_of(&attr));
                        self.found(place, &here, false);
                    }
                }
                if !any {
                    self.found(place, path, true);
                }
            }
            ast::Expr::Paren(paren) => self.value(paren.expr(), place, path, depth + 1),
            ast::Expr::LetIn(let_in) => self.value(let_in.body(), place, path, depth + 1),
            ast::Expr::With(with) => self.value(with.body(), place, path, depth + 1),
            ast::Expr::Assert(assert) => self.value(assert.body(), place, path, depth + 1),
            ast::Expr::IfElse(if_else) => {
                self.value(if_else.body(), place, path, depth + 1);
                self.value(if_else.else_body(), place, path, depth + 1);
            }
            ast::Expr::BinOp(op) if op.operator() == Some(ast::BinOpKind::Update) => {
                self.value(op.lhs(), place, path, depth + 1);
                self.value(op.rhs(), place, path, depth + 1);
            }
            ast::Expr::Ident(ident) => {
                let name = ident.syntax().text().to_string();
                if matches!(name.as_str(), "true" | "false" | "null") {
                    self.found(place, path, true);
                } else {
                    match self.bound.get(&name).cloned() {
                        Some(bound) => self.value(Some(bound), place, path, depth + 1),
                        None => self.found(place, path, false),
                    }
                }
            }
            ast::Expr::Apply(_) => match call(&expr) {
                Some((name, arguments)) => match (name.as_str(), arguments.as_slice()) {
                    ("mkIf" | "optionalAttrs" | "mkOverride" | "mkOrder", [_, content]) => {
                        self.value(Some(content.clone()), place, path, depth + 1)
                    }
                    (
                        "mkDefault"
                        | "mkForce"
                        | "mkBefore"
                        | "mkAfter"
                        | "mkVMOverride"
                        | "mkImageMediaOverride",
                        [content],
                    ) => self.value(Some(content.clone()), place, path, depth + 1),
                    ("mkMerge", [ast::Expr::List(list)]) => {
                        for item in list.items() {
                            self.value(Some(item), place, path, depth + 1);
                        }
                    }
                    _ => self.found(place, path, false),
                },
                None => self.found(place, path, false),
            },
            ast::Expr::Str(_) | ast::Expr::List(_) | ast::Expr::Literal(_) | ast::Expr::Path(_) => {
                self.found(place, path, true)
            }
            _ => self.found(place, path, false),
        }
    }

    /// Read the attribute set that is the module. What is under
    /// `options` and under `config` is found with that place and without
    /// the first name. Everything else is found as it is written.
    fn module(&mut self, set: &ast::AttrSet) {
        for entry in set.attrpath_values() {
            let Some(attrpath) = entry.attrpath() else {
                continue;
            };
            let names: Vec<String> = attrpath.attrs().map(|attr| name_of(&attr)).collect();
            let (place, path) = match names.first().map(String::as_str) {
                Some("options") => (Place::Options, &names[1..]),
                Some("config") => (Place::Config, &names[1..]),
                _ => (Place::Top, &names[..]),
            };
            self.value(entry.value(), place, path, 0);
        }
        for inherit in set.inherits() {
            for attr in inherit.attrs() {
                let name = name_of(&attr);
                match name.as_str() {
                    "options" => self.found(Place::Options, &[], false),
                    "config" => self.found(Place::Config, &[], false),
                    _ => self.found(Place::Top, &[name], false),
                }
            }
        }
    }
}

/// Read what a module file writes. `None` when the file does not parse.
pub fn read(text: &str) -> Option<BTreeSet<Finding>> {
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
        bound,
        findings: BTreeSet::new(),
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
    match expr {
        Some(ast::Expr::AttrSet(set)) => reader.module(&set),
        // A module that is not written as an attribute set, such as
        // `lib.mkIf condition { ... }`.
        other => reader.value(other, Place::Top, &[], 0),
    }
    Some(reader.findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The findings as text: the place, the path, and `?` after a path
    /// whose value is not written out.
    fn findings(text: &str) -> Vec<String> {
        read(text)
            .unwrap()
            .into_iter()
            .map(|finding| {
                format!(
                    "{:?} {}{}",
                    finding.place,
                    finding.path.join("."),
                    if finding.written_out { "" } else { "?" }
                )
            })
            .collect()
    }

    #[test]
    fn finds_what_is_written_whatever_its_condition() {
        assert_eq!(
            findings(
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
"#
            ),
            [
                "Options services.web.enable?",
                "Config ",
                "Config environment.systemPackages?",
                "Config networking.firewall.allowedTCPPorts",
                "Config systemd.services.web.wantedBy",
                "Config users.users.web",
            ]
        );
    }

    #[test]
    fn finds_every_attribute_of_the_module() {
        // Nothing is left out here. `imports` and `meta` are found like
        // any other path, and what they are is decided afterwards.
        assert_eq!(
            findings(
                "{ lib, name, ... }:\n{\n  imports = [ ./a.nix ];\n  meta.maintainers = [ ];\n  systemd.services.${name}.enable = true;\n  users = lib.mapAttrs (n: v: v) { };\n}\n"
            ),
            [
                "Top imports",
                "Top meta.maintainers",
                "Top systemd.services.*.enable",
                "Top users?",
            ]
        );
    }

    #[test]
    fn reads_a_module_that_is_not_an_attribute_set() {
        assert_eq!(
            findings(
                "{ lib, config, ... }:\nlib.mkIf config.services.web.enable {\n  users.users.web.group = config.services.web.group;\n}\n"
            ),
            ["Top users.users.web.group?"]
        );
    }
}
