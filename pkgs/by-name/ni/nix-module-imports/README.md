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

Modules that use each other are handled one import at a time.
The tool leaves an import out only while the file stays in reach, so two modules that import each other cannot each leave a file to the other.
Where several choices are possible, the tool goes through the modules and their imports in the order of their paths, so the same tree always gets the same imports.

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
