# GLib Networking

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
    ../../../config/system-environment.nix
    ../../../system/boot/systemd.nix
    ../../system/dbus.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.gnome ];
  };

  ###### interface

  options = {

    services.gnome.glib-networking = {

      enable = lib.mkEnableOption "network extensions for GLib";

    };

  };

  ###### implementation

  config = lib.mkIf config.services.gnome.glib-networking.enable {

    services.dbus.packages = [ pkgs.glib-networking ];

    systemd.packages = [ pkgs.glib-networking ];

    environment.sessionVariables.GIO_EXTRA_MODULES = [ "${pkgs.glib-networking.out}/lib/gio/modules" ];

  };

}
