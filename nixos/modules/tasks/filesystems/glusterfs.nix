{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../filesystems.nix
    # END generated imports
  ];

  config = mkIf (config.boot.supportedFilesystems.glusterfs or false) {

    system.fsPackages = [ pkgs.glusterfs ];

  };
}
