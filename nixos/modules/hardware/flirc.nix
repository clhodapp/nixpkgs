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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.flirc.enable = lib.mkEnableOption "software to configure a Flirc USB device";

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.flirc ];
    services.udev.packages = [ pkgs.flirc ];
  };
}
