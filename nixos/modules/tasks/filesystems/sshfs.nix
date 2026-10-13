{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../programs/fuse.nix
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
