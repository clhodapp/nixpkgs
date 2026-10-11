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
//! because it takes both branches of the `if` that ends a recursion. So
//! a call is cut when it is a recursion that has gone on for long
//! enough, and the run stops after a number of steps.
//!
//! Nothing of this may lose a touch. What the interpretation stops
//! following is escaped instead, see `Interpreter::escape`: a part of
//! the configuration counts as read with everything below it, and a
//! function is called with an argument that is not known. The result of
//! a call that is cut is not known, and a setting that is not known is
//! one that can have anything below it. So where the interpretation
//! stops, the answer gets wider, up to a path further up, and never
//! narrower. A run that stops altogether says that the file can touch
//! anything.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rnix::ast::{self, AstToken, HasEntry, InterpolPart};
use rowan::ast::AstNode;

/// How deep calls may go before a call is cut.
const DEPTH: usize = 400;

/// How many calls of one function may be under way inside each other
/// before a further call is cut. A helper that walks a path of names
/// calls itself once for each name, so this is also how long a path of
/// a setting can be before the setting is cut off at a parent.
const REENTRIES: usize = 8;

/// How many values a value may be one of before it is not followed as
/// each of them.
const ALTERNATIVES: usize = 32;

enum Pending {
    Value(Value),
    Thunk(Thunk),
}

/// How many expressions are interpreted before the run stops, and the
/// file counts as one that can touch anything.
const STEPS: u64 = 50_000_000;

#[derive(Clone)]
pub enum Value {
    /// A value that is not tracked.
    Unknown,
    /// A string that is known. Names of attributes come from these.
    Str(Rc<str>),
    /// A path that is known. Imports follow these.
    Path(Rc<PathBuf>),
    Attrs(Rc<Attrs>),
    /// A list whose elements are not all known, as the values that can
    /// be among them. It has no order and no length.
    List(Rc<Vec<Thunk>>),
    /// A list that is known element for element, in order.
    Seq(Rc<Vec<Thunk>>),
    /// A number that is known.
    Int(i64),
    /// A truth value that is known.
    Bool(bool),
    Null,
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
    /// The calls that were under way where the function was written
    /// down.
    chain: Chain,
}

impl Closure {
    /// What tells one function of the source from another: its file and
    /// where its body starts.
    fn identity(&self) -> Identity {
        (
            Rc::as_ptr(&self.file) as usize,
            u32::from(self.body.syntax().text_range().start()),
        )
    }
}

type Identity = (usize, u32);

/// The functions whose calls are under way, the innermost first.
#[derive(Clone, Default)]
struct Chain(Option<Rc<Link>>);

struct Link {
    identity: Identity,
    outer: Chain,
}

impl Chain {
    fn count(&self, identity: Identity) -> usize {
        let mut count = 0;
        let mut link = &self.0;
        while let Some(current) = link {
            if current.identity == identity {
                count += 1;
            }
            link = &current.outer.0;
        }
        count
    }

    fn with(&self, identity: Identity) -> Chain {
        Chain(Some(Rc::new(Link {
            identity,
            outer: self.clone(),
        })))
    }
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

struct Suspended {
    state: RefCell<State>,
    /// Whether the value can hold a part of the configuration, or a
    /// function that can reach one. `false` is a promise: nothing that
    /// is ever done with the value reads the configuration.
    reaches: bool,
    /// The calls that were under way where the value was written down.
    /// Working the value out continues from these, and not from the
    /// calls of whoever asks for it.
    chain: Chain,
    /// Everything that working the value out can use.
    sees: Sees,
    /// Whether the value was written down by code of the library.
    library: bool,
}

/// What the work of a delayed value can use: the names of a scope, and
/// values that it holds itself. It has to be all of it. A delayed value
/// of the library is escaped by escaping what it sees, without doing
/// the work, and anything that is left out here is then lost.
#[derive(Clone, Default)]
struct Sees {
    env: Option<Env>,
    thunks: Vec<Thunk>,
    values: Vec<Value>,
}

impl Sees {
    fn env(env: &Env) -> Sees {
        Sees {
            env: Some(env.clone()),
            ..Sees::default()
        }
    }

    fn thunk(mut self, thunk: &Thunk) -> Sees {
        self.thunks.push(thunk.clone());
        self
    }

    fn value(mut self, value: &Value) -> Sees {
        self.values.push(value.clone());
        self
    }

    fn reaches(&self) -> bool {
        self.env.as_ref().is_some_and(Env::reaches)
            || self.thunks.iter().any(|thunk| thunk.0.reaches)
            || self.values.iter().any(reaches)
    }
}

#[derive(Clone)]
pub struct Thunk(Rc<Suspended>);

impl Thunk {
    pub fn done(value: Value) -> Thunk {
        Thunk(Rc::new(Suspended {
            reaches: reaches(&value),
            state: RefCell::new(State::Done(value)),
            chain: Chain::default(),
            sees: Sees::default(),
            library: false,
        }))
    }
}

/// Whether a value can hold a part of the configuration, or a function
/// that can reach one.
fn reaches(value: &Value) -> bool {
    match value {
        Value::Config(_) => true,
        Value::Unknown
        | Value::Str(_)
        | Value::Path(_)
        | Value::Int(_)
        | Value::Bool(_)
        | Value::Null => false,
        Value::Attrs(attrs) => {
            attrs.fields.values().any(|field| field.0.reaches)
                || attrs.any.as_ref().is_some_and(|any| any.0.reaches)
        }
        Value::List(items) | Value::Seq(items) => items.iter().any(|item| item.0.reaches),
        Value::Lambda(closure) => closure.env.reaches(),
        Value::Builtin(_, given) => given.iter().any(|argument| argument.0.reaches),
        Value::Either(values) => values.iter().any(reaches),
    }
}

/// Whether a value is all of the configuration, all of the options or
/// all of the arguments of a module.
fn is_everything(value: &Value) -> bool {
    match value {
        Value::Config(path) => path.len() <= 1 || path.iter().map(String::as_str).eq(ARGUMENTS),
        Value::Either(values) => values.iter().any(is_everything),
        _ => false,
    }
}

/// Whether a name that a function can see is known to be all of the
/// configuration, all of the options or all of the arguments of a
/// module.
///
/// An argument of a call is worked out for this. A module that asks for
/// `options` and has not used it yet is then seen as what it is. A name
/// of a `let` is looked at only when it was worked out already.
fn sees_everything(interpreter: &mut Interpreter, env: &Env) -> bool {
    let mut scope = env.0.clone();
    while let Some(current) = scope {
        if !current.reaches {
            return false;
        }
        let names: Vec<Thunk> = current.names.borrow().values().cloned().collect();
        for name in names {
            if current.call && name.0.reaches {
                if is_everything(&interpreter.force(&name)) {
                    return true;
                }
            } else if let State::Done(value) = &*name.0.state.borrow()
                && is_everything(value)
            {
                return true;
            }
        }
        scope = current.parent.0.clone();
    }
    false
}

struct Scope {
    names: RefCell<BTreeMap<String, Thunk>>,
    /// Whether the names are the argument of a call and its parts.
    call: bool,
    /// The value of a `with`, when this scope is one.
    with: Option<Thunk>,
    parent: Env,
    /// Whether a name of this scope or of one around it can reach the
    /// configuration.
    reaches: bool,
}

#[derive(Clone)]
struct Env(Option<Rc<Scope>>);

impl Env {
    fn reaches(&self) -> bool {
        self.0.as_ref().is_some_and(|scope| scope.reaches)
    }

