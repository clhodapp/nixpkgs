{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.hardware.keyboard.teck;
  inherit (lib) mkEnableOption mkIf;

in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.keyboard.teck = {
    enable = mkEnableOption "non-root access to the firmware of TECK keyboards";
  };

  config = mkIf cfg.enable {
    services.udev.packages = [ pkgs.teck-udev-rules ];
  };
}
