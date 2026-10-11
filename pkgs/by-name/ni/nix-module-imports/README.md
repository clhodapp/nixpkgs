# nix-module-imports

`nix-module-imports` finds, writes and checks the imports that the modules of a tree need.
It works for any project that uses the module system in `lib.modules`.
It knows nothing about NixOS or about any other project.

## The problem

A module that sets or reads an option of another module needs the module that declares the option.
Many module trees do not write that down.
They hand one long list of all modules to `lib.evalModules`, and each module relies on the rest of the list.
Such a module cannot be used on its own, and nothing shows which other modules it needs.

## How it works

The module system reports what an evaluation shows.
`lib.modules.moduleDependencies` takes the result of `lib.evalModules` and lists, for every module file, the options of other files that the file sets.
It also lists the files that each file imports and the files that declare each option.

The tool reads that report as JSON.
It also reads the source of each module file to find the options that the file only reads, such as `config.services.nginx.enable`.
From both it works out the files that each module file needs, and it compares them with the imports in the file.

The imports that the tool writes go between two comments in the `imports` list:

```nix
{
  imports = [
    ./written-by-a-person.nix
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../config/users-groups.nix
    ../system/boot/systemd.nix
    # END generated imports
  ];
}
```

The tool owns the lines between the two comments and replaces them on each run.
An import that is no longer needed goes away on the next run.
The tool does not touch the imports outside the two comments, and it does not repeat one of them between the comments.

Where several files declare one option, the tool imports the file whose declaration has the description.
The other declarations add to that one.

The tool writes no import of a file that the module reaches through another import.
When module A uses options of B and of C, and B imports C, the tool writes only the import of B into A.
A later change that takes C out of the reach of B puts the import of C into A on the next run.
Every import that the tool writes is of a file whose options the module uses.

## The smallest set of imports

The imports that the tool writes are the smallest set with which every module still reaches every file it needs.
In terms of graphs this is the transitive reduction of the import graph, where the imports that a person wrote cost nothing.

The tool first puts the modules into groups that reach each other, the strongly connected components of the graph.
Between the groups the graph has no cycle.
There the smallest set is unique: one import from a group to another, and only where no way leads through a third group.

Inside a group the task is the minimum strongly connected spanning subgraph, which is NP-hard in general.
The groups of a module tree are small enough to solve.
The tool first keeps every import that is the only one to enter or to leave a module, and drops every import of a file that those reach.
It solves the rest as an integer program: one import has to leave and one has to enter each set of modules, and the tool adds that condition for a set whenever an answer leaves the set cut off.
The first answer in which all modules reach each other is a smallest set.

`--smallest-set-timeout` gives about the most seconds that the tool spends on finding the smallest sets, for all groups together.
The default is 300.
The tool looks at the clock between the steps of its work, so a run can go a little over the time.
The small groups come first.
The timeout does not end the run: when the time is up, the tool builds a set for each group that is left from what the search has found out.
The imports that the search has decided on stay decided.
The last answer of the integer program is as small as any valid set, and it leaves some modules cut off.
The tool takes the imports outside that answer out one at a time first, and keeps one only when its file would go out of reach.
The result has no import that could be left out, and it is close to the smallest set: for the largest group of NixOS, 5 seconds give 445 imports where the smallest set has 443.
With `--smallest-set-timeout 0` no program is solved, and the tool takes the imports of every group out one at a time in the order of their paths.

`report` prints the number of imports inside the groups next to the number that no set is smaller than.
The two are equal when every group has its smallest set.
Every command says so when the time was up before that.

How far the tool comes in a given time depends on the machine.
`check` compares the files with what the tool writes, so a `check` that reaches the timeout can ask for a rewrite of files that `fix` wrote on a faster machine.
Give `check` the time to finish, or run both with `--smallest-set-timeout 0`.

A tree often has several smallest sets.
The tool gives the same tree the same imports on every run.
Another version of the tool can choose another smallest set.

## Usage

Write the report of an evaluation to a file:

```ShellSession
$ nix-instantiate --eval --strict --json \
    --expr '(import <nixpkgs/lib>).modules.moduleDependencies (import ./evaluation.nix)' \
    > dependencies.json
```

Describe what the modules need:

```ShellSession
$ nix-module-imports report --dependencies dependencies.json --root .
```

Write the imports into the module files:

```ShellSession
$ nix-module-imports fix --dependencies dependencies.json --root .
```

Fail when a module file lacks an import, without changing a file:

```ShellSession
$ nix-module-imports check --dependencies dependencies.json --root .
```

`--dependencies` can be given several times.
One evaluation shows only the modules that are in it, so a tree whose modules are spread over several evaluations needs a report of each.

`--evaluated-root` names the directory as the evaluation saw it, when the evaluation read the modules from another place than `--root`, such as the Nix store.

`--definitions-only` leaves out the options that a module only reads.

`report --json` prints everything the tool worked out, with the options behind each import.

Run the formatter of the project after `fix`.
In a few files `fix` puts `++ [ ... ]` after a computed `imports` value, and the formatter may lay out that value in another way.
The result of the formatter is stable: `fix` and `check` find the list again.

`testers.moduleImports` in Nixpkgs runs `check` as a derivation, for use in continuous integration.

## Files the tool does not rewrite

The tool rewrites a module file when the module is an attribute set, or a function, `let`, `with` or `assert` around one.
The value of `imports` can be a list, a chain of `++`, a `let` or `with` around those, or any other expression, which gets `++ [ ... ]` after it.

The tool reports every other file with the reason and the imports it needs, and `fix` and `check` then fail.
Two shapes are reported:

- The module is something else than an attribute set, for example `lib.mkIf condition { ... }`.
- `imports` is set with `inherit`.

A person writes the imports of such a file by hand, in whatever way the file needs.
The evaluation then shows the imports, and the tool accepts the file.

## Limits

- The tool knows a module by its file.
  A module that is written inside another file counts as that file.
- The tool writes no import of a file outside `--root`, and none of a file that is not imported by its path, such as a file that `lib.modules.importApply` loads.
  `report` lists those uses.
- The imports of one file depend on the imports of other files, because a file that is in reach needs no import.
  A change in one module can therefore change the generated imports of another module.
- The search for the options that a module reads looks at the source and does not evaluate it.
  It follows a name that a `let` gives to a part of `config`, and `inherit (config.a) b;`.
  It finds nothing under `with config;`, and nothing where a part of the path is computed.
- A tree with complete imports has modules that import each other.
  The module system evaluates such a tree since it visits each module once when it collects the imports.
