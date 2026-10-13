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
    ../../system/boot/kernel.nix
    ../filesystems.nix
    # END generated imports
  ];

  config = mkIf (config.boot.supportedFilesystems.exfat or false) {
    system.fsPackages =
      if config.boot.kernelPackages.kernelOlder "5.7" then
        [
          pkgs.exfat # FUSE
        ]
      else
        [
          pkgs.exfatprogs # non-FUSE
        ];
  };
}
