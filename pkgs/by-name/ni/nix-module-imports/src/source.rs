//! Reading and rewriting the source of one module file.
//!
//! Everything here works on the syntax of the file. A file that is not
//! written in one of the shapes this module knows is reported with a
//! reason, and is left as it is.

use std::collections::BTreeSet;
use std::fmt;

use rnix::ast::{self, HasEntry};
use rnix::{NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken};
use rowan::ast::AstNode;

/// The comment that starts the imports this tool writes.
pub const BEGIN: &str = "# BEGIN generated imports";
/// The rest of the first comment, for the reader of the module.
const BEGIN_REST: &str = ": nix-module-imports writes the lines up to END";
/// The comment that ends the imports this tool writes.
pub const END: &str = "# END generated imports";

/// Why a file cannot be read or rewritten.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Unsupported {
    DoesNotParse,
    BodyIsNotAnAttributeSet,
    ImportsIsInherited,
    ImportsIsNotAList,
    DividerCommentsAreIncomplete,
    Layout(&'static str),
}

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unsupported::DoesNotParse => write!(f, "the file does not parse"),
            Unsupported::BodyIsNotAnAttributeSet => write!(
                f,
                "the module is not written as an attribute set, or as a function, `let` or `with` around one"
            ),
            Unsupported::ImportsIsInherited => write!(f, "`imports` is set with `inherit`"),
            Unsupported::ImportsIsNotAList => {
                write!(f, "the value of `imports` has no list written in brackets")
            }
            Unsupported::DividerCommentsAreIncomplete => write!(
                f,
                "one of the two comments around the generated imports is missing"
            ),
            Unsupported::Layout(what) => write!(f, "{what}"),
        }
    }
}

/// A parsed module file.
pub struct Module {
    text: String,
    body: ast::AttrSet,
}

/// The `imports` list of a module, as far as it is written in the file.
struct Imports {
    binding: ast::AttrpathValue,
    /// The list the generated imports go into. `None` when the value
    /// has no list written in brackets, for example when a function
    /// computes it.
    list: Option<ast::List>,
    begin: Option<SyntaxToken>,
    end: Option<SyntaxToken>,
}

fn start(node: &SyntaxNode) -> usize {
    node.text_range().start().into()
}

fn end_of(node: &SyntaxNode) -> usize {
    node.text_range().end().into()
}

fn token_start(token: &SyntaxToken) -> usize {
    token.text_range().start().into()
}

fn token_end(token: &SyntaxToken) -> usize {
    token.text_range().end().into()
}

/// The attribute set that is the module, behind the function, `let`,
/// `with`, `assert` and parentheses that may be written around it.
fn module_body(expr: ast::Expr) -> Result<ast::AttrSet, Unsupported> {
    let mut expr = Some(expr);
    while let Some(current) = expr {
        expr = match current {
            ast::Expr::AttrSet(set) => return Ok(set),
            ast::Expr::Lambda(lambda) => lambda.body(),
            ast::Expr::LetIn(let_in) => let_in.body(),
            ast::Expr::With(with) => with.body(),
            ast::Expr::Assert(assert) => assert.body(),
            ast::Expr::Paren(paren) => paren.expr(),
            _ => return Err(Unsupported::BodyIsNotAnAttributeSet),
        };
    }
    Err(Unsupported::BodyIsNotAnAttributeSet)
}

/// The name of an attribute when it is written as a plain name or as a
/// string without `${}`.
pub(crate) fn attr_name(attr: &ast::Attr) -> Option<String> {
    match attr {
        ast::Attr::Ident(ident) => Some(ident.syntax().text().to_string()),
        ast::Attr::Str(string) => {
            let text = string.syntax().text().to_string();
            let inner = text.strip_prefix('"')?.strip_suffix('"')?;
            if inner.contains("${") || inner.contains('\\') {
                None
            } else {
                Some(inner.to_string())
            }
        }
        ast::Attr::Dynamic(_) => None,
    }
}

/// The list in the value of `imports` that the generated imports go
/// into: the value itself, the last list of a chain of `++`, or such a
/// list behind a `let` or a `with`.
fn find_list(expr: ast::Expr) -> Option<ast::List> {
    match expr {
        ast::Expr::List(list) => Some(list),
        ast::Expr::Paren(paren) => find_list(paren.expr()?),
        ast::Expr::LetIn(let_in) => find_list(let_in.body()?),
        ast::Expr::With(with) => find_list(with.body()?),
        ast::Expr::BinOp(op) if op.operator() == Some(ast::BinOpKind::Concat) => op
            .rhs()
            .and_then(find_list)
            .or_else(|| op.lhs().and_then(find_list)),
        _ => None,
    }
}

