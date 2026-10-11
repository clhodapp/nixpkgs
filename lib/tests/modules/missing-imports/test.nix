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
in
assert actual == expected;
assert definingFiles == expectedDefiningFiles;
null
