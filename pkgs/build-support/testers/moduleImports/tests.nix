# Run:
#   nix-build -A tests.testers.moduleImports

{
  lib,
  testers,
}:
let
  evaluate =
    root:
    lib.evalModules {
      modules = [
        (root + "/greeting.nix")
        (root + "/message.nix")
        (root + "/polite.nix")
      ];
    };

  # The last lines of the log. The log that the tester compares with has
  # no line break at its end.
  writtenByHand = lib.concatStringsSep "\n" [
    "1 files need their imports written by hand:"
    "  polite.nix: the module is not written as an attribute set, or as a function, `let` or `with` around one"
    "    needs ./greeting.nix"
  ];
in
lib.recurseIntoAttrs {
  # Every module imports the module whose option it uses.
  complete = testers.moduleImports {
    name = "complete";
    root = ./src/complete;
    evaluations = [ (evaluate ./src/complete) ];
  };

  # `message.nix` reads `greeting` without importing its declaration, and
  # the tool can write that import. `polite.nix` sets `greeting`, and it is
  # not written as an attribute set, so a person has to write its import.
  incomplete = testers.testBuildFailure' {
    drv = testers.moduleImports {
      name = "incomplete";
      root = ./src/incomplete;
      evaluations = [ (evaluate ./src/incomplete) ];
    };
    expectedBuilderLogEntries = [
      ''
        1 files do not have the generated imports they need:
          message.nix
      ''
      writtenByHand
    ];
  };

  # With `definitionsOnly`, the option that `message.nix` reads needs no
  # import.
  incomplete-definitions-only = testers.testBuildFailure' {
    drv = testers.moduleImports {
      name = "incomplete-definitions-only";
      root = ./src/incomplete;
      evaluations = [ (evaluate ./src/incomplete) ];
      definitionsOnly = true;
    };
    expectedBuilderLogEntries = [ writtenByHand ];
  };
}
