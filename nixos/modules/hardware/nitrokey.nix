{
  config,
  lib,
  pkgs,
  ...
}:
let

  cfg = config.hardware.nitrokey;

in

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.nitrokey = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Enables udev rules for Nitrokey devices.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    services.udev.packages = [ pkgs.nitrokey-udev-rules ];
  };
}
