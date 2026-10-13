# To build, use:
# nix-build nixos -I nixos-config=nixos/modules/installer/sd-card/sd-image-x86_64.nix -A config.system.build.sdImage

# This image is primarily used in NixOS tests (boot.nix) to test `boot.loader.generic-extlinux-compatible`.
{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    ../../profiles/base.nix
    ./sd-image.nix
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/activation/top-level.nix
    ../../system/boot/kernel.nix
    ../../system/boot/loader/generic-extlinux-compatible
    ../../system/boot/loader/grub/grub.nix
    ../../system/build.nix
    # END generated imports
  ];

  boot.loader = {
    grub.enable = false;
    generic-extlinux-compatible.enable = true;
  };

  boot.consoleLogLevel = lib.mkDefault 7;

  sdImage = {
    populateFirmwareCommands = "";
    populateRootCommands = ''
      mkdir -p ./files/boot
      ${config.boot.loader.generic-extlinux-compatible.populateCmd} -c ${config.system.build.toplevel} -d ./files/boot
    '';
  };
}
