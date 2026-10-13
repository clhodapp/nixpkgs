{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  inInitrd = config.boot.initrd.supportedFilesystems.jfs or false;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/kernel.nix
    # END generated imports
  ];

  config = mkIf (config.boot.supportedFilesystems.jfs or false) {

    system.fsPackages = [ pkgs.jfsutils ];

    boot.initrd.kernelModules = mkIf inInitrd [ "jfs" ];

    boot.initrd.extraUtilsCommands = mkIf (inInitrd && !config.boot.initrd.systemd.enable) ''
      copy_bin_and_libs ${lib.getBin pkgs.jfsutils}/sbin/fsck.jfs
    '';

    boot.initrd.systemd.initrdBin = mkIf inInitrd [ pkgs.jfsutils ];
  };
}
