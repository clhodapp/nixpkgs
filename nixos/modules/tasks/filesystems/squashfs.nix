{ config, lib, ... }:

let

  inInitrd = config.boot.initrd.supportedFilesystems.squashfs or false;

in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/kernel.nix
    # END generated imports
  ];

  boot.initrd.availableKernelModules = lib.mkIf inInitrd [ "squashfs" ];

}
