{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
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
