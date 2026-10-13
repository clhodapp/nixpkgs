{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../filesystems.nix
    # END generated imports
  ];

  config = mkIf (config.boot.supportedFilesystems.glusterfs or false) {

    system.fsPackages = [ pkgs.glusterfs ];

  };
}