    /// A scope whose names are defined afterwards. `reaches` says
    /// whether one of them will reach the configuration.
    fn child(&self, reaches: bool) -> Env {
        Env(Some(Rc::new(Scope {
            names: RefCell::new(BTreeMap::new()),
            with: None,
            parent: self.clone(),
            reaches: reaches || self.reaches(),
            call: false,
        })))
    }

    /// The scope of a call, for the argument and its parts.
    fn call(&self, reaches: bool) -> Env {
        Env(Some(Rc::new(Scope {
            names: RefCell::new(BTreeMap::new()),
            with: None,
            parent: self.clone(),
            reaches: reaches || self.reaches(),
            call: true,
        })))
    }

    fn with(&self, value: Thunk) -> Env {
        Env(Some(Rc::new(Scope {
            names: RefCell::new(BTreeMap::new()),
            reaches: value.0.reaches || self.reaches(),
            with: Some(value),
            parent: self.clone(),
            call: false,
        })))
    }

    fn define(&self, name: String, value: Thunk) {
        if let Some(scope) = &self.0 {
            debug_assert!(scope.reaches || !value.0.reaches);
            scope.names.borrow_mut().insert(name, value);
        }
    }
}

/// What kind of value this is, when that is known.
fn kind(value: &Value) -> Option<&'static str> {
    Some(match value {
        Value::Int(_) => "int",
        Value::Bool(_) => "bool",
        Value::Null => "null",
        Value::Str(_) => "string",
        Value::Path(_) => "path",
        Value::Attrs(_) => "set",
        Value::List(_) | Value::Seq(_) => "list",
        Value::Lambda(_) | Value::Builtin(..) => "lambda",
        Value::Unknown | Value::Either(_) | Value::Config(_) => return None,
    })
}

/// Whether two values are the same known number, truth value, string,
/// path or null.
fn same_scalar(first: &Value, second: &Value) -> bool {
    match (first, second) {
        (Value::Int(first), Value::Int(second)) => first == second,
        (Value::Bool(first), Value::Bool(second)) => first == second,
        (Value::Str(first), Value::Str(second)) => first == second,
        (Value::Path(first), Value::Path(second)) => first == second,
        (Value::Null, Value::Null) | (Value::Unknown, Value::Unknown) => true,
        _ => false,
    }
}

/// Whether two values are one value: the same scalar, the same part of
/// the configuration, or the very same set, list or function.
fn same(first: &Value, second: &Value) -> bool {
    match (first, second) {
        (Value::Config(first), Value::Config(second)) => first == second,
        (Value::Attrs(first), Value::Attrs(second)) => Rc::ptr_eq(first, second),
        (Value::List(first), Value::List(second)) | (Value::Seq(first), Value::Seq(second)) => {
            Rc::ptr_eq(first, second)
        }
        (Value::Lambda(first), Value::Lambda(second)) => Rc::ptr_eq(first, second),
        _ => same_scalar(first, second),
    }
}

