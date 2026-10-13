# GNOME Online Accounts daemon.

{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../../lib/modules/generic/meta-maintainers.nix
    ../../../config/system-path.nix
    ../../system/dbus.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.gnome ];
  };

  ###### interface

  options = {

    services.gnome.gnome-online-accounts = {

      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to enable GNOME Online Accounts daemon, a service that provides
          a single sign-on framework for the GNOME desktop.
        '';
      };

    };

  };

  ###### implementation

  config = lib.mkIf config.services.gnome.gnome-online-accounts.enable {

    environment.systemPackages = [ pkgs.gnome-online-accounts ];

    services.dbus.packages = [ pkgs.gnome-online-accounts ];

  };

}
