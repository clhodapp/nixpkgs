{
  config,
  lib,
  pkgs,
  ...
}:
let
  kernelVersion = config.boot.kernelPackages.kernel.version;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../services/hardware/udev.nix
    # END generated imports
  ];

  ###### interface

  options = {

    networking.enableB43Firmware = lib.mkOption {
      default = false;
      type = lib.types.bool;
      description = ''
        Turn on this option if you want firmware for the NICs supported by the b43 module.
      '';
    };

  };

  ###### implementation

  config = lib.mkIf config.networking.enableB43Firmware {
    hardware.firmware = [ pkgs.b43Firmware_5_1_138 ];
  };

}
