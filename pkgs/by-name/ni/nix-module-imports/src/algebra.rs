//! A second interpretation of the Nix language.
//!
//! The standard interpreter computes values. This one answers another
//! question: which attribute paths can an expression touch? It reads the
//! same language as an algebra and gives its operations another meaning.
//!
//! - Nothing is decided by a condition. An `if` is both of its branches.
//!   `mkIf` is therefore nothing but its content, and no library helper
//!   has to be known by name: `optionalAttrs` is read from its source,
//!   where it is an `if`.
//! - A value that the question does not depend on, such as a number, is
//!   `Unknown`. So is every input that only a user has, and so is the
//!   result of a builtin that reads a file or the store.
//! - The configuration is an input. Selecting from it gives the path that
//!   was selected, and the selection is recorded as a read.
//! - Functions are applied and files are imported like in the standard
//!   interpreter, lazily. The library is read from its source.
//!
//! Every kind of expression of the language has a case here. Every
//! builtin has a meaning in `builtin`, and one that has none stops the
//! run with its name. So what this interpretation does not know is an
//! input, and never a construct it has no case for.
//!
//! The interpretation cannot always end where the standard one does,
//! because it takes both branches of the `if` that ends a recursion. A
//! call that goes deeper than `DEPTH` gives `Unknown`, and the run counts
//! how often that happened.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rnix::ast::{self, HasEntry, InterpolPart};
use rowan::ast::AstNode;

/// How deep calls may go before the result is `Unknown`.
const DEPTH: usize = 400;

#[derive(Clone)]
pub enum Value {
    /// A value that is not tracked.
    Unknown,
    /// A string that is known. Names of attributes come from these.
    Str(Rc<str>),
    /// A path that is known. Imports follow these.
    Path(Rc<PathBuf>),
    Attrs(Rc<Attrs>),
    /// A list, as the values that can be among its elements.
    List(Rc<Vec<Thunk>>),
    Lambda(Rc<Closure>),
    Builtin(&'static str, Rc<Vec<Thunk>>),
    /// One of several values: what both branches of an `if` give.
    Either(Rc<Vec<Value>>),
    /// The configuration, at a path.
    Config(Rc<Vec<String>>),
}

#[derive(Default)]
pub struct Attrs {
    pub fields: BTreeMap<String, Thunk>,
    /// The entries whose names are not known.
    pub any: Option<Thunk>,
}

pub struct Closure {
    param: ast::Param,
    body: ast::Expr,
    env: Env,
    file: Rc<PathBuf>,
}

type Delayed = Rc<dyn Fn(&mut Interpreter) -> Value>;

/// One entry of a set as it is written: the path of names, where a name
/// that is not known is `None`, and the value.
type Entry = (Vec<Option<String>>, Thunk);

enum State {
    Delayed(Delayed),
    Forcing,
    Done(Value),
}

#[derive(Clone)]
pub struct Thunk(Rc<RefCell<State>>);

impl Thunk {
    pub fn done(value: Value) -> Thunk {
        Thunk(Rc::new(RefCell::new(State::Done(value))))
    }

    fn delayed(work: impl Fn(&mut Interpreter) -> Value + 'static) -> Thunk {
        Thunk(Rc::new(RefCell::new(State::Delayed(Rc::new(work)))))
    }
}

struct Scope {
    names: RefCell<BTreeMap<String, Thunk>>,
    /// The value of a `with`, when this scope is one.
    with: Option<Thunk>,
    parent: Env,
}

#[derive(Clone)]
struct Env(Option<Rc<Scope>>);

impl Env {
    fn child(&self, names: BTreeMap<String, Thunk>) -> Env {
        Env(Some(Rc::new(Scope {
            names: RefCell::new(names),
            with: None,
            parent: self.clone(),
        })))
    }

    fn with(&self, value: Thunk) -> Env {
        Env(Some(Rc::new(Scope {
            names: RefCell::new(BTreeMap::new()),
            with: Some(value),
            parent: self.clone(),
        })))
    }

