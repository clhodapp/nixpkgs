{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.stratis;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.services.stratis = {
    enable = lib.mkEnableOption "Stratis Storage - Easy to use local storage management for Linux";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.stratis-cli ];
    systemd.packages = [ pkgs.stratisd ];
    services.dbus.packages = [ pkgs.stratisd ];
    services.udev.packages = [ pkgs.stratisd ];
    systemd.services.stratisd.wantedBy = [ "sysinit.target" ];
  };
}