/// The whitespace at the start of the line that `offset` is on, when
/// nothing but whitespace is between the start of the line and `offset`.
fn indent_before(text: &str, offset: usize) -> Option<&str> {
    let line_start = text[..offset].rfind('\n').map_or(0, |index| index + 1);
    let before = &text[line_start..offset];
    before
        .chars()
        .all(|character| character == ' ' || character == '\t')
        .then_some(before)
}

/// The whitespace at the start of the line that `offset` is on.
fn line_indent(text: &str, offset: usize) -> &str {
    let line_start = text[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = &text[line_start..];
    let length = line
        .find(|character| character != ' ' && character != '\t')
        .unwrap_or(line.len());
    &line[..length]
}

/// The lines of the generated imports, each with `indent` before it.
fn block(indent: &str, paths: &[String]) -> String {
    let mut lines = vec![format!("{indent}{BEGIN}{BEGIN_REST}")];
    lines.extend(paths.iter().map(|path| format!("{indent}{path}")));
    lines.push(format!("{indent}{END}"));
    lines.join("\n")
}

/// Join a directory and a relative path literal such as `../a/b.nix`.
/// Both are relative to the root of the tree. The result is `None` when
/// the path leaves the tree.
fn resolve(directory: &str, literal: &str) -> Option<String> {
    let mut parts: Vec<&str> = directory
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    for part in literal.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name),
        }
    }
    Some(parts.join("/"))
}

/// The path literal that reaches `target` from `directory`. Both are
/// relative to the root of the tree.
pub fn relative_literal(directory: &str, target: &str) -> String {
    let from: Vec<&str> = directory
        .split('/')
        .filter(|part| !part.is_empty())
        .collect();
    let to: Vec<&str> = target.split('/').filter(|part| !part.is_empty()).collect();
    let shared = from
        .iter()
        .zip(to.iter())
        .take_while(|(left, right)| left == right)
        .count();
    let up = from.len() - shared;
    let mut parts: Vec<&str> = if up == 0 { vec!["."] } else { vec![".."; up] };
    parts.extend(&to[shared..]);
    if shared == to.len() {
        // The target is the directory itself or one above it. A path
        // literal needs a slash, so `.` becomes `./.`.
        parts.push(".");
    }
    parts.join("/")
}

/// Whether the text of a file is written as a module: a function whose
/// arguments end in `...`, or a set that has one of the names of a
/// module. A file of helpers is neither.
pub fn looks_like_module(text: &str) -> bool {
    let parse = rnix::Root::parse(text);
    if !parse.errors().is_empty() {
        return false;
    }
    let mut expr = parse.tree().expr();
    let mut function = false;
    while let Some(current) = expr {
        expr = match current {
            ast::Expr::Lambda(lambda) => {
                match lambda.param() {
                    Some(ast::Param::Pattern(pattern)) if pattern.ellipsis_token().is_some() => {
                        function = true;
                    }
                    _ => return false,
                }
                lambda.body()
            }
            ast::Expr::LetIn(let_in) => let_in.body(),
            ast::Expr::With(with) => with.body(),
            ast::Expr::Assert(assert) => assert.body(),
            ast::Expr::Paren(paren) => paren.expr(),
            ast::Expr::AttrSet(set) => {
                return function
                    || set.attrpath_values().any(|binding| {
                        let first = binding
                            .attrpath()
                            .and_then(|attrpath| attrpath.attrs().next())
                            .and_then(|attr| attr_name(&attr));
                        matches!(
                            first.as_deref(),
                            Some("imports" | "options" | "config" | "meta")
                        )
                    });
            }
            _ => return function,
        };
    }
    false
}

impl Module {
    /// Parse the text of a module file.
    pub fn parse(text: String) -> Result<Module, Unsupported> {
        let parse = rnix::Root::parse(&text);
        if !parse.errors().is_empty() {
            return Err(Unsupported::DoesNotParse);
        }
        let root = parse.tree();
        let expr = root.expr().ok_or(Unsupported::DoesNotParse)?;
        let body = module_body(expr)?;
        Ok(Module { text, body })
    }

