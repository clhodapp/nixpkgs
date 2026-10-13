{ lib, config, ... }:
let
  facterLib = import ../lib.nix lib;

  inherit (config.hardware.facter) report;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../.
    ../../../system/boot/initrd-network.nix
    ../../../system/boot/kernel.nix
    # END generated imports
  ];

  options.hardware.facter.detected.boot.initrd.networking.kernelModules = lib.mkOption {
    type = lib.types.listOf lib.types.str;
    default = lib.uniqueStrings (facterLib.collectDrivers (report.hardware.network_controller or [ ]));
    defaultText = "hardware dependent";
    description = ''
      List of kernel modules to include in the initrd to support networking.
    '';
  };

  config = lib.mkIf (config.hardware.facter.enable && config.boot.initrd.network.enable) {
    boot.initrd.kernelModules = config.hardware.facter.detected.boot.initrd.networking.kernelModules;
  };
}
