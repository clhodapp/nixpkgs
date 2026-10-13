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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
