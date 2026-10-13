{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../system/boot/kernel.nix
    ../sane.nix
    ../udev.nix
    # END generated imports
  ];

  options = {

    hardware.sane.dsseries.enable = lib.mkEnableOption "Brother DSSeries scan backend" // {
      description = ''
        When enabled, will automatically register the "dsseries" SANE backend.

        This supports the Brother DSmobile scanner series, including the
        DS-620, DS-720D, DS-820W, and DS-920DW scanners.
      '';
    };
  };

  config = lib.mkIf (config.hardware.sane.enable && config.hardware.sane.dsseries.enable) {

    hardware.sane.extraBackends = [ pkgs.dsseries ];
    services.udev.packages = [ pkgs.dsseries ];
    boot.kernelModules = [ "sg" ];

  };
}