    fn imports(&self) -> Result<Option<Imports>, Unsupported> {
        for inherit in self.body.inherits() {
            if inherit
                .attrs()
                .any(|attr| attr_name(&attr).as_deref() == Some("imports"))
            {
                return Err(Unsupported::ImportsIsInherited);
            }
        }
        for binding in self.body.attrpath_values() {
            let Some(attrpath) = binding.attrpath() else {
                continue;
            };
            let names: Vec<Option<String>> =
                attrpath.attrs().map(|attr| attr_name(&attr)).collect();
            if names.len() != 1 || names[0].as_deref() != Some("imports") {
                continue;
            }
            let value = binding.value().ok_or(Unsupported::ImportsIsNotAList)?;
            let list = find_list(value);
            let comment = |marker: &str| {
                list.as_ref()?
                    .syntax()
                    .children_with_tokens()
                    .filter_map(NodeOrToken::into_token)
                    .find(|token| {
                        token.kind() == SyntaxKind::TOKEN_COMMENT
                            && token.text().starts_with(marker)
                    })
            };
            let begin = comment(BEGIN);
            let end = comment(END);
            if begin.is_some() != end.is_some() {
                return Err(Unsupported::DividerCommentsAreIncomplete);
            }
            if let (Some(begin), Some(end)) = (&begin, &end)
                && token_start(begin) > token_start(end)
            {
                return Err(Unsupported::DividerCommentsAreIncomplete);
            }
            return Ok(Some(Imports {
                binding,
                list,
                begin,
                end,
            }));
        }
        Ok(None)
    }

    /// The generated imports that the file has: every path between the
    /// two comments. The paths are relative to the root of the tree, and
    /// `directory` is the directory of this file.
    pub fn generated_imports(&self, directory: &str) -> Result<BTreeSet<String>, Unsupported> {
        let Some(imports) = self.imports()? else {
            return Ok(BTreeSet::new());
        };
        let (Some(begin), Some(end), Some(list)) = (&imports.begin, &imports.end, &imports.list)
        else {
            return Ok(BTreeSet::new());
        };
        let (from, to) = (token_start(begin), token_end(end));
        let mut generated = BTreeSet::new();
        for node in list.syntax().children() {
            if node.kind() != SyntaxKind::NODE_PATH || start(&node) < from || end_of(&node) > to {
                continue;
            }
            if let Some(path) = resolve(directory, &node.text().to_string()) {
                generated.insert(path);
            }
        }
        Ok(generated)
    }

    /// The text of the file with `paths` as its generated imports.
    /// `paths` are path literals, in the order they are written.
    pub fn with_generated_imports(&self, paths: &[String]) -> Result<String, Unsupported> {
        let text = &self.text;
        match self.imports()? {
            Some(imports) => match (&imports.list, &imports.begin, &imports.end) {
                (Some(list), Some(begin), Some(end)) => {
                    self.replace_block(&imports, list, begin, end, paths)
                }
                _ if paths.is_empty() => Ok(text.clone()),
                (Some(list), _, _) => self.add_block(list, paths),
                (None, _, _) => self.append_list(&imports.binding, paths),
            },
            None if paths.is_empty() => Ok(text.clone()),
            None => self.add_binding(paths),
        }
    }

    fn replace_block(
        &self,
        imports: &Imports,
        list: &ast::List,
        begin: &SyntaxToken,
        end: &SyntaxToken,
        paths: &[String],
    ) -> Result<String, Unsupported> {
        let text = &self.text;
        let from = token_start(begin);
        let to = token_end(end);
        let indent = indent_before(text, from).ok_or(Unsupported::Layout(
            "the comment that starts the generated imports is not at the start of a line",
        ))?;
        if !paths.is_empty() {
            return Ok(format!(
                "{}{}{}",
                &text[..from - indent.len()],
                block(indent, paths),
                &text[to..]
            ));
        }
        // Nothing is needed any more. Is anything else in the list?
        let list = list.syntax();
        let other_content = list.children_with_tokens().any(|child| match child {
            NodeOrToken::Node(node) => start(&node) < from || start(&node) > to,
            NodeOrToken::Token(token) => {
                token.kind() == SyntaxKind::TOKEN_COMMENT
                    && (token_start(&token) < from || token_start(&token) > to)
            }
        });
        let value_is_the_list = imports
            .binding
            .value()
            .is_some_and(|value| value.syntax() == list);
        if !other_content && value_is_the_list {
            // The tool wrote the whole `imports`, so the whole of it goes.
            let binding = imports.binding.syntax();
            let binding_start = start(binding);
            let line_start = indent_before(text, binding_start)
                .map_or(binding_start, |indent| binding_start - indent.len());
            let mut after = end_of(binding);
            if text[after..].starts_with('\n') {
                after += 1;
                if text[after..].starts_with('\n') && text[..line_start].ends_with("{\n") {
                    after += 1;
                }
            }
            return Ok(format!("{}{}", &text[..line_start], &text[after..]));
        }
        if !other_content {
            return Ok(format!(
                "{}[ ]{}",
                &text[..start(list)],
                &text[end_of(list)..]
            ));
        }
        // Take the generated lines out, with the line break before them.
        let before = text[..from].trim_end_matches([' ', '\t']);
        let before = before.strip_suffix('\n').unwrap_or(before);
        Ok(format!("{}{}", before, &text[to..]))
    }

