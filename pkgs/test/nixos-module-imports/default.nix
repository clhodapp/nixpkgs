# The modules of NixOS import the modules that declare what they refer to.
#
# `nix-module-imports check` reads the source of the modules and fails when a module
# does not have the generated imports that `nix-module-imports fix` writes, or refers
# to an option that no module declares. After a change of a module, run
#
#     nix-module-imports fix \
#       --library lib \
#       --special-arg modulesPath=$PWD/nixos/modules \
#       --module-list nixos/modules/module-list.nix \
#       --tree nixos/modules
#
# in the root of the tree.
{
  lib,
  runCommand,
  nix-module-imports,
}:

let
  tree = lib.fileset.toSource {
    root = ../../..;
    fileset = lib.fileset.unions [
      ../../../lib
      ../../../modules
      ../../../nixos
    ];
  };
in
runCommand "nixos-module-imports"
  {
    nativeBuildInputs = [ nix-module-imports ];
  }
  ''
    nix-module-imports check \
      --library ${tree}/lib \
      --special-arg modulesPath=${tree}/nixos/modules \
      --module-list ${tree}/nixos/modules/module-list.nix \
      --tree ${tree}/nixos/modules
    touch $out
  ''
