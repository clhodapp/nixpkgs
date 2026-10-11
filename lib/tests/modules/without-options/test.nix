let
  lib = import ../../..;

  evaluation = lib.evalModules {
    modules = [
      ./declares.nix
      {
        options.farewell = lib.mkOption {
          type = lib.types.str;
          # Reading the default of this option fails. Taking options
          # out of the tree does not read that far.
          default = throw "the default of farewell is not read";
        };
      }
    ];
  };

  without = files: lib.options.withoutOptionsDeclaredOnlyIn files evaluation.options;
in
# The option that is declared only in the given file is taken out.
assert !((without [ ./declares.nix ]) ? greeting);
assert (without [ ./declares.nix ]) ? farewell;
# The file can be given as a string as well.
assert !((without [ (toString ./declares.nix) ]) ? greeting);
# An option that is declared in another file stays.
assert (without [ ./test.nix ]) ? greeting;
assert (without [ ]) ? greeting;
# The options of the module system, which are nested, stay.
assert (without [ ./declares.nix ])._module ? args;
# The option that is written in the list of modules has no file of its
# own, and it can be taken out by the name that such modules have.
assert
  builtins.attrNames (without [ "<unknown-file>" ]) == [
    "_module"
    "greeting"
  ];
null