    fn add_block(&self, list: &ast::List, paths: &[String]) -> Result<String, Unsupported> {
        let text = &self.text;
        let node = list.syntax();
        let close = list.r_brack_token().ok_or(Unsupported::DoesNotParse)?;
        let close_at = token_start(&close);
        if node.text().to_string().contains('\n') {
            let indent = indent_before(text, close_at).ok_or(Unsupported::Layout(
                "the bracket that closes the `imports` list is not at the start of a line",
            ))?;
            let before = text[..close_at - indent.len()].trim_end_matches(['\n', ' ', '\t']);
            let inner = format!("{indent}  ");
            return Ok(format!(
                "{}\n{}\n{}{}",
                before,
                block(&inner, paths),
                indent,
                &text[close_at..]
            ));
        }
        // The list is on one line. It is written out again with one
        // item to a line.
        let has_comment = node
            .children_with_tokens()
            .filter_map(NodeOrToken::into_token)
            .any(|token| token.kind() == SyntaxKind::TOKEN_COMMENT);
        if has_comment {
            return Err(Unsupported::Layout(
                "the `imports` list is on one line and has a comment in it",
            ));
        }
        let indent = line_indent(text, start(node));
        let inner = format!("{indent}  ");
        let mut lines = vec!["[".to_string()];
        lines.extend(
            list.items()
                .map(|item| format!("{inner}{}", item.syntax().text())),
        );
        lines.push(block(&inner, paths));
        lines.push(format!("{indent}]"));
        Ok(format!(
            "{}{}{}",
            &text[..start(node)],
            lines.join("\n"),
            &text[end_of(node)..]
        ))
    }

    /// Put `++ [ ... ]` with the generated imports after a value of
    /// `imports` that has no list written in brackets.
    fn append_list(
        &self,
        binding: &ast::AttrpathValue,
        paths: &[String],
    ) -> Result<String, Unsupported> {
        let text = &self.text;
        let value = binding.value().ok_or(Unsupported::ImportsIsNotAList)?;
        // `++` binds less tightly than a function call and a
        // selection, and more tightly than everything else that can be
        // written here, such as `if` and `let`.
        let binds_tightly = matches!(
            value,
            ast::Expr::Apply(_) | ast::Expr::Select(_) | ast::Expr::Ident(_) | ast::Expr::Paren(_)
        );
        let node = value.syntax();
        let indent = format!("{}  ", line_indent(text, start(binding.syntax())));
        let inner = format!("{indent}  ");
        let (open, close) = if binds_tightly { ("", "") } else { ("(", ")") };
        Ok(format!(
            "{}{open}{}{close}\n{indent}++ [\n{}\n{indent}]{}",
            &text[..start(node)],
            node.text(),
            block(&inner, paths),
            &text[end_of(node)..]
        ))
    }

