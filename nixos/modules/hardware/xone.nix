{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.hardware.xone;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../services/hardware/udev.nix
    ../system/boot/kernel.nix
    ../system/boot/modprobe.nix
    ./xpad-noone.nix
    # END generated imports
  ];

  options.hardware.xone = {
    enable = lib.mkEnableOption "the xone driver for Xbox One and Xbox Series X|S accessories";
  };

  config = lib.mkIf cfg.enable {
    boot = {
      blacklistedKernelModules = [
        "xpad"
        "mt76x2u"
      ];
      extraModulePackages = with config.boot.kernelPackages; [ xone ];
    };
    hardware.firmware = [ pkgs.xone-dongle-firmware ];
    hardware.xpad-noone.enable = lib.mkDefault true;
  };

  meta = {
    maintainers = with lib.maintainers; [ rhysmdnz ];
  };
}
