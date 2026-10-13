# Zeitgeist
{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../system/boot/systemd.nix
    ../system/dbus.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.pantheon ];
  };

  ###### interface

  options = {
    services.zeitgeist = {
      enable = lib.mkEnableOption "zeitgeist, a service which logs the users' activities and events";
    };
  };

  ###### implementation

  config = lib.mkIf config.services.zeitgeist.enable {

    environment.systemPackages = [ pkgs.zeitgeist ];

    services.dbus.packages = [ pkgs.zeitgeist ];

    systemd.packages = [ pkgs.zeitgeist ];
  };
}
