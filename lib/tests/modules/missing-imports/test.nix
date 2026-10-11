let
  lib = import ../../..;

  evaluation = lib.evalModules {
    modules = [
      ./declares.nix
      ./defines-with-import.nix
      ./defines-through-another.nix
      ./defines-without-import.nix
      # A definition of an option that the module system declares
      # itself needs no import.
      { _module.args.unused = null; }
    ];
  };

  actual = lib.modules.missingImports evaluation;

  expected = [
    {
      option = "greeting";
      file = toString ./defines-without-import.nix;
      declaredIn = [ (toString ./declares.nix) ];
    }
  ];

  # `filesWithAnyDefinition` lists every file with a definition, also
  # the file whose definition is under a false condition.
  definingFiles = lib.sort lib.lessThan evaluation.options.greeting.filesWithAnyDefinition;

  expectedDefiningFiles = lib.sort lib.lessThan [
    (toString ./defines-with-import.nix)
    (toString ./defines-through-another.nix)
    (toString ./defines-without-import.nix)
  ];

  dependencies = lib.modules.moduleDependencies evaluation;

  # Every file that defines `greeting` without declaring it uses it,
  # also the files that import the declaration.
  usingFiles = lib.sort lib.lessThan (map (use: use.file) dependencies.uses);
in
assert actual == expected;
assert usingFiles == expectedDefiningFiles;
assert
  dependencies.imports.${toString ./defines-through-another.nix} == [
    (toString ./defines-with-import.nix)
  ];
assert dependencies.imports.${toString ./declares.nix} == [ ];
# The module that is written in the list of modules has no path.
assert
  lib.sort lib.lessThan dependencies.importable == lib.sort lib.lessThan [
    (toString ./declares.nix)
    (toString ./defines-with-import.nix)
    (toString ./defines-through-another.nix)
    (toString ./defines-without-import.nix)
  ];
assert
  dependencies.declarations == {
    greeting = {
      files = [ (toString ./declares.nix) ];
      withDescription = [ (toString ./declares.nix) ];
    };
  };
assert definingFiles == expectedDefiningFiles;
null
