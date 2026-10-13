{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.hardware.flirc;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.flirc.enable = lib.mkEnableOption "software to configure a Flirc USB device";

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.flirc ];
    services.udev.packages = [ pkgs.flirc ];
  };
}
