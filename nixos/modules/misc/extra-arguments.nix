{
  lib,
  config,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./version.nix
    # END generated imports
  ];

  _module.args = {
    utils = import ../../lib/utils.nix { inherit lib config pkgs; };
  };
}
