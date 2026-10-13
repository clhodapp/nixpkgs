{
  config,
  lib,
  pkgs,
  ...
}:

{

  imports = [
    (lib.mkRemovedOptionModule [ "programs" "oblogout" ]
      "programs.oblogout has been removed from NixOS. This is because the oblogout repository has been archived upstream."
    )
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/assertions.nix
    # END generated imports
  ];

}
