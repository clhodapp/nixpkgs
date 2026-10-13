{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../programs/fuse.nix
    ../filesystems.nix
    # END generated imports
  ];

  config =
    lib.mkIf
      (config.boot.supportedFilesystems.sshfs or config.boot.supportedFilesystems."fuse.sshfs" or false)
      {
        programs.fuse.enable = true;

        system.fsPackages = [ pkgs.sshfs ];
      };
}
