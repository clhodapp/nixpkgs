{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../services/hardware/udev.nix
    # END generated imports
  ];

  ###### interface

  options = {

    networking.enableIntel2200BGFirmware = lib.mkOption {
      default = false;
      type = lib.types.bool;
      description = ''
        Turn on this option if you want firmware for the Intel
        PRO/Wireless 2200BG to be loaded automatically.  This is
        required if you want to use this device.
      '';
    };

  };

  ###### implementation

  config = lib.mkIf config.networking.enableIntel2200BGFirmware {

    hardware.firmware = [ pkgs.ipw2200-firmware ];

  };

}
