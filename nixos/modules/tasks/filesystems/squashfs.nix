{ config, lib, ... }:

let

  inInitrd = config.boot.initrd.supportedFilesystems.squashfs or false;

in

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/kernel.nix
    # END generated imports
  ];

  boot.initrd.availableKernelModules = lib.mkIf inInitrd [ "squashfs" ];

}