pub fn either(values: Vec<Value>) -> Value {
    let mut flat: Vec<Value> = Vec::new();
    for value in values {
        let parts = match value {
            Value::Either(inner) => inner.as_ref().clone(),
            other => vec![other],
        };
        for part in parts {
            if !flat.iter().any(|known| same(known, &part)) {
                flat.push(part);
            }
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
    /// Errors in the code that was read, which the standard interpreter
    /// reports only when it reaches them.
    pub problems: BTreeSet<String>,
    /// The paths of the configuration that were read with everything
    /// below them. The empty path is all of the configuration and all of
    /// the options.
    pub deep_reads: BTreeSet<Vec<String>>,
    /// The arguments of a module that are known: `config`, `options`
    /// and `lib`. Every other argument is what the configuration has at
    /// `_module.args`, which is where the module system takes it from.
    pub arguments: BTreeMap<String, Value>,
    /// The directory of the library that the modules get as `lib`.
    pub library: Option<PathBuf>,
    /// Take both sides of every `if`, whatever the condition is.
    both_sides: bool,
    /// How many calls of one function may be under way inside each other
    /// before a further call is cut.
    reentries: usize,
    /// The functions whose calls are under way where the interpretation
    /// is now.
    chain: Chain,
    /// How many expressions were interpreted, and how many may be.
    steps: u64,
    step_limit: u64,
    /// The addresses of the values that were escaped already. The
    /// values are kept, so that no other value gets one of the addresses.
    escaped: HashSet<usize>,
    kept: Vec<Thunk>,
    kept_closures: Vec<Rc<Closure>>,
    /// The values that are still to be escaped.
    pending: Vec<(Pending, Option<Rc<str>>)>,
    /// Whether `settle` is at work.
    escaping: bool,
    /// Whether the code that is interpreted now is code of the library.
    in_library: bool,
    /// With `trace`: what made the escaping that is under way.
    origin: Option<Rc<str>>,
    /// Whether to count where calls are cut, for whoever works on this
    /// interpretation, and the counts.
    trace: bool,
    pub cuts: BTreeMap<String, usize>,
}

/// Where the arguments of a module are in the configuration.
pub const ARGUMENTS: [&str; 3] = ["config", "_module", "args"];

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
            problems: BTreeSet::new(),
            deep_reads: BTreeSet::new(),
            arguments: BTreeMap::new(),
            library: None,
            both_sides: std::env::var_os("TOUCHES_BOTH_SIDES").is_some(),
            reentries: std::env::var("TOUCHES_REENTRIES")
                .ok()
                .and_then(|text| text.parse().ok())
                .unwrap_or(REENTRIES),
            chain: Chain::default(),
            steps: 0,
            step_limit: std::env::var("TOUCHES_STEPS")
                .ok()
                .and_then(|text| text.parse().ok())
                .unwrap_or(STEPS),
            escaped: HashSet::new(),
            kept: Vec::new(),
            kept_closures: Vec::new(),
            pending: Vec::new(),
            escaping: false,
            in_library: false,
            origin: None,
            trace: std::env::var_os("TOUCHES_TRACE").is_some(),
            cuts: BTreeMap::new(),
        }
    }

    /// Forget what the interpretation of the file before recorded.
    pub fn begin(&mut self) {
        self.reads.clear();
        self.deep_reads.clear();
        self.too_deep = 0;
        self.cycles = 0;
        self.fatal = None;
        self.steps = 0;
    }

    fn stop(&mut self, why: String) -> Value {
        if self.fatal.is_none() {
            self.fatal = Some(why);
        }
        Value::Unknown
    }

    pub fn force(&mut self, thunk: &Thunk) -> Value {
        let state = thunk.0.state.replace(State::Forcing);
        match state {
            State::Done(value) => {
                *thunk.0.state.borrow_mut() = State::Done(value.clone());
                value
            }
            State::Forcing => {
                self.cycles += 1;
                Value::Unknown
            }
            State::Delayed(work) => {
                let outside = std::mem::replace(&mut self.chain, thunk.0.chain.clone());
                let outside_library = std::mem::replace(&mut self.in_library, thunk.0.library);
                let value = work(self);
                let value = self.bounded(value);
                self.chain = outside;
                self.in_library = outside_library;
                *thunk.0.state.borrow_mut() = State::Done(value.clone());
                value
            }
        }
    }

    /// A value that is worked out when it is needed. `sees` is all that
    /// the work can use.
    fn delayed(&self, sees: Sees, work: impl Fn(&mut Interpreter) -> Value + 'static) -> Thunk {
        Thunk(Rc::new(Suspended {
            state: RefCell::new(State::Delayed(Rc::new(work))),
            reaches: sees.reaches(),
            chain: self.chain.clone(),
            sees,
            library: self.in_library,
        }))
    }

    fn delay(&self, expr: Option<ast::Expr>, env: &Env, file: &Rc<PathBuf>) -> Thunk {
        let library = self.of_library(file);
        let sees = Sees::env(env);
        let (env, file) = (env.clone(), file.clone());
        match expr {
            Some(expr) => Thunk(Rc::new(Suspended {
                state: RefCell::new(State::Delayed(Rc::new(move |interpreter| {
                    interpreter.eval(&expr, &env, &file)
                }))),
                reaches: sees.reaches(),
                chain: self.chain.clone(),
                sees,
                library,
            })),
            None => Thunk::done(Value::Unknown),
        }
    }

    /// Whether a file is one of the library.
    fn of_library(&self, file: &Path) -> bool {
        self.library
            .as_ref()
            .is_some_and(|library| file.starts_with(library))
    }

    /// Whether something that a delayed value can use is all of the
    /// configuration. See `sees_everything`.
    fn sees_everything_of(&mut self, sees: &Sees) -> bool {
        let known = |thunk: &Thunk| matches!(&*thunk.0.state.borrow(), State::Done(value) if is_everything(value));
        sees.values.iter().any(is_everything)
            || sees.thunks.iter().any(known)
            || sees
                .env
                .as_ref()
                .is_some_and(|env| sees_everything(self, env))
    }

    /// Record everything that can still be done with a value that the
    /// interpretation stops following: a value that goes to a function
    /// that is not known, into a result that is not tracked, or into a
    /// call that is cut. A part of the configuration is read with
    /// everything below it. A function is called with an argument that
    /// is not known, and its result is escaped. A set or a list is
    /// escaped entry by entry.
    ///
    /// A value that does not reach the configuration is left alone,
    /// because nothing that is done with it reads the configuration.
    ///
    /// The work is put aside here and done by `settle`, so that a value
    /// that holds values that hold values does not go deeper and deeper
    /// into this function.
    pub fn escape(&mut self, value: &Value) {
        if reaches(value) {
            self.pending
                .push((Pending::Value(value.clone()), self.origin.clone()));
        }
    }

    pub fn escape_thunk(&mut self, thunk: &Thunk) {
        if thunk.0.reaches && self.escaped.insert(Rc::as_ptr(&thunk.0) as usize) {
            self.pending
                .push((Pending::Thunk(thunk.clone()), self.origin.clone()));
        }
    }

    /// Do the escaping that was put aside, and what it puts aside in
    /// turn, until nothing is left.
    pub fn settle(&mut self) {
        if self.trace {
            eprintln!(
                "{} steps before the escaping, {} cuts, {} to escape",
                self.steps,
                self.too_deep,
                self.pending.len()
            );
        }
        self.escaping = true;
        while let Some((pending, origin)) = self.pending.pop() {
            if self.fatal.is_some() {
                break;
            }
            self.origin = origin;
            let before = self.steps;
            let label = match &pending {
                _ if !self.trace => String::new(),
                Pending::Value(Value::Lambda(closure)) => format!(
                    "function {}:{}",
                    closure.file.display(),
                    usize::from(closure.body.syntax().text_range().start())
                ),
                Pending::Value(_) => "value".to_string(),
                Pending::Thunk(thunk) if thunk.0.library => "value of the library".to_string(),
                Pending::Thunk(_) => "value of a module".to_string(),
            };
            match pending {
                Pending::Value(value) => self.escape_now(&value),
                Pending::Thunk(thunk) => {
                    // A value of the library that was not worked out is
                    // not worked out for this. Whatever the work does,
                    // it does with what it sees, and that is escaped.
                    // See the same step for a function.
                    let waiting = matches!(&*thunk.0.state.borrow(), State::Delayed(_));
                    if waiting && thunk.0.library && !self.sees_everything_of(&thunk.0.sees) {
                        if self.trace {
                            self.origin = Some("a value of the library".into());
                        }
                        let sees = thunk.0.sees.clone();
                        if let Some(env) = &sees.env {
                            self.escape_scopes(env);
                        }
                        for seen in &sees.thunks {
                            self.escape_thunk(seen);
                        }
                        for seen in &sees.values {
                            self.escape(seen);
                        }
                        self.kept.push(thunk);
                        continue;
                    }
                    let value = self.force(&thunk);
                    self.kept.push(thunk);
                    self.escape_now(&value);
                }
            }
            if self.trace && self.steps - before > 10_000 {
                *self.cuts.entry(format!("steps for {label}")).or_default() +=
                    (self.steps - before) as usize;
            }
        }
        self.escaping = false;
        self.origin = None;
    }

    fn escape_now(&mut self, value: &Value) {
        match value {
            Value::Config(path) => {
                if self.trace && path.len() <= 1 {
                    let origin = self.origin.as_deref().unwrap_or("the module");
                    *self
                        .cuts
                        .entry(format!("all of {} by {origin}", path.join(".")))
                        .or_default() += 1;
                }
                self.deep_reads.insert(path.as_ref().clone());
            }
            Value::Attrs(attrs) => {
                for field in attrs.fields.values() {
                    self.escape_thunk(field);
                }
                if let Some(any) = &attrs.any {
                    self.escape_thunk(any);
                }
            }
            Value::List(items) | Value::Seq(items) => {
                for item in items.iter() {
                    self.escape_thunk(item);
                }
            }
            Value::Either(values) => {
                for value in values.iter() {
                    self.escape(value);
                }
            }
            Value::Builtin(_, given) => {
                for argument in given.iter() {
                    self.escape_thunk(argument);
                }
            }
            Value::Lambda(closure) => {
                if closure.env.reaches() && self.escaped.insert(Rc::as_ptr(closure) as usize) {
                    self.kept_closures.push(closure.clone());
                    // A function of the library sees only what it was
                    // given. Whatever a call of it does, it does with
                    // that, so escaping what it sees records the call.
                    // That is far less work than a call with an
                    // argument that is not known, which takes every
                    // branch of the library function. A function of a
                    // module sees all of the configuration, so for it
                    // the same step would say that it reads everything,
                    // and it is called instead.
                    // A module that the library writes, as for an option
                    // that was renamed, is called like any module.
                    if self.of_library(&closure.file) && !sees_everything(self, &closure.env) {
                        if self.trace {
                            let start = usize::from(closure.body.syntax().text_range().start());
                            self.origin =
                                Some(format!("function {}:{start}", closure.file.display()).into());
                        }
                        self.escape_scopes(&closure.env);
                        return;
                    }
                    // Whoever calls the function is not known. The call
                    // goes on from the calls that were under way where
                    // the function was written down, so that a function
                    // that gives a function that gives the first one
                    // anew is a recursion and is cut.
                    let outside = std::mem::replace(&mut self.chain, closure.chain.clone());
                    let result = self.apply(value.clone(), Thunk::done(Value::Unknown));
                    self.chain = outside;
                    self.escape(&result);
                }
            }
            Value::Unknown
            | Value::Str(_)
            | Value::Path(_)
            | Value::Int(_)
            | Value::Bool(_)
            | Value::Null => {}
        }
    }

    /// A value that is one of more values than `ALTERNATIVES` is not
    /// followed as each of them. It becomes a value that is not known,
    /// and the values are escaped.
    fn bounded(&mut self, value: Value) -> Value {
        match &value {
            Value::Either(values) if values.len() > ALTERNATIVES => {
                if self.trace {
                    *self.cuts.entry("alternatives".to_string()).or_default() += 1;
                }
                self.too_deep += 1;
                self.forget(&[&value])
            }
            _ => value,
        }
    }

    /// A value that is not tracked, made from values that are escaped
    /// because of it.
    fn forget(&mut self, values: &[&Value]) -> Value {
        for value in values {
            self.escape(value);
        }
        Value::Unknown
    }

    /// End a call without making it. Whatever the call could have done
    /// with its argument, or with a name that the function can see, is
    /// recorded by escaping them, and the result is not known.
    fn cut(&mut self, closure: &Closure, argument: &Thunk) -> Value {
        self.too_deep += 1;
        if self.trace {
            let start = usize::from(closure.body.syntax().text_range().start());
            let why = if self.depth >= DEPTH {
                "deep"
            } else {
                "recursion"
            };
            *self
                .cuts
                .entry(format!("{why} {}:{start}", closure.file.display()))
                .or_default() += 1;
        }
        let outside = self.origin.clone();
        if self.trace {
            let start = usize::from(closure.body.syntax().text_range().start());
            self.origin = Some(format!("cut of {}:{start}", closure.file.display()).into());
        }
        self.escape_thunk(argument);
        self.escape_scopes(&closure.env);
        self.origin = outside;
        Value::Unknown
    }

    /// Escape every name that a function can see.
    fn escape_scopes(&mut self, env: &Env) {
        let mut scope = env.0.clone();
        while let Some(current) = scope {
            if !current.reaches {
                break;
            }
            let names: Vec<Thunk> = current.names.borrow().values().cloned().collect();
            for name in &names {
                self.escape_thunk(name);
            }
            if let Some(with) = &current.with {
                self.escape_thunk(with);
            }
            scope = current.parent.0.clone();
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
                // `with builtins;` gives only the names that are
                // builtins. Any other name comes from further out.
                if matches!(value, Value::Builtin("builtins", _))
                    && arity(name).is_none()
                    && !CONSTANTS.contains(&name)
                {
                    scope = current.parent.0.clone();
                    continue;
                }
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
            "true" => Value::Bool(true),
            "false" => Value::Bool(false),
            "null" => Value::Null,
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
                if path.iter().map(String::as_str).eq(ARGUMENTS)
                    && let Some(known) = self.arguments.get(name)
                {
                    return Some(known.clone());
                }
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
                // `arity` and `CONSTANTS` list the builtins of the
                // language. A name that is in neither does not exist,
                // which is an error in the code that asks for it. The
                // value of an error is the one of a `throw`.
                self.problems
                    .insert(format!("`builtins.{name}` does not exist"));
                nothing()
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
                    other => {
                        self.escape(&other);
                        None
                    }
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
                        // The text of the part is not known. Making the
                        // text reads all of the part.
                        Some(other) => {
                            self.escape(&other);
                            known = false;
                        }
                        None => known = false,
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
                    let sees = Sees::default().thunk(&existing).value(&below);
                    self.delayed(sees, move |interpreter| {
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
            _ => Some(self.delayed(
                Sees {
                    thunks: any.clone(),
                    ..Sees::default()
                },
                move |interpreter| {
                    let values = any.iter().map(|thunk| interpreter.force(thunk)).collect();
                    either(values)
                },
            )),
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
                        Some(self.delayed(
                            Sees::default().thunk(&first).thunk(&second),
                            move |interpreter| {
                                either(vec![interpreter.force(&first), interpreter.force(&second)])
                            },
                        ))
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
            (Value::Attrs(known), other) | (other, Value::Attrs(known)) => {
                let fields = if matches!(right, Value::Attrs(_)) {
                    known.fields.clone()
                } else {
                    // The unknown side is on the right, so an entry of
                    // it replaces the entry of the same name.
                    known
                        .fields
                        .iter()
                        .map(|(name, value)| {
                            let (value, other, field) =
                                (value.clone(), other.clone(), name.clone());
                            let both = self.delayed(
                                Sees::default().thunk(&value).value(&other),
                                move |interpreter| {
                                    let mut values = vec![interpreter.force(&value)];
                                    values.extend(interpreter.select_one(&other, &field));
                                    either(values)
                                },
                            );
                            (name.clone(), both)
                        })
                        .collect()
                };
                let (any, other) = (known.any.clone(), other.clone());
                let mut sees = Sees::default().value(&other);
                sees.thunks.extend(any.clone());
                let any = self.delayed(sees, move |interpreter| {
                    let mut values = vec![interpreter.any_entry(&other)];
                    values.extend(any.as_ref().map(|any| interpreter.force(any)));
                    either(values)
                });
                Value::Attrs(Rc::new(Attrs {
                    fields,
                    any: Some(any),
                }))
            }
            _ => self.forget(&[&left, &right]),
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
                        self.delayed(Sees::env(env).thunk(&from), move |interpreter| {
                            let from = interpreter.force(&from);
                            interpreter.select_one(&from, &name).unwrap_or_else(nothing)
                        })
                    }
                    None => {
                        let (env, name) = (env.clone(), name.clone());
                        self.delayed(Sees::env(&env), move |interpreter| {
                            interpreter.lookup(&name, &env)
                        })
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
        self.steps += 1;
        if self.steps > self.step_limit {
            return self.stop(format!(
                "the interpretation did not end within {} steps",
                self.step_limit
            ));
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
            ast::Expr::Literal(literal) => match literal.kind() {
                ast::LiteralKind::Integer(integer) => match integer.value() {
                    Ok(number) => Value::Int(number),
                    Err(_) => Value::Unknown,
                },
                ast::LiteralKind::Float(_) => Value::Unknown,
                ast::LiteralKind::Uri(uri) => Value::Str(uri.syntax().text().into()),
            },
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
            ast::Expr::List(list) => Value::Seq(Rc::new(
                list.items()
                    .map(|item| self.delay(Some(item), env, file))
                    .collect(),
            )),
            ast::Expr::AttrSet(set) => {
                if set.rec_token().is_some() {
                    let inner = env.child(false);
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
                let inner = env.child(false);
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
                let inner = env.child(false);
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
            // The condition of an assertion decides nothing here. It is
            // still read.
            ast::Expr::Assert(assert) => {
                if let Some(condition) = assert.condition() {
                    let condition = self.eval(&condition, env, file);
                    self.escape(&condition);
                }
                match assert.body() {
                    Some(body) => self.eval(&body, env, file),
                    None => Value::Unknown,
                }
            }
            // A condition that depends on an input decides nothing: the
            // result is both branches. The condition is still read,
            // because reading the configuration in it is a touch. A
            // condition that is known from what is written in the
            // source, such as a counter that has reached the length of
            // a list that is written out, has one answer.
            ast::Expr::IfElse(if_else) => {
                let condition = match if_else.condition() {
                    Some(condition) => self.eval(&condition, env, file),
                    None => Value::Unknown,
                };
                self.escape(&condition);
                let branches = match condition {
                    Value::Bool(true) if !self.both_sides => vec![if_else.body()],
                    Value::Bool(false) if !self.both_sides => vec![if_else.else_body()],
                    _ => vec![if_else.body(), if_else.else_body()],
                };
                let mut values = Vec::new();
                for branch in branches.into_iter().flatten() {
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
                    chain: self.chain.clone(),
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
                let (Some(base), Some(attrpath)) = (has.expr(), has.attrpath()) else {
                    return Value::Unknown;
                };
                let mut value = self.eval(&base, env, file);
                for attr in attrpath.attrs() {
                    let Some(name) = self.name_of(&attr, env, file) else {
                        // Asking for a name that is not known asks for
                        // any of the names.
                        self.any_entry(&value);
                        return Value::Unknown;
                    };
                    match self.has(&value, &name) {
                        Some(true) => {
                            value = self.select_one(&value, &name).unwrap_or(Value::Unknown);
                        }
                        Some(false) => return Value::Bool(false),
                        None => {
                            // Asking the configuration for a name reads it.
                            self.select_one(&value, &name);
                            return Value::Unknown;
                        }
                    }
                }
                Value::Bool(true)
            }
            ast::Expr::UnaryOp(op) => {
                let inner = match op.expr() {
                    Some(inner) => self.eval(&inner, env, file),
                    None => Value::Unknown,
                };
                match (op.operator(), inner) {
                    (Some(ast::UnaryOpKind::Invert), Value::Bool(known)) => Value::Bool(!known),
                    (Some(ast::UnaryOpKind::Negate), Value::Int(known)) => Value::Int(-known),
                    (Some(ast::UnaryOpKind::Invert | ast::UnaryOpKind::Negate) | None, inner) => {
                        self.forget(&[&inner])
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
                    // The second side of these is not looked at when the
                    // first side settles the result.
                    Some(ast::BinOpKind::And) if matches!(left, Value::Bool(false)) => {
                        return Value::Bool(false);
                    }
                    Some(ast::BinOpKind::Or) if matches!(left, Value::Bool(true)) => {
                        return Value::Bool(true);
                    }
                    Some(ast::BinOpKind::Implication) if matches!(left, Value::Bool(false)) => {
                        return Value::Bool(true);
                    }
                    _ => {}
                }
                let right = op
                    .rhs()
                    .map(|rhs| self.eval(&rhs, env, file))
                    .unwrap_or(Value::Unknown);
                let number = |known: Option<i64>| known.map_or(Value::Unknown, Value::Int);
                let truth = |known: Option<bool>| known.map_or(Value::Unknown, Value::Bool);
                // Every operator but `//` and `++` works its sides out
                // in full: a comparison looks at all of a value, and
                // the text of a string is all of it.
                if !matches!(
                    op.operator(),
                    Some(ast::BinOpKind::Update | ast::BinOpKind::Concat)
                ) {
                    self.escape(&left);
                    self.escape(&right);
                }
                match op.operator() {
                    Some(ast::BinOpKind::Update) => self.update(left, right),
                    Some(ast::BinOpKind::Concat) => self.concat(left, right),
                    Some(ast::BinOpKind::And) => match (&left, &right) {
                        // The first side is true or not known here.
                        (Value::Bool(true), Value::Bool(known)) => Value::Bool(*known),
                        (_, Value::Bool(false)) => Value::Bool(false),
                        _ => Value::Unknown,
                    },
                    Some(ast::BinOpKind::Or) => match (&left, &right) {
                        (Value::Bool(false), Value::Bool(known)) => Value::Bool(*known),
                        (_, Value::Bool(true)) => Value::Bool(true),
                        _ => Value::Unknown,
                    },
                    Some(ast::BinOpKind::Implication) => match (&left, &right) {
                        (Value::Bool(true), Value::Bool(known)) => Value::Bool(*known),
                        (_, Value::Bool(true)) => Value::Bool(true),
                        _ => Value::Unknown,
                    },
                    Some(ast::BinOpKind::Equal) => truth(self.equal(&left, &right)),
                    Some(ast::BinOpKind::NotEqual) => {
                        truth(self.equal(&left, &right).map(|same| !same))
                    }
                    Some(
                        operator @ (ast::BinOpKind::Sub
                        | ast::BinOpKind::Mul
                        | ast::BinOpKind::Div
                        | ast::BinOpKind::Less
                        | ast::BinOpKind::LessOrEq
                        | ast::BinOpKind::More
                        | ast::BinOpKind::MoreOrEq),
                    ) => match (&left, &right) {
                        (Value::Int(first), Value::Int(second)) => match operator {
                            ast::BinOpKind::Sub => number(first.checked_sub(*second)),
                            ast::BinOpKind::Mul => number(first.checked_mul(*second)),
                            ast::BinOpKind::Div => number(first.checked_div(*second)),
                            ast::BinOpKind::Less => Value::Bool(first < second),
                            ast::BinOpKind::LessOrEq => Value::Bool(first <= second),
                            ast::BinOpKind::More => Value::Bool(first > second),
                            _ => Value::Bool(first >= second),
                        },
                        _ => Value::Unknown,
                    },
                    Some(ast::BinOpKind::Add) => match (&left, &right) {
                        (Value::Int(first), Value::Int(second)) => {
                            number(first.checked_add(*second))
                        }
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

    /// Whether a value has an attribute, when that is known.
    fn has(&mut self, value: &Value, name: &str) -> Option<bool> {
        match value {
            Value::Attrs(attrs) => {
                if attrs.fields.contains_key(name) {
                    Some(true)
                } else if attrs.any.is_none() {
                    Some(false)
                } else {
                    None
                }
            }
            Value::Unknown | Value::Either(_) | Value::Config(_) => None,
            // A value of another kind has no attributes.
            _ => Some(false),
        }
    }

    /// Whether two values are equal, when that is known.
    fn equal(&mut self, first: &Value, second: &Value) -> Option<bool> {
        let (first_kind, second_kind) = (kind(first)?, kind(second)?);
        if first_kind != second_kind {
            return Some(false);
        }
        match (first, second) {
            (Value::Seq(first), Value::Seq(second)) => {
                if first.len() != second.len() {
                    return Some(false);
                }
                let mut all = Some(true);
                for (left, right) in first.iter().zip(second.iter()) {
                    let (left, right) = (self.force(left), self.force(right));
                    match self.equal(&left, &right) {
                        Some(false) => return Some(false),
                        Some(true) => {}
                        None => all = None,
                    }
                }
                all
            }
            (Value::Attrs(first), Value::Attrs(second)) => {
                let closed = first.any.is_none() && second.any.is_none();
                let same_names = first.fields.keys().eq(second.fields.keys());
                if closed && !same_names {
                    Some(false)
                } else if closed && first.fields.is_empty() {
                    Some(true)
                } else {
                    None
                }
            }
            (Value::List(_), _) | (_, Value::List(_)) => None,
            (Value::Lambda(_) | Value::Builtin(..), _) => None,
            _ => Some(same_scalar(first, second)),
        }
    }

    /// The elements of a list that is known element for element.
    fn exact(&mut self, value: &Value) -> Option<Rc<Vec<Thunk>>> {
        match value {
            Value::Seq(items) => Some(items.clone()),
            _ => None,
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
            Value::List(items) | Value::Seq(items) => {
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
        if let (Value::Seq(first), Value::Seq(second)) = (&left, &right) {
            return Value::Seq(Rc::new(
                first.iter().chain(second.iter()).cloned().collect(),
            ));
        }
        let mut items = Vec::new();
        for side in [left, right] {
            match side {
                Value::List(list) | Value::Seq(list) => items.extend(list.iter().cloned()),
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
                // A call is cut when it goes too deep, and when it is a
                // recursion: the function is already under way where
                // the call is made, or was under way where this
                // function was written down, more often than allowed.
                // The second is a function that makes itself anew, as
                // in a list that is made as it is walked.
                let identity = closure.identity();
                let under_way = self
                    .chain
                    .count(identity)
                    .max(closure.chain.count(identity));
                if self.depth >= DEPTH || under_way > self.reentries {
                    return self.cut(&closure, &argument);
                }
                // While values are escaped, a function of the library
                // that is given an argument that is not known is not
                // called. It reads what it sees, so escaping that says
                // as much as the call. The call would take every branch
                // of the library function, which for a function such as
                // `evalModules` is more work than everything else
                // together. A function that sees all of the
                // configuration is called all the same, because
                // escaping all of the configuration says that
                // everything is read.
                if self.escaping
                    && self.of_library(&closure.file)
                    && matches!(self.force(&argument), Value::Unknown)
                    && !sees_everything(self, &closure.env)
                {
                    return self.cut(&closure, &argument);
                }
                // A recursion with an argument that is not known finds
                // nothing that the call around it does not find, and
                // takes every branch again at each level.
                if under_way > 0 && matches!(self.force(&argument), Value::Unknown) {
                    return self.cut(&closure, &argument);
                }
                let mut names = BTreeMap::new();
                let library = self.of_library(&closure.file);
                let outside_library = std::mem::replace(&mut self.in_library, library);
                let inner = closure.env.call(argument.0.reaches);
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
                                self.delayed(
                                    Sees::env(&inner).thunk(&argument),
                                    move |interpreter| {
                                        let given = interpreter.force(&argument);
                                        let found = match &given {
                                            Value::Attrs(_)
                                            | Value::Either(_)
                                            | Value::Config(_) => {
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
                                    },
                                ),
                            );
                        }
                    }
                }
                for (name, value) in names {
                    inner.define(name, value);
                }
                let inside = self.chain.with(identity);
                let outside = std::mem::replace(&mut self.chain, inside);
                self.depth += 1;
                let result = self.eval(&closure.body, &inner, &closure.file);
                self.depth -= 1;
                self.chain = outside;
                self.in_library = outside_library;
                self.bounded(result)
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
            other => {
                // The function is not known, so it can do anything with
                // its argument.
                if self.trace && self.origin.is_none() {
                    self.origin = Some("a function that is not known".into());
                }
                self.escape(&other);
                self.escape_thunk(&argument);
                Value::Unknown
            }
        }
    }

    fn elements(&mut self, value: &Value) -> Vec<Thunk> {
        match value {
            Value::List(items) | Value::Seq(items) => items.as_ref().clone(),
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

    /// What a builtin gives when its arguments are known well enough for
    /// the standard meaning: the length of a list that is written out,
    /// whether a value is a set, the sum of two numbers. `None` when
    /// they are not, and then `builtin` gives the meaning for values
    /// that are not known.
    fn builtin_exact(&mut self, name: &str, arguments: &[Thunk]) -> Option<Value> {
        let first = self.force(&arguments[0]);
        let is = |wanted: &str| kind(&first).map(|known| Value::Bool(known == wanted));
        match name {
            "isAttrs" => is("set"),
            "isList" => is("list"),
            "isString" => is("string"),
            "isInt" => is("int"),
            "isBool" => is("bool"),
            "isNull" => is("null"),
            "isPath" => is("path"),
            "isFunction" => is("lambda"),
            "isFloat" => kind(&first).map(|_| Value::Bool(false)),
            "typeOf" => kind(&first).map(|known| Value::Str(known.into())),
            "length" => self
                .exact(&first)
                .map(|items| Value::Int(items.len() as i64)),
            "head" => {
                let items = self.exact(&first)?;
                Some(match items.first() {
                    Some(item) => self.force(item),
                    None => nothing(),
                })
            }
            "tail" => {
                let items = self.exact(&first)?;
                Some(Value::Seq(Rc::new(items.iter().skip(1).cloned().collect())))
            }
            "elemAt" => {
                let items = self.exact(&first)?;
                let Value::Int(index) = self.force(&arguments[1]) else {
                    return None;
                };
                Some(
                    match usize::try_from(index)
                        .ok()
                        .and_then(|index| items.get(index))
                    {
                        Some(item) => self.force(item),
                        None => nothing(),
                    },
                )
            }
            "elem" => {
                let list = self.force(&arguments[1]);
                let items = self.exact(&list)?;
                let mut known = true;
                for item in items.iter() {
                    let item = self.force(item);
                    match self.equal(&first, &item) {
                        Some(true) => return Some(Value::Bool(true)),
                        Some(false) => {}
                        None => known = false,
                    }
                }
                known.then_some(Value::Bool(false))
            }
            "genList" => {
                let Value::Int(count) = self.force(&arguments[1]) else {
                    return None;
                };
                if !(0..=10_000).contains(&count) {
                    return None;
                }
                let items = (0..count)
                    .map(|index| {
                        let function = first.clone();
                        self.delayed(Sees::default().value(&first), move |interpreter| {
                            interpreter.apply(function.clone(), Thunk::done(Value::Int(index)))
                        })
                    })
                    .collect();
                Some(Value::Seq(Rc::new(items)))
            }
            "map" => {
                let list = self.force(&arguments[1]);
                let items = self.exact(&list)?;
                let items = items
                    .iter()
                    .map(|item| {
                        let (function, item) = (first.clone(), item.clone());
                        let sees = Sees::default().value(&first).thunk(&item);
                        self.delayed(sees, move |interpreter| {
                            interpreter.apply(function.clone(), item.clone())
                        })
                    })
                    .collect();
                Some(Value::Seq(Rc::new(items)))
            }
            "filter" => {
                let list = self.force(&arguments[1]);
                let items = self.exact(&list)?;
                let mut kept = Vec::new();
                for item in items.iter() {
                    match self.apply(first.clone(), item.clone()) {
                        Value::Bool(true) => kept.push(item.clone()),
                        Value::Bool(false) => {}
                        _ => return None,
                    }
                }
                Some(Value::Seq(Rc::new(kept)))
            }
            "all" | "any" => {
                let list = self.force(&arguments[1]);
                let items = self.exact(&list)?;
                let wanted = name == "any";
                let mut known = true;
                for item in items.iter() {
                    match self.apply(first.clone(), item.clone()) {
                        Value::Bool(found) if found == wanted => return Some(Value::Bool(wanted)),
                        Value::Bool(_) => {}
                        _ => known = false,
                    }
                }
                known.then_some(Value::Bool(!wanted))
            }
            "concatLists" => {
                let lists = self.exact(&first)?;
                let mut items = Vec::new();
                for list in lists.iter() {
                    let list = self.force(list);
                    items.extend(self.exact(&list)?.iter().cloned());
                }
                Some(Value::Seq(Rc::new(items)))
            }
            "concatMap" => {
                let list = self.force(&arguments[1]);
                let given = self.exact(&list)?;
                let mut items = Vec::new();
                for item in given.iter() {
                    let produced = self.apply(first.clone(), item.clone());
                    items.extend(self.exact(&produced)?.iter().cloned());
                }
                Some(Value::Seq(Rc::new(items)))
            }
            "foldl'" => {
                let list = self.force(&arguments[2]);
                let items = self.exact(&list)?;
                let mut so_far = arguments[1].clone();
                for item in items.iter() {
                    let (function, before, item) = (first.clone(), so_far.clone(), item.clone());
                    so_far = self.delayed(
                        Sees::default().value(&first).thunk(&before).thunk(&item),
                        move |interpreter| {
                            let partly = interpreter.apply(function.clone(), before.clone());
                            interpreter.apply(partly, item.clone())
                        },
                    );
                }
                Some(self.force(&so_far))
            }
            "attrNames" | "attrValues" => match &first {
                Value::Attrs(attrs) if attrs.any.is_none() => {
                    let items = attrs
                        .fields
                        .iter()
                        .map(|(field, value)| {
                            if name == "attrNames" {
                                Thunk::done(Value::Str(field.as_str().into()))
                            } else {
                                value.clone()
                            }
                        })
                        .collect();
                    Some(Value::Seq(Rc::new(items)))
                }
                _ => None,
            },
            "hasAttr" => {
                let Value::Str(field) = &first else {
                    return None;
                };
                let set = self.force(&arguments[1]);
                self.has(&set, field).map(Value::Bool)
            }
            "intersectAttrs" => {
                let second = self.force(&arguments[1]);
                match (&first, &second) {
                    (Value::Attrs(names), Value::Attrs(values))
                        if names.any.is_none() && values.any.is_none() =>
                    {
                        let fields = values
                            .fields
                            .iter()
                            .filter(|(field, _)| names.fields.contains_key(*field))
                            .map(|(field, value)| (field.clone(), value.clone()))
                            .collect();
                        Some(Value::Attrs(Rc::new(Attrs { fields, any: None })))
                    }
                    _ => None,
                }
            }
            "functionArgs" => match &first {
                Value::Lambda(closure) => {
                    let fields = match &closure.param {
                        ast::Param::Pattern(pattern) => pattern
                            .pat_entries()
                            .filter_map(|entry| {
                                let name = entry.ident()?.syntax().text().to_string();
                                Some((name, Thunk::done(Value::Bool(entry.default().is_some()))))
                            })
                            .collect(),
                        ast::Param::IdentParam(_) => BTreeMap::new(),
                    };
                    Some(Value::Attrs(Rc::new(Attrs { fields, any: None })))
                }
                _ => None,
            },
            "concatStringsSep" => {
                let Value::Str(separator) = &first else {
                    return None;
                };
                let list = self.force(&arguments[1]);
                let items = self.exact(&list)?;
                let mut parts = Vec::new();
                for item in items.iter() {
                    match self.force(item) {
                        Value::Str(part) => parts.push(part.to_string()),
                        Value::Path(part) => parts.push(part.to_string_lossy().to_string()),
                        _ => return None,
                    }
                }
                Some(Value::Str(parts.join(separator).into()))
            }
            "stringLength" => match &first {
                Value::Str(text) => Some(Value::Int(text.len() as i64)),
                _ => None,
            },
            "substring" => {
                let (Value::Int(start), Value::Int(length), Value::Str(text)) =
                    (&first, self.force(&arguments[1]), self.force(&arguments[2]))
                else {
                    return None;
                };
                let start = usize::try_from(*start).ok()?.min(text.len());
                let end = match usize::try_from(length) {
                    Ok(length) => start.saturating_add(length).min(text.len()),
                    // A negative length is the rest of the string.
                    Err(_) => text.len(),
                };
                text.get(start..end).map(|part| Value::Str(part.into()))
            }
            "toString" => match &first {
                Value::Int(number) => Some(Value::Str(number.to_string().into())),
                Value::Bool(true) => Some(Value::Str("1".into())),
                Value::Bool(false) | Value::Null => Some(Value::Str("".into())),
                _ => None,
            },
            "add" | "sub" | "mul" | "div" | "lessThan" => {
                let (Value::Int(left), Value::Int(right)) = (&first, self.force(&arguments[1]))
                else {
                    return None;
                };
                match name {
                    "add" => left.checked_add(right).map(Value::Int),
                    "sub" => left.checked_sub(right).map(Value::Int),
                    "mul" => left.checked_mul(right).map(Value::Int),
                    "div" => left.checked_div(right).map(Value::Int),
                    _ => Some(Value::Bool(*left < right)),
                }
            }
            _ => None,
        }
    }

    /// The meaning of each builtin. One that gives a value that is not
    /// tracked, such as a number or the content of a file, gives
    /// `Unknown`. Its arguments are read first where they can read the
    /// configuration.
    fn builtin(&mut self, name: &'static str, arguments: &[Thunk]) -> Value {
        if let Some(exact) = self.builtin_exact(name, arguments) {
            return exact;
        }
        let first = arguments[0].clone();
        match name {
            "import" => match self.force(&first) {
                Value::Path(path) => self.import(&path),
                Value::Str(text) if text.starts_with('/') => self.import(Path::new(&*text)),
                other => self.forget(&[&other]),
            },
            "throw" | "abort" => {
                self.escape_thunk(&first);
                nothing()
            }
            "seq" => {
                self.force(&first);
                self.force(&arguments[1])
            }
            // The first argument is worked out in full, or printed.
            "deepSeq" | "trace" | "traceVerbose" | "warn" | "addErrorContext" => {
                self.escape_thunk(&first);
                self.force(&arguments[1])
            }
            "break"
            | "unsafeDiscardStringContext"
            | "unsafeDiscardOutputDependency"
            | "addDrvOutputDependencies" => self.force(&first),
            "toString" | "toPath" => match self.force(&first) {
                Value::Str(text) => Value::Str(text),
                Value::Path(path) => Value::Str(path.to_string_lossy().into()),
                other => self.forget(&[&other]),
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
                        let sees = Sees::default().value(&function).thunk(&item);
                        self.delayed(sees, move |interpreter| {
                            interpreter.apply(function.clone(), item.clone())
                        })
                    })
                    .collect();
                Value::List(Rc::new(items))
            }
            "genList" => {
                let function = self.force(&first);
                self.force(&arguments[1]);
                Value::List(Rc::new(vec![self.delayed(
                    Sees::default().value(&function),
                    move |interpreter| {
                        interpreter.apply(function.clone(), Thunk::done(Value::Unknown))
                    },
                )]))
            }
            // The function is called with the elements, and what it
            // gives decides something that is not tracked here.
            "filter" | "all" | "any" => {
                let function = self.force(&first);
                let list = self.force(&arguments[1]);
                let items = self.elements(&list);
                for item in &items {
                    let result = self.apply(function.clone(), item.clone());
                    self.escape(&result);
                }
                if name == "filter" {
                    Value::List(Rc::new(items))
                } else {
                    Value::Unknown
                }
            }
            "sort" => {
                let function = self.force(&first);
                let list = self.force(&arguments[1]);
                let items = self.elements(&list);
                let any = Thunk::done(self.any_entry(&list));
                let partly = self.apply(function, any.clone());
                let result = self.apply(partly, any);
                self.escape(&result);
                Value::List(Rc::new(items))
            }
            "partition" => {
                let function = self.force(&first);
                let list = self.force(&arguments[1]);
                for item in self.elements(&list) {
                    let result = self.apply(function.clone(), item);
                    self.escape(&result);
                }
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
                // An element here can stand for many elements of the
                // real list, so the real fold can go on from any of the
                // results. What it gives then is not followed.
                let so_far = either(results.clone());
                results.push(self.forget(&[&function, &so_far, &list]));
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
                            self.delayed(
                                Sees::default().value(&function).thunk(&value),
                                move |interpreter| {
                                    let partly = interpreter
                                        .apply(function.clone(), Thunk::done(name.clone()));
                                    interpreter.apply(partly, value.clone())
                                },
                            )
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
                            any: Some(self.delayed(
                                Sees::default().value(&function).thunk(&entry),
                                move |interpreter| {
                                    let partly = interpreter
                                        .apply(function.clone(), Thunk::done(Value::Unknown));
                                    interpreter.apply(partly, entry.clone())
                                },
                            )),
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
                        Some(other) => {
                            self.escape(&other);
                            None
                        }
                        None => None,
                    };
                    let value = self.delayed(Sees::default().value(&item), move |interpreter| {
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
                        other => {
                            // `groupBy` calls the function with the
                            // element to get the name, which is not
                            // tracked.
                            if name == "groupBy" {
                                let key = self.apply(function.clone(), Thunk::done(other.clone()));
                                self.escape(&key);
                            }
                            by_name.entry(None).or_default().push(Thunk::done(other));
                        }
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
                        self.delayed(
                            Sees::default().value(&function).thunk(&list),
                            move |interpreter| {
                                if zip {
                                    let key = match &given {
                                        Some(given) => Value::Str(given.as_str().into()),
                                        None => Value::Unknown,
                                    };
                                    let partly =
                                        interpreter.apply(function.clone(), Thunk::done(key));
                                    interpreter.apply(partly, list.clone())
                                } else {
                                    interpreter.force(&list)
                                }
                            },
                        ),
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
                let more = self.apply(operator.clone(), any);
                items.extend(self.elements(&more));
                // The real closure goes on from what the operator gave,
                // for as long as there is something new. That is not
                // followed, so what it could give is one more element
                // that is not known.
                items.push(Thunk::done(self.forget(&[&operator, &start, &more])));
                Value::List(Rc::new(items))
            }
            // The answer depends on what kind of value the argument is,
            // or on how many elements it has, and on nothing inside it.
            "isAttrs" | "isBool" | "isFloat" | "isFunction" | "isInt" | "isList" | "isNull"
            | "isPath" | "isString" | "typeOf" | "length" => {
                self.force(&first);
                Value::Unknown
            }
            "hasAttr" => {
                let set = self.force(&arguments[1]);
                match self.force(&first) {
                    Value::Str(name) => {
                        self.select_one(&set, &name);
                    }
                    other => {
                        self.escape(&other);
                        self.any_entry(&set);
                    }
                }
                Value::Unknown
            }
            // The builtins whose result is not tracked: numbers, tests,
            // strings that are computed, and what comes from files, the
            // store or the network. They can use all of their
            // arguments, so the arguments are escaped.
            "add" | "appendContext" | "attrsets" | "baseNameOf" | "bitAnd" | "bitOr" | "bitXor"
            | "ceil" | "compareVersions" | "concatStringsSep" | "convertHash" | "derivation"
            | "derivationStrict" | "dirOf" | "div" | "elem" | "fetchGit" | "fetchTarball"
            | "fetchTree" | "fetchurl" | "filterSource" | "findFile" | "flakeRefToString"
            | "floor" | "fromJSON" | "fromTOML" | "getContext" | "getEnv" | "getFlake"
            | "hasContext" | "hashFile" | "hashString" | "lessThan" | "match" | "mul"
            | "outputOf" | "parseDrvName" | "parseFlakeRef" | "path" | "pathExists"
            | "placeholder" | "readDir" | "readFile" | "readFileType" | "replaceStrings"
            | "scopedImport" | "split" | "splitVersion" | "storePath" | "stringLength" | "sub"
            | "substring" | "toFile" | "toJSON" | "toXML" | "unsafeGetAttrPos" => {
                for argument in arguments {
                    self.escape_thunk(argument);
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

pub fn string(text: &str) -> Value {
    Value::Str(text.into())
}
