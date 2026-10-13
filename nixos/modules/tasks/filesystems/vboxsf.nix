{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let

  inInitrd = config.boot.initrd.supportedFilesystems.vboxsf or false;

  package = pkgs.runCommand "mount.vboxsf" { preferLocalBuild = true; } ''
    mkdir -p $out/bin
    cp ${config.virtualisation.virtualbox.guest.package}/bin/mount.vboxsf $out/bin
  '';
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/kernel.nix
    ../../virtualisation/virtualbox-guest.nix
    ../filesystems.nix
    # END generated imports
  ];

  config = mkIf (config.boot.supportedFilesystems.vboxsf or false) {

    system.fsPackages = [ package ];

    boot.initrd.kernelModules = mkIf inInitrd [ "vboxsf" ];

  };
}