    fn define(&self, name: String, value: Thunk) {
        if let Some(scope) = &self.0 {
            scope.names.borrow_mut().insert(name, value);
        }
    }
}

pub fn either(values: Vec<Value>) -> Value {
    let mut flat = Vec::new();
    for value in values {
        match value {
            Value::Either(inner) => flat.extend(inner.iter().cloned()),
            other => flat.push(other),
        }
    }
    if flat.len() == 1 {
        flat.pop().unwrap()
    } else {
        Value::Either(Rc::new(flat))
    }
}

/// The value that nothing can be: what `throw` gives.
fn nothing() -> Value {
    Value::Either(Rc::new(Vec::new()))
}

pub struct Interpreter {
    imported: HashMap<PathBuf, Value>,
    /// The paths of the configuration that were read.
    pub reads: BTreeSet<Vec<String>>,
    depth: usize,
    /// How often a call went too deep and gave `Unknown`.
    pub too_deep: usize,
    /// How often a value was needed while it was being worked out.
    pub cycles: usize,
    /// What stops the run: a builtin without a meaning here, or a file
    /// that does not parse.
    pub fatal: Option<String>,
}

fn arity(name: &str) -> Option<usize> {
    Some(match name {
        "abort"
        | "attrNames"
        | "attrValues"
        | "baseNameOf"
        | "break"
        | "ceil"
        | "concatLists"
        | "derivation"
        | "derivationStrict"
        | "dirOf"
        | "fetchGit"
        | "fetchTarball"
        | "fetchTree"
        | "fetchurl"
        | "floor"
        | "fromJSON"
        | "fromTOML"
        | "functionArgs"
        | "genericClosure"
        | "getContext"
        | "getEnv"
        | "getFlake"
        | "hasContext"
        | "head"
        | "import"
        | "isAttrs"
        | "isBool"
        | "isFloat"
        | "isFunction"
        | "isInt"
        | "isList"
        | "isNull"
        | "isPath"
        | "isString"
        | "length"
        | "listToAttrs"
        | "parseDrvName"
        | "path"
        | "pathExists"
        | "placeholder"
        | "readDir"
        | "readFile"
        | "readFileType"
        | "splitVersion"
        | "storePath"
        | "stringLength"
        | "tail"
        | "throw"
        | "toJSON"
        | "toPath"
        | "toString"
        | "toXML"
        | "tryEval"
        | "typeOf"
        | "unsafeDiscardStringContext"
        | "unsafeDiscardOutputDependency"
        | "addDrvOutputDependencies"
        | "convertHash"
        | "flakeRefToString"
        | "parseFlakeRef" => 1,
        "add" | "addErrorContext" | "all" | "any" | "appendContext" | "bitAnd" | "bitOr"
        | "bitXor" | "catAttrs" | "compareVersions" | "concatMap" | "concatStringsSep"
        | "deepSeq" | "div" | "elem" | "elemAt" | "filter" | "filterSource" | "findFile"
        | "genList" | "getAttr" | "groupBy" | "hasAttr" | "hashFile" | "hashString"
        | "intersectAttrs" | "lessThan" | "map" | "mapAttrs" | "match" | "mul" | "outputOf"
        | "partition" | "removeAttrs" | "scopedImport" | "seq" | "sort" | "split" | "sub"
        | "toFile" | "trace" | "traceVerbose" | "unsafeGetAttrPos" | "warn" | "zipAttrsWith" => 2,
        "foldl'" | "replaceStrings" | "substring" => 3,
        _ => return None,
    })
}

/// The builtins that are values and not functions.
const CONSTANTS: [&str; 9] = [
    "currentSystem",
    "currentTime",
    "langVersion",
    "nixPath",
    "nixVersion",
    "storeDir",
    "true",
    "false",
    "null",
];

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            imported: HashMap::new(),
            reads: BTreeSet::new(),
            depth: 0,
            too_deep: 0,
            cycles: 0,
            fatal: None,
        }
    }

    fn stop(&mut self, why: String) -> Value {
        if self.fatal.is_none() {
            self.fatal = Some(why);
        }
        Value::Unknown
    }

    pub fn force(&mut self, thunk: &Thunk) -> Value {
        let state = thunk.0.replace(State::Forcing);
        match state {
            State::Done(value) => {
                *thunk.0.borrow_mut() = State::Done(value.clone());
                value
            }
            State::Forcing => {
                self.cycles += 1;
                Value::Unknown
            }
            State::Delayed(work) => {
                let value = work(self);
                *thunk.0.borrow_mut() = State::Done(value.clone());
                value
            }
        }
    }

    fn delay(&self, expr: Option<ast::Expr>, env: &Env, file: &Rc<PathBuf>) -> Thunk {
        let (env, file) = (env.clone(), file.clone());
        match expr {
            Some(expr) => Thunk::delayed(move |interpreter| interpreter.eval(&expr, &env, &file)),
            None => Thunk::done(Value::Unknown),
        }
    }

    fn builtins(&self) -> Value {
        Value::Builtin("builtins", Rc::new(Vec::new()))
    }

    fn lookup(&mut self, name: &str, env: &Env) -> Value {
        let mut scope = env.0.clone();
        while let Some(current) = scope {
            if let Some(found) = current.names.borrow().get(name).cloned() {
                return self.force(&found);
            }
            scope = current.parent.0.clone();
        }
        // A name that is not bound comes from a `with`, the nearest first.
        let mut scope = env.0.clone();
        let mut from_with = Vec::new();
        while let Some(current) = scope {
            if let Some(with) = &current.with {
                let value = self.force(with);
                if let Some(found) = self.select_one(&value, name) {
                    from_with.push(found);
                    // A set that is known and has the name settles it.
                    if matches!(value, Value::Attrs(ref attrs) if attrs.fields.contains_key(name)) {
                        return either(from_with);
                    }
                }
            }
            scope = current.parent.0.clone();
        }
        if !from_with.is_empty() {
            return either(from_with);
        }
        match name {
            "builtins" => self.builtins(),
            "true" | "false" | "null" => Value::Unknown,
            _ if arity(name).is_some() => Value::Builtin(
                // The names are the ones of `arity`, which are static.
                Box::leak(name.to_string().into_boxed_str()),
                Rc::new(Vec::new()),
            ),
            _ => Value::Unknown,
        }
    }

    /// Select one name. `None` when the value is a set that is known not
    /// to have it.
    fn select_one(&mut self, value: &Value, name: &str) -> Option<Value> {
        match value {
            Value::Attrs(attrs) => match attrs.fields.get(name) {
                Some(field) => Some(self.force(field)),
                None => attrs.any.clone().map(|any| self.force(&any)),
            },
            Value::Config(path) => {
                let mut path = path.as_ref().clone();
                path.push(name.to_string());
                self.reads.insert(path.clone());
                Some(Value::Config(Rc::new(path)))
            }
            Value::Either(values) => {
                let found: Vec<Value> = values
                    .iter()
                    .filter_map(|value| self.select_one(value, name))
                    .collect();
                (!found.is_empty()).then(|| either(found))
            }
            Value::Builtin("builtins", _) => Some(if CONSTANTS.contains(&name) {
                Value::Unknown
            } else if arity(name).is_some() {
                Value::Builtin(
                    Box::leak(name.to_string().into_boxed_str()),
                    Rc::new(Vec::new()),
                )
            } else {
                self.stop(format!(
                    "the builtin `{name}` has no meaning in this interpretation"
                ))
            }),
            _ => Some(Value::Unknown),
        }
    }

    fn name_of(&mut self, attr: &ast::Attr, env: &Env, file: &Rc<PathBuf>) -> Option<String> {
        match attr {
            ast::Attr::Ident(ident) => Some(ident.syntax().text().to_string()),
            ast::Attr::Str(string) => match self.string(string, env, file) {
                Value::Str(text) => Some(text.to_string()),
                _ => None,
            },
            ast::Attr::Dynamic(dynamic) => match dynamic.expr() {
                Some(expr) => match self.eval(&expr, env, file) {
                    Value::Str(text) => Some(text.to_string()),
                    _ => None,
                },
                None => None,
            },
        }
    }

    fn string(&mut self, string: &ast::Str, env: &Env, file: &Rc<PathBuf>) -> Value {
        let mut text = String::new();
        let mut known = true;
        for part in string.normalized_parts() {
            match part {
                InterpolPart::Literal(literal) => text.push_str(&literal),
                InterpolPart::Interpolation(interpolation) => {
                    match interpolation.expr().map(|expr| self.eval(&expr, env, file)) {
                        Some(Value::Str(inner)) => text.push_str(&inner),
                        Some(Value::Path(path)) => text.push_str(&path.to_string_lossy()),
                        _ => known = false,
                    }
                }
            }
        }
        if known {
            Value::Str(text.into())
        } else {
            Value::Unknown
        }
    }

    /// Build a set from entries with paths, as `a.b = 1; a.c = 2;`.
    fn build(&mut self, entries: Vec<Entry>) -> Attrs {
        let mut direct: BTreeMap<String, Thunk> = BTreeMap::new();
        let mut nested: BTreeMap<String, Vec<Entry>> = BTreeMap::new();
        let mut any: Vec<Thunk> = Vec::new();
        for (mut path, value) in entries {
            if path.is_empty() {
                continue;
            }
            let first = path.remove(0);
            let value = if path.is_empty() {
                value
            } else {
                match &first {
                    Some(name) => {
                        nested.entry(name.clone()).or_default().push((path, value));
                        continue;
                    }
                    None => Thunk::done(Value::Attrs(Rc::new(self.build(vec![(path, value)])))),
                }
            };
            match first {
                Some(name) => {
                    direct.insert(name, value);
                }
                None => any.push(value),
            }
        }
        for (name, entries) in nested {
            let below = Value::Attrs(Rc::new(self.build(entries)));
            let value = match direct.remove(&name) {
                // `a = x; a.b = y;` is not allowed in the language, and
                // `a = { ... }; a.b = y;` merges. Both are kept.
                Some(existing) => {
                    let below = below.clone();
                    Thunk::delayed(move |interpreter| {
                        let existing = interpreter.force(&existing);
                        interpreter.update(existing, below.clone())
                    })
                }
                None => Thunk::done(below),
            };
            direct.insert(name, value);
        }
        let any = match any.len() {
            0 => None,
            1 => any.pop(),
            _ => Some(Thunk::delayed(move |interpreter| {
                let values = any.iter().map(|thunk| interpreter.force(thunk)).collect();
                either(values)
            })),
        };
        Attrs {
            fields: direct,
            any,
        }
    }

    /// `left // right`.
    fn update(&mut self, left: Value, right: Value) -> Value {
        match (&left, &right) {
            (Value::Attrs(left), Value::Attrs(right)) => {
                let mut fields = left.fields.clone();
                fields.extend(
                    right
                        .fields
                        .iter()
                        .map(|(name, value)| (name.clone(), value.clone())),
                );
                let any = match (&left.any, &right.any) {
                    (Some(first), Some(second)) => {
                        let (first, second) = (first.clone(), second.clone());
                        Some(Thunk::delayed(move |interpreter| {
                            either(vec![interpreter.force(&first), interpreter.force(&second)])
                        }))
                    }
                    (first, second) => first.clone().or_else(|| second.clone()),
                };
                Value::Attrs(Rc::new(Attrs { fields, any }))
            }
            (Value::Either(values), _) => {
                let values = values.as_ref().clone();
                either(
                    values
                        .into_iter()
                        .map(|value| self.update(value, right.clone()))
                        .collect(),
                )
            }
            (_, Value::Either(values)) => {
                let values = values.as_ref().clone();
                either(
                    values
                        .into_iter()
                        .map(|value| self.update(left.clone(), value))
                        .collect(),
                )
            }
            // One side is not known. What the other side has is there,
            // and more can be: the entries of the unknown side.
            (Value::Attrs(known), _) | (_, Value::Attrs(known)) => Value::Attrs(Rc::new(Attrs {
                fields: known.fields.clone(),
                any: Some(Thunk::done(Value::Unknown)),
            })),
            _ => Value::Unknown,
        }
    }

    fn set(&mut self, set: &impl HasEntry, env: &Env, file: &Rc<PathBuf>) -> Attrs {
        let mut entries = Vec::new();
        for entry in set.attrpath_values() {
            let Some(attrpath) = entry.attrpath() else {
                continue;
            };
            let path: Vec<Option<String>> = attrpath
                .attrs()
                .map(|attr| self.name_of(&attr, env, file))
                .collect();
            entries.push((path, self.delay(entry.value(), env, file)));
        }
        for inherit in set.inherits() {
            let from = inherit.from().and_then(|from| from.expr());
            let from = from.map(|from| self.delay(Some(from), env, file));
            for attr in inherit.attrs() {
                let Some(name) = self.name_of(&attr, env, file) else {
                    continue;
                };
                let value = match &from {
                    Some(from) => {
                        let (from, name) = (from.clone(), name.clone());
                        Thunk::delayed(move |interpreter| {
                            let from = interpreter.force(&from);
                            interpreter
                                .select_one(&from, &name)
                                .unwrap_or(Value::Unknown)
                        })
                    }
                    None => {
                        let (env, name) = (env.clone(), name.clone());
                        Thunk::delayed(move |interpreter| interpreter.lookup(&name, &env))
                    }
                };
                entries.push((vec![Some(name)], value));
            }
        }
        self.build(entries)
    }

    fn eval(&mut self, expr: &ast::Expr, env: &Env, file: &Rc<PathBuf>) -> Value {
        if self.fatal.is_some() {
            return Value::Unknown;
        }
        match expr {
            ast::Expr::Paren(paren) => match paren.expr() {
                Some(inner) => self.eval(&inner, env, file),
                None => Value::Unknown,
            },
            ast::Expr::Root(root) => match root.expr() {
                Some(inner) => self.eval(&inner, env, file),
                None => Value::Unknown,
            },
            ast::Expr::Ident(ident) => {
                let name = ident.syntax().text().to_string();
                self.lookup(&name, env)
            }
            ast::Expr::Literal(_) => Value::Unknown,
            ast::Expr::Str(string) => self.string(string, env, file),
            ast::Expr::Path(path) => {
                let text = path.syntax().text().to_string();
                if text.contains("${") || text.starts_with('<') || text.starts_with('~') {
                    Value::Unknown
                } else if text.starts_with('/') {
                    Value::Path(Rc::new(PathBuf::from(text)))
                } else {
                    let directory = file.parent().unwrap_or(Path::new("/"));
                    Value::Path(Rc::new(normalize(&directory.join(text))))
                }
            }
            ast::Expr::List(list) => Value::List(Rc::new(
                list.items()
                    .map(|item| self.delay(Some(item), env, file))
                    .collect(),
            )),
            ast::Expr::AttrSet(set) => {
                if set.rec_token().is_some() {
                    let inner = env.child(BTreeMap::new());
                    let attrs = self.set(set, &inner, file);
                    for (name, value) in &attrs.fields {
                        inner.define(name.clone(), value.clone());
                    }
                    Value::Attrs(Rc::new(attrs))
                } else {
                    Value::Attrs(Rc::new(self.set(set, env, file)))
                }
            }
            ast::Expr::LetIn(let_in) => {
                let inner = env.child(BTreeMap::new());
                let attrs = self.set(let_in, &inner, file);
                for (name, value) in &attrs.fields {
                    inner.define(name.clone(), value.clone());
                }
                match let_in.body() {
                    Some(body) => self.eval(&body, &inner, file),
                    None => Value::Unknown,
                }
            }
            ast::Expr::LegacyLet(legacy) => {
                let inner = env.child(BTreeMap::new());
                let attrs = self.set(legacy, &inner, file);
                for (name, value) in &attrs.fields {
                    inner.define(name.clone(), value.clone());
                }
                match attrs.fields.get("body") {
                    Some(body) => self.force(body),
                    None => Value::Unknown,
                }
            }
            ast::Expr::With(with) => {
                let namespace = self.delay(with.namespace(), env, file);
                match with.body() {
                    Some(body) => self.eval(&body, &env.with(namespace), file),
                    None => Value::Unknown,
                }
            }
            // The condition of an assertion decides nothing here.
            ast::Expr::Assert(assert) => match assert.body() {
                Some(body) => self.eval(&body, env, file),
                None => Value::Unknown,
            },
            // Both branches. The condition is read, because reading the
            // configuration in it is a touch, and its value is dropped.
            ast::Expr::IfElse(if_else) => {
                if let Some(condition) = if_else.condition() {
                    self.eval(&condition, env, file);
                }
                let mut values = Vec::new();
                for branch in [if_else.body(), if_else.else_body()].into_iter().flatten() {
                    values.push(self.eval(&branch, env, file));
                }
                either(values)
            }
            ast::Expr::Lambda(lambda) => match (lambda.param(), lambda.body()) {
                (Some(param), Some(body)) => Value::Lambda(Rc::new(Closure {
                    param,
                    body,
                    env: env.clone(),
                    file: file.clone(),
                })),
                _ => Value::Unknown,
            },
            ast::Expr::Apply(apply) => {
                let Some(function) = apply.lambda() else {
                    return Value::Unknown;
                };
                let function = self.eval(&function, env, file);
                let argument = self.delay(apply.argument(), env, file);
                self.apply(function, argument)
            }
            ast::Expr::Select(select) => {
                let Some(base) = select.expr() else {
                    return Value::Unknown;
                };
                let mut value = self.eval(&base, env, file);
                let mut missing = false;
                if let Some(attrpath) = select.attrpath() {
                    for attr in attrpath.attrs() {
                        value = match self.name_of(&attr, env, file) {
                            Some(name) => match self.select_one(&value, &name) {
                                Some(found) => found,
                                None => {
                                    missing = true;
                                    break;
                                }
                            },
                            // A name that is not known: any entry.
                            None => self.any_entry(&value),
                        };
                    }
                }
                match (select.default_expr(), missing) {
                    (Some(default), true) => self.eval(&default, env, file),
                    // The default is taken when the path is not there,
                    // which is not known for a value that is not known.
                    (Some(default), false) => {
                        let default = self.eval(&default, env, file);
                        match value {
                            Value::Attrs(_) | Value::Lambda(_) | Value::List(_) | Value::Str(_) => {
                                value
                            }
                            other => either(vec![other, default]),
                        }
                    }
                    (None, true) => nothing(),
                    (None, false) => value,
                }
            }
            ast::Expr::HasAttr(has) => {
                if let Some(base) = has.expr() {
                    let value = self.eval(&base, env, file);
                    // Asking the configuration for a name reads it.
                    if let (Value::Config(_), Some(attrpath)) = (&value, has.attrpath()) {
                        let mut value = value.clone();
                        for attr in attrpath.attrs() {
                            let Some(name) = self.name_of(&attr, env, file) else {
                                break;
                            };
                            value = self.select_one(&value, &name).unwrap_or(Value::Unknown);
                        }
                    }
                }
                Value::Unknown
            }
            ast::Expr::UnaryOp(op) => {
                if let Some(inner) = op.expr() {
                    self.eval(&inner, env, file);
                }
                match op.operator() {
                    Some(ast::UnaryOpKind::Invert) | Some(ast::UnaryOpKind::Negate) | None => {
                        Value::Unknown
                    }
                }
            }
            ast::Expr::BinOp(op) => {
                let left = op
                    .lhs()
                    .map(|lhs| self.eval(&lhs, env, file))
                    .unwrap_or(Value::Unknown);
                match op.operator() {
                    Some(ast::BinOpKind::PipeRight) => {
                        let function = op
                            .rhs()
                            .map(|rhs| self.eval(&rhs, env, file))
                            .unwrap_or(Value::Unknown);
                        return self.apply(function, Thunk::done(left));
                    }
                    Some(ast::BinOpKind::PipeLeft) => {
                        let argument = self.delay(op.rhs(), env, file);
                        return self.apply(left, argument);
                    }
                    _ => {}
                }
                let right = op
                    .rhs()
                    .map(|rhs| self.eval(&rhs, env, file))
                    .unwrap_or(Value::Unknown);
                match op.operator() {
                    Some(ast::BinOpKind::Update) => self.update(left, right),
                    Some(ast::BinOpKind::Concat) => self.concat(left, right),
                    Some(ast::BinOpKind::Add) => match (&left, &right) {
                        (Value::Str(first), Value::Str(second)) => {
                            Value::Str(format!("{first}{second}").into())
                        }
                        (Value::Path(first), Value::Str(second)) => {
                            Value::Path(Rc::new(normalize(&PathBuf::from(format!(
                                "{}{second}",
                                first.to_string_lossy()
                            )))))
                        }
                        _ => Value::Unknown,
                    },
                    Some(
                        ast::BinOpKind::Sub
                        | ast::BinOpKind::Mul
                        | ast::BinOpKind::Div
                        | ast::BinOpKind::And
                        | ast::BinOpKind::Equal
                        | ast::BinOpKind::Implication
                        | ast::BinOpKind::Less
                        | ast::BinOpKind::LessOrEq
                        | ast::BinOpKind::More
                        | ast::BinOpKind::MoreOrEq
                        | ast::BinOpKind::NotEqual
                        | ast::BinOpKind::Or,
                    ) => Value::Unknown,
                    Some(ast::BinOpKind::PipeRight | ast::BinOpKind::PipeLeft) | None => {
                        Value::Unknown
                    }
                }
            }
            ast::Expr::Error(_) => {
                self.stop(format!("{} has a part that does not parse", file.display()))
            }
        }
    }

    /// Any entry of a set or any element of a list.
    fn any_entry(&mut self, value: &Value) -> Value {
        match value {
            Value::Attrs(attrs) => {
                let mut values: Vec<Value> = Vec::new();
                for field in attrs.fields.values().cloned().collect::<Vec<_>>() {
                    values.push(self.force(&field));
                }
                if let Some(any) = attrs.any.clone() {
                    values.push(self.force(&any));
                }
                either(values)
            }
            Value::List(items) => {
                let values = items.iter().map(|item| self.force(item)).collect();
                either(values)
            }
            Value::Config(path) => {
                let mut path = path.as_ref().clone();
                path.push("*".to_string());
                self.reads.insert(path.clone());
                Value::Config(Rc::new(path))
            }
            Value::Either(values) => {
                let values = values.iter().map(|value| self.any_entry(value)).collect();
                either(values)
            }
            _ => Value::Unknown,
        }
    }

    fn concat(&mut self, left: Value, right: Value) -> Value {
        let mut items = Vec::new();
        for side in [left, right] {
            match side {
                Value::List(list) => items.extend(list.iter().cloned()),
                // A list that is not known: one element that stands for
                // all of them.
                other => {
                    let element = self.any_entry(&other);
                    items.push(Thunk::done(element));
                }
            }
        }
        Value::List(Rc::new(items))
    }

    pub fn apply(&mut self, function: Value, argument: Thunk) -> Value {
        match function {
            Value::Lambda(closure) => {
                if self.depth >= DEPTH {
                    self.too_deep += 1;
                    return Value::Unknown;
                }
                let mut names = BTreeMap::new();
                let inner = closure.env.child(BTreeMap::new());
                match &closure.param {
                    ast::Param::IdentParam(param) => {
                        if let Some(ident) = param.ident() {
                            names.insert(ident.syntax().text().to_string(), argument);
                        }
                    }
                    ast::Param::Pattern(pattern) => {
                        if let Some(bind) = pattern.pat_bind().and_then(|bind| bind.ident()) {
                            names.insert(bind.syntax().text().to_string(), argument.clone());
                        }
                        for entry in pattern.pat_entries() {
                            let Some(ident) = entry.ident() else {
                                continue;
                            };
                            let name = ident.syntax().text().to_string();
                            let (argument, field, default, inner, file) = (
                                argument.clone(),
                                name.clone(),
                                entry.default(),
                                inner.clone(),
                                closure.file.clone(),
                            );
                            names.insert(
                                name,
                                Thunk::delayed(move |interpreter| {
                                    let given = interpreter.force(&argument);
                                    let found = match &given {
                                        Value::Attrs(_) | Value::Either(_) | Value::Config(_) => {
                                            interpreter.select_one(&given, &field)
                                        }
                                        _ => Some(Value::Unknown),
                                    };
                                    match (found, &default) {
                                        (Some(found), None) => found,
                                        (Some(found), Some(default)) => match given {
                                            // The set is known and has the name.
                                            Value::Attrs(ref attrs)
                                                if attrs.fields.contains_key(&field) =>
                                            {
                                                found
                                            }
                                            _ => either(vec![
                                                found,
                                                interpreter.eval(default, &inner, &file),
                                            ]),
                                        },
                                        (None, Some(default)) => {
                                            interpreter.eval(default, &inner, &file)
                                        }
                                        (None, None) => Value::Unknown,
                                    }
                                }),
                            );
                        }
                    }
                }
                for (name, value) in names {
                    inner.define(name, value);
                }
                self.depth += 1;
                let result = self.eval(&closure.body, &inner, &closure.file);
                self.depth -= 1;
                result
            }
            Value::Builtin(name, given) => {
                let mut given = given.as_ref().clone();
                given.push(argument);
                match arity(name) {
                    Some(needed) if given.len() < needed => Value::Builtin(name, Rc::new(given)),
                    Some(_) => self.builtin(name, &given),
                    None => self.stop(format!("`{name}` is called and is not a function here")),
                }
            }
            Value::Either(functions) => {
                let functions = functions.as_ref().clone();
                either(
                    functions
                        .into_iter()
                        .map(|function| self.apply(function, argument.clone()))
                        .collect(),
                )
            }
            // A set with `__functor` is a function.
            Value::Attrs(ref attrs) if attrs.fields.contains_key("__functor") => {
                let functor = self.force(&attrs.fields["__functor"]);
                let partly = self.apply(functor, Thunk::done(function.clone()));
                self.apply(partly, argument)
            }
            _ => {
                // The function is not known. Its argument is still read,
                // so that what it reads of the configuration is recorded.
                self.force(&argument);
                Value::Unknown
            }
        }
    }

    fn elements(&mut self, value: &Value) -> Vec<Thunk> {
        match value {
            Value::List(items) => items.as_ref().clone(),
            other => vec![Thunk::done(self.any_entry(other))],
        }
    }

    fn import(&mut self, path: &Path) -> Value {
        let path = if path.is_dir() {
            path.join("default.nix")
        } else {
            path.to_path_buf()
        };
        if let Some(known) = self.imported.get(&path) {
            return known.clone();
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Value::Unknown;
        };
        let parse = rnix::Root::parse(&text);
        if !parse.errors().is_empty() {
            return self.stop(format!("{} does not parse", path.display()));
        }
        // A file that imports itself, through others, gets `Unknown`.
        self.imported.insert(path.clone(), Value::Unknown);
        let value = match parse.tree().expr() {
            Some(expr) => self.eval(&expr, &Env(None), &Rc::new(path.clone())),
            None => Value::Unknown,
        };
        self.imported.insert(path, value.clone());
        value
    }

    pub fn import_file(&mut self, path: &Path) -> Value {
        self.import(path)
    }

    /// The meaning of each builtin. One that gives a value that is not
    /// tracked, such as a number or the content of a file, gives
    /// `Unknown`. Its arguments are read first where they can read the
    /// configuration.
    fn builtin(&mut self, name: &'static str, arguments: &[Thunk]) -> Value {
        let first = arguments[0].clone();
        match name {
            "import" => match self.force(&first) {
                Value::Path(path) => self.import(&path),
                Value::Str(text) if text.starts_with('/') => self.import(Path::new(&*text)),
                _ => Value::Unknown,
            },
            "scopedImport" => Value::Unknown,
            "throw" | "abort" => nothing(),
            "seq" | "deepSeq" | "trace" | "traceVerbose" | "warn" | "addErrorContext" => {
                self.force(&arguments[1])
            }
            "break"
            | "unsafeDiscardStringContext"
            | "unsafeDiscardOutputDependency"
            | "addDrvOutputDependencies" => self.force(&first),
            "toString" | "toPath" => match self.force(&first) {
                Value::Str(text) => Value::Str(text),
                Value::Path(path) => Value::Str(path.to_string_lossy().into()),
                _ => Value::Unknown,
            },
            "tryEval" => {
                let mut fields = BTreeMap::new();
                fields.insert("success".to_string(), Thunk::done(Value::Unknown));
                fields.insert("value".to_string(), first);
                Value::Attrs(Rc::new(Attrs { fields, any: None }))
            }
            "map" => {
                let function = self.force(&first);
                let list = self.force(&arguments[1]);
                let items = self
                    .elements(&list)
                    .into_iter()
                    .map(|item| {
                        let function = function.clone();
                        Thunk::delayed(move |interpreter| {
                            interpreter.apply(function.clone(), item.clone())
                        })
                    })
                    .collect();
                Value::List(Rc::new(items))
            }
            "genList" => {
                let function = self.force(&first);
                Value::List(Rc::new(vec![Thunk::delayed(move |interpreter| {
                    interpreter.apply(function.clone(), Thunk::done(Value::Unknown))
                })]))
            }
            "filter" | "sort" => {
                let list = self.force(&arguments[1]);
                Value::List(Rc::new(self.elements(&list)))
            }
            "partition" => {
                let list = self.force(&arguments[1]);
                let items = Thunk::done(Value::List(Rc::new(self.elements(&list))));
                let mut fields = BTreeMap::new();
                fields.insert("right".to_string(), items.clone());
                fields.insert("wrong".to_string(), items);
                Value::Attrs(Rc::new(Attrs { fields, any: None }))
            }
            "concatLists" => {
                let lists = self.force(&first);
                let mut items = Vec::new();
                for list in self.elements(&lists) {
                    let list = self.force(&list);
                    items.extend(self.elements(&list));
                }
                Value::List(Rc::new(items))
            }
            "concatMap" => {
                let function = self.force(&first);
                let list = self.force(&arguments[1]);
                let mut items = Vec::new();
                for item in self.elements(&list) {
                    let produced = self.apply(function.clone(), item);
                    items.extend(self.elements(&produced));
                }
                Value::List(Rc::new(items))
            }
            "head" | "elemAt" => {
                let list = self.force(&first);
                self.any_entry(&list)
            }
            "tail" => {
                let list = self.force(&first);
                Value::List(Rc::new(self.elements(&list)))
            }
            "foldl'" => {
                // The function is applied to each element once, with a
                // start that can be any of the results so far.
                let function = self.force(&first);
                let mut results = vec![self.force(&arguments[1])];
                let list = self.force(&arguments[2]);
                for item in self.elements(&list) {
                    let so_far = Thunk::done(either(results.clone()));
                    let partly = self.apply(function.clone(), so_far);
                    results.push(self.apply(partly, item));
                }
                either(results)
            }
            "attrNames" => match self.force(&first) {
                Value::Attrs(attrs) => {
                    let mut items: Vec<Thunk> = attrs
                        .fields
                        .keys()
                        .map(|name| Thunk::done(Value::Str(name.as_str().into())))
                        .collect();
                    if attrs.any.is_some() {
                        items.push(Thunk::done(Value::Unknown));
                    }
                    Value::List(Rc::new(items))
                }
                other => {
                    self.any_entry(&other);
                    Value::List(Rc::new(vec![Thunk::done(Value::Unknown)]))
                }
            },
            "attrValues" => {
                let set = self.force(&first);
                Value::List(Rc::new(vec![Thunk::done(self.any_entry(&set))]))
            }
            "mapAttrs" => {
                let function = self.force(&first);
                match self.force(&arguments[1]) {
                    Value::Attrs(attrs) => {
                        let call = |name: Value, value: Thunk, function: Value| {
                            Thunk::delayed(move |interpreter| {
                                let partly =
                                    interpreter.apply(function.clone(), Thunk::done(name.clone()));
                                interpreter.apply(partly, value.clone())
                            })
                        };
                        let fields = attrs
                            .fields
                            .iter()
                            .map(|(name, value)| {
                                (
                                    name.clone(),
                                    call(
                                        Value::Str(name.as_str().into()),
                                        value.clone(),
                                        function.clone(),
                                    ),
                                )
                            })
                            .collect();
                        let any = attrs
                            .any
                            .clone()
                            .map(|any| call(Value::Unknown, any, function.clone()));
                        Value::Attrs(Rc::new(Attrs { fields, any }))
                    }
                    other => {
                        let entry = Thunk::done(self.any_entry(&other));
                        Value::Attrs(Rc::new(Attrs {
                            fields: BTreeMap::new(),
                            any: Some(Thunk::delayed(move |interpreter| {
                                let partly = interpreter
                                    .apply(function.clone(), Thunk::done(Value::Unknown));
                                interpreter.apply(partly, entry.clone())
                            })),
                        }))
                    }
                }
            }
            "listToAttrs" => {
                let list = self.force(&first);
                let mut entries = Vec::new();
                for item in self.elements(&list) {
                    let item = self.force(&item);
                    let name = match self.select_one(&item, "name") {
                        Some(Value::Str(name)) => Some(name.to_string()),
                        _ => None,
                    };
                    let value = Thunk::delayed(move |interpreter| {
                        interpreter
                            .select_one(&item, "value")
                            .unwrap_or(Value::Unknown)
                    });
                    entries.push((vec![name], value));
                }
                Value::Attrs(Rc::new(self.build(entries)))
            }
            "removeAttrs" => match self.force(&first) {
                Value::Attrs(attrs) => {
                    let names = self.force(&arguments[1]);
                    let mut fields = attrs.fields.clone();
                    for name in self.elements(&names) {
                        if let Value::Str(name) = self.force(&name) {
                            fields.remove(&*name);
                        }
                    }
                    Value::Attrs(Rc::new(Attrs {
                        fields,
                        any: attrs.any.clone(),
                    }))
                }
                other => other,
            },
            "intersectAttrs" => {
                // The entries of the second set, at most.
                self.force(&first);
                self.force(&arguments[1])
            }
            "getAttr" => {
                let set = self.force(&arguments[1]);
                match self.force(&first) {
                    Value::Str(name) => self.select_one(&set, &name).unwrap_or_else(nothing),
                    _ => self.any_entry(&set),
                }
            }
            "catAttrs" => {
                let list = self.force(&arguments[1]);
                let name = self.force(&first);
                let mut items = Vec::new();
                for item in self.elements(&list) {
                    let item = self.force(&item);
                    let found = match &name {
                        Value::Str(name) => self.select_one(&item, name),
                        _ => Some(self.any_entry(&item)),
                    };
                    items.extend(found.map(Thunk::done));
                }
                Value::List(Rc::new(items))
            }
            "zipAttrsWith" | "groupBy" => {
                // The names are the ones of the sets of the list, and
                // the function gives the value of each from a list.
                let function = self.force(&first);
                let list = self.force(&arguments[1]);
                let mut by_name: BTreeMap<Option<String>, Vec<Thunk>> = BTreeMap::new();
                for item in self.elements(&list) {
                    match self.force(&item) {
                        Value::Attrs(attrs) if name == "zipAttrsWith" => {
                            for (field, value) in &attrs.fields {
                                by_name
                                    .entry(Some(field.clone()))
                                    .or_default()
                                    .push(value.clone());
                            }
                            if let Some(any) = &attrs.any {
                                by_name.entry(None).or_default().push(any.clone());
                            }
                        }
                        other => by_name.entry(None).or_default().push(Thunk::done(other)),
                    }
                }
                let mut entries = Vec::new();
                for (field, values) in by_name {
                    let function = function.clone();
                    let list = Thunk::done(Value::List(Rc::new(values)));
                    let given = field.clone();
                    let zip = name == "zipAttrsWith";
                    entries.push((
                        vec![field],
                        Thunk::delayed(move |interpreter| {
                            if zip {
                                let key = match &given {
                                    Some(given) => Value::Str(given.as_str().into()),
                                    None => Value::Unknown,
                                };
                                let partly = interpreter.apply(function.clone(), Thunk::done(key));
                                interpreter.apply(partly, list.clone())
                            } else {
                                interpreter.force(&list)
                            }
                        }),
                    ));
                }
                Value::Attrs(Rc::new(self.build(entries)))
            }
            "functionArgs" => match self.force(&first) {
                Value::Lambda(closure) => match &closure.param {
                    ast::Param::Pattern(pattern) => {
                        let fields = pattern
                            .pat_entries()
                            .filter_map(|entry| entry.ident())
                            .map(|ident| {
                                (
                                    ident.syntax().text().to_string(),
                                    Thunk::done(Value::Unknown),
                                )
                            })
                            .collect();
                        Value::Attrs(Rc::new(Attrs { fields, any: None }))
                    }
                    ast::Param::IdentParam(_) => Value::Attrs(Rc::new(Attrs::default())),
                },
                _ => Value::Attrs(Rc::new(Attrs {
                    fields: BTreeMap::new(),
                    any: Some(Thunk::done(Value::Unknown)),
                })),
            },
            "genericClosure" => {
                let given = self.force(&first);
                let start = self
                    .select_one(&given, "startSet")
                    .unwrap_or(Value::Unknown);
                let operator = self
                    .select_one(&given, "operator")
                    .unwrap_or(Value::Unknown);
                let mut items = self.elements(&start);
                let any = Thunk::done(self.any_entry(&start));
                let more = self.apply(operator, any);
                items.extend(self.elements(&more));
                Value::List(Rc::new(items))
            }
            // The builtins whose result is not tracked: numbers, tests,
            // strings that are computed, and what comes from files, the
            // store or the network. Their arguments are read, because
            // they can read the configuration.
            "add" | "all" | "any" | "appendContext" | "attrsets" | "baseNameOf" | "bitAnd"
            | "bitOr" | "bitXor" | "ceil" | "compareVersions" | "concatStringsSep"
            | "convertHash" | "derivation" | "derivationStrict" | "dirOf" | "div" | "elem"
            | "fetchGit" | "fetchTarball" | "fetchTree" | "fetchurl" | "filterSource"
            | "findFile" | "flakeRefToString" | "floor" | "fromJSON" | "fromTOML"
            | "getContext" | "getEnv" | "getFlake" | "hasAttr" | "hasContext" | "hashFile"
            | "hashString" | "isAttrs" | "isBool" | "isFloat" | "isFunction" | "isInt"
            | "isList" | "isNull" | "isPath" | "isString" | "length" | "lessThan" | "match"
            | "mul" | "outputOf" | "parseDrvName" | "parseFlakeRef" | "path" | "pathExists"
            | "placeholder" | "readDir" | "readFile" | "readFileType" | "replaceStrings"
            | "split" | "splitVersion" | "storePath" | "stringLength" | "sub" | "substring"
            | "toFile" | "toJSON" | "toXML" | "typeOf" | "unsafeGetAttrPos" => {
                for argument in arguments {
                    self.force(argument);
                }
                Value::Unknown
            }
            other => self.stop(format!(
                "the builtin `{other}` has no meaning in this interpretation"
            )),
        }
    }
}

/// A path without `.` and `..`.
fn normalize(path: &Path) -> PathBuf {
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    for part in path.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                parts.pop();
            }
            other => parts.push(other.as_os_str().to_os_string()),
        }
    }
    parts.iter().collect()
}

/// An input of a module whose reads are recorded: the configuration, as
/// `config`, or the options, as `options`. The name is the first part of
/// every path that is read from it.
pub fn tracked(name: &str) -> Value {
    Value::Config(Rc::new(vec![name.to_string()]))
}

pub fn attrs(fields: Vec<(&str, Value)>, open: bool) -> Value {
    Value::Attrs(Rc::new(Attrs {
        fields: fields
            .into_iter()
            .map(|(name, value)| (name.to_string(), Thunk::done(value)))
            .collect(),
        any: open.then(|| Thunk::done(Value::Unknown)),
    }))
}

pub fn string(text: &str) -> Value {
    Value::Str(text.into())
}