    fn add_binding(&self, paths: &[String]) -> Result<String, Unsupported> {
        let text = &self.text;
        let open = self.body.l_curly_token().ok_or(Unsupported::DoesNotParse)?;
        let close = self.body.r_curly_token().ok_or(Unsupported::DoesNotParse)?;
        let after_open = token_end(&open);
        let binding = |indent: &str| {
            let inner = format!("{indent}  ");
            format!("{indent}imports = [\n{}\n{indent}];", block(&inner, paths))
        };
        match self.body.entries().next() {
            Some(first) => {
                let indent = indent_before(text, start(first.syntax()))
                    .filter(|_| text[after_open..start(first.syntax())].contains('\n'))
                    .ok_or(Unsupported::Layout(
                        "the first attribute of the module is on the line of the opening brace",
                    ))?;
                // A blank line goes between `imports` and what follows,
                // unless one is there already.
                let rest = &text[after_open..];
                let blank_line_follows = rest
                    .trim_start_matches([' ', '\t'])
                    .strip_prefix('\n')
                    .is_some_and(|rest| rest.trim_start_matches([' ', '\t']).starts_with('\n'));
                let separator = if blank_line_follows { "" } else { "\n" };
                Ok(format!(
                    "{}\n{}{}{}",
                    &text[..after_open],
                    binding(indent),
                    separator,
                    rest
                ))
            }
            None => {
                let close_at = token_start(&close);
                if !text[after_open..close_at].trim().is_empty() {
                    return Err(Unsupported::Layout(
                        "the module is an empty attribute set with a comment in it",
                    ));
                }
                let indent = line_indent(text, token_start(&open));
                let inner = format!("{indent}  ");
                Ok(format!(
                    "{}\n{}\n{}{}",
                    &text[..after_open],
                    binding(&inner),
                    indent,
                    &text[close_at..]
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewrite(text: &str, paths: &[&str]) -> Result<String, Unsupported> {
        let paths: Vec<String> = paths.iter().map(|path| path.to_string()).collect();
        Module::parse(text.to_string())?.with_generated_imports(&paths)
    }

    /// Writing the imports and then taking them away gives the first text.
    fn round_trip(text: &str, expected: &str) {
        let written = rewrite(text, &["./a.nix", "./b.nix"]).unwrap();
        assert_eq!(written, expected);
        assert_eq!(rewrite(&written, &["./a.nix", "./b.nix"]).unwrap(), written);
        assert_eq!(rewrite(&written, &[]).unwrap(), text);
    }

    #[test]
    fn adds_imports_to_a_module_without_them() {
        round_trip(
            "{ lib, ... }:\n{\n  options.a = lib.mkOption { };\n}\n",
            "{ lib, ... }:\n{\n  imports = [\n    # BEGIN generated imports: nix-module-imports writes the lines up to END\n    ./a.nix\n    ./b.nix\n    # END generated imports\n  ];\n\n  options.a = lib.mkOption { };\n}\n",
        );
    }

    #[test]
    fn adds_imports_behind_let_and_with() {
        let expected = "{ lib, config, ... }:\nwith lib;\nlet\n  cfg = config.a;\nin\n{\n  imports = [\n    # BEGIN generated imports: nix-module-imports writes the lines up to END\n    ./a.nix\n    ./b.nix\n    # END generated imports\n  ];\n\n  config.b = cfg;\n}\n";
        round_trip(
            "{ lib, config, ... }:\nwith lib;\nlet\n  cfg = config.a;\nin\n{\n  config.b = cfg;\n}\n",
            expected,
        );
        // A blank line after the opening brace is not doubled.
        assert_eq!(
            rewrite(
                "{ lib, config, ... }:\nwith lib;\nlet\n  cfg = config.a;\nin\n{\n\n  config.b = cfg;\n}\n",
                &["./a.nix", "./b.nix"]
            )
            .unwrap(),
            expected
        );
    }

    #[test]
    fn adds_to_a_list_on_several_lines() {
        round_trip(
            "{\n  imports = [\n    ./manual.nix\n  ];\n}\n",
            "{\n  imports = [\n    ./manual.nix\n    # BEGIN generated imports: nix-module-imports writes the lines up to END\n    ./a.nix\n    ./b.nix\n    # END generated imports\n  ];\n}\n",
        );
    }

    #[test]
    fn adds_to_a_list_on_one_line() {
        let written = rewrite("{\n  imports = [ ./manual.nix ];\n}\n", &["./a.nix"]).unwrap();
        assert_eq!(
            written,
            "{\n  imports = [\n    ./manual.nix\n    # BEGIN generated imports: nix-module-imports writes the lines up to END\n    ./a.nix\n    # END generated imports\n  ];\n}\n"
        );
        assert_eq!(
            rewrite(&written, &[]).unwrap(),
            "{\n  imports = [\n    ./manual.nix\n  ];\n}\n"
        );
    }

    #[test]
    fn adds_to_the_last_list_of_a_chain() {
        let written = rewrite(
            "{ lib, ... }:\n{\n  imports = lib.optional true ./x.nix ++ [\n    ./manual.nix\n  ];\n}\n",
            &["./a.nix"],
        )
        .unwrap();
        assert!(written.contains("    ./manual.nix\n    # BEGIN generated imports"));
    }

    #[test]
    fn adds_to_a_list_behind_a_let() {
        let written = rewrite(
            "{\n  imports =\n    let\n      x = ./x.nix;\n    in\n    [\n      x\n    ];\n}\n",
            &["./a.nix"],
        )
        .unwrap();
        assert!(written.contains("      x\n      # BEGIN generated imports"));
    }

    #[test]
    fn appends_a_list_to_a_computed_value() {
        let written = rewrite(
            "{ lib, ... }:\n{\n  imports = lib.optional true ./x.nix;\n}\n",
            &["./a.nix"],
        )
        .unwrap();
        assert_eq!(
            written,
            "{ lib, ... }:\n{\n  imports = lib.optional true ./x.nix\n    ++ [\n      # BEGIN generated imports: nix-module-imports writes the lines up to END\n      ./a.nix\n      # END generated imports\n    ];\n}\n"
        );
        // The next run finds the list it appended.
        assert_eq!(rewrite(&written, &["./a.nix"]).unwrap(), written);
        let written = rewrite(
            "{ lib, ... }:\n{\n  imports = if true then [ ] else throw \"no\";\n}\n",
            &["./a.nix"],
        )
        .unwrap();
        assert!(written.contains("imports = (if true then [ ] else throw \"no\")\n    ++ ["));
    }

    #[test]
    fn replaces_what_it_wrote_before() {
        let before = rewrite(
            "{\n  imports = [\n    ./manual.nix\n  ];\n}\n",
            &["./old.nix"],
        )
        .unwrap();
        let after = rewrite(&before, &["./new.nix"]).unwrap();
        assert!(after.contains("./new.nix"));
        assert!(!after.contains("./old.nix"));
        assert!(after.contains("./manual.nix"));
    }

    #[test]
    fn fills_an_empty_module() {
        assert_eq!(
            rewrite("{ }\n", &["./a.nix"]).unwrap(),
            "{\n  imports = [\n    # BEGIN generated imports: nix-module-imports writes the lines up to END\n    ./a.nix\n    # END generated imports\n  ];\n}\n"
        );
    }

    #[test]
    fn reports_what_it_cannot_rewrite() {
        assert_eq!(
            rewrite("{ lib, ... }: lib.mkIf true { }\n", &["./a.nix"]),
            Err(Unsupported::BodyIsNotAnAttributeSet)
        );
        assert_eq!(
            rewrite(
                "let imports = [ ]; in\n{\n  inherit imports;\n}\n",
                &["./a.nix"]
            ),
            Err(Unsupported::ImportsIsInherited)
        );
        assert_eq!(
            rewrite(
                "{\n  imports = [\n    # BEGIN generated imports\n  ];\n}\n",
                &["./a.nix"]
            ),
            Err(Unsupported::DividerCommentsAreIncomplete)
        );
        assert_eq!(
            rewrite("{ a = ; }\n", &["./a.nix"]),
            Err(Unsupported::DoesNotParse)
        );
    }

    #[test]
    fn leaves_a_module_that_needs_nothing_alone() {
        let text = "{ lib, ... }: lib.mkIf true { }\n";
        assert_eq!(
            rewrite(text, &[]),
            Err(Unsupported::BodyIsNotAnAttributeSet)
        );
        let text = "{\n  imports = [ ./manual.nix ];\n}\n";
        assert_eq!(rewrite(text, &[]).unwrap(), text);
    }

    #[test]
    fn finds_the_generated_imports() {
        let module = Module::parse(
            "{\n  imports = [\n    ./one.nix\n    ../two\n    <nixpkgs/three.nix>\n    # BEGIN generated imports\n    ./generated.nix\n    ../other\n    # END generated imports\n  ];\n}\n"
                .to_string(),
        )
        .unwrap();
        let generated: Vec<String> = module
            .generated_imports("modules/services")
            .unwrap()
            .into_iter()
            .collect();
        assert_eq!(
            generated,
            ["modules/other", "modules/services/generated.nix"]
        );
    }

    #[test]
    fn writes_relative_paths() {
        assert_eq!(relative_literal("a/b", "a/b/c.nix"), "./c.nix");
        assert_eq!(relative_literal("a/b", "a/c/d.nix"), "../c/d.nix");
        assert_eq!(relative_literal("a/b", "x.nix"), "../../x.nix");
        assert_eq!(relative_literal("", "x/y"), "./x/y");
        assert_eq!(relative_literal("a/b", "a/b"), "./.");
        assert_eq!(relative_literal("a/b/c", "a/b"), "../.");
    }
}
