{
  config,
  lib,
  pkgs,
  ...
}:
let

  cfg = config.hardware.flipperzero;

in

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.flipperzero.enable = lib.mkEnableOption "udev rules and software for Flipper Zero devices";

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.qFlipper ];
    services.udev.packages = [ pkgs.qFlipper ];
  };
}
