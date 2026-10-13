{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../filesystems.nix
    # END generated imports
  ];

  config = lib.mkIf (config.boot.supportedFilesystems."fuse.bindfs" or false) {
    system.fsPackages = [ pkgs.bindfs ];
  };

  meta = {
    maintainers = with lib.maintainers; [ Luflosi ];
  };
}
