{ lib, config, ... }:

let
  cfg = config.hardware.hid-fanatecff;
  inherit (config.boot.kernelPackages) hid-fanatecff;
  inherit (lib) maintainers mkEnableOption mkIf;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../services/hardware/udev.nix
    ../system/boot/kernel.nix
    # END generated imports
  ];

  options.hardware.hid-fanatecff = {
    enable = mkEnableOption "hid-fanatecff, a Linux kernel driver that aims to add support for Fanatec devices";
  };

  config = mkIf cfg.enable {
    boot.extraModulePackages = [ hid-fanatecff ];
    services.udev.packages = [ hid-fanatecff ];
  };

  meta.maintainers = with maintainers; [ rake5k ];
}
