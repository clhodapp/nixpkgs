{ lib, config, ... }:
let
  facterLib = import ./lib.nix lib;

  inherit (config.hardware.facter) report;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/kernel.nix
    ./.
    # END generated imports
  ];

  options.hardware.facter.detected.boot.disk.kernelModules = lib.mkOption {
    type = lib.types.listOf lib.types.str;
    default = lib.uniqueStrings (
      facterLib.collectDrivers (
        # A disk might be attached.
        (report.hardware.firewire_controller or [ ])
        # definitely important
        ++ (report.hardware.disk or [ ])
        ++ (report.hardware.storage_controller or [ ])
      )
    );
    defaultText = "hardware dependent";
    description = ''
      List of kernel modules that are needed to access the disk.
    '';
  };

  config = lib.mkIf config.hardware.facter.enable {
    boot.initrd.availableKernelModules = config.hardware.facter.detected.boot.disk.kernelModules;
  };
}
