{
  lib,
  config,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../system/boot/systemd.nix
    ./version.nix
    # END generated imports
  ];

  _module.args = {
    utils = import ../../lib/utils.nix { inherit lib config pkgs; };
  };
}
