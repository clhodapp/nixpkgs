# blueman service
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.blueman;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../system/boot/systemd.nix
    ../system/dbus.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.blueman = {
      enable = lib.mkEnableOption "blueman, a bluetooth manager";
    };
  };

  ###### implementation
  config = lib.mkIf cfg.enable {

    environment.systemPackages = [ pkgs.blueman ];

    services.dbus.packages = [ pkgs.blueman ];

    systemd.packages = [ pkgs.blueman ];
  };
}
