# Remote desktop daemon using Pipewire.
{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../../lib/modules/generic/meta-maintainers.nix
    ../../../config/system-path.nix
    ../../../config/users-groups.nix
    ../../../security/polkit.nix
    ../../../system/boot/systemd.nix
    ../../../system/boot/systemd/tmpfiles.nix
    ../../system/dbus.nix
    ../pipewire/pipewire.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.gnome ];
  };

  ###### interface
  options = {
    services.gnome.gnome-remote-desktop = {
      enable = lib.mkEnableOption "Remote Desktop support using Pipewire";
    };
  };

  ###### implementation
  config = lib.mkIf config.services.gnome.gnome-remote-desktop.enable {
    services.pipewire.enable = true;
    services.dbus.packages = [ pkgs.gnome-remote-desktop ];
    security.polkit = {
      enable = true;
      enablePkexecWrapper = lib.mkDefault true;
    };

    environment.systemPackages = [ pkgs.gnome-remote-desktop ];

    systemd.packages = [ pkgs.gnome-remote-desktop ];
    systemd.tmpfiles.packages = [ pkgs.gnome-remote-desktop ];

    # TODO: if possible, switch to using provided g-r-d sysusers.d
    users = {
      users.gnome-remote-desktop = {
        isSystemUser = true;
        group = "gnome-remote-desktop";
        home = "/var/lib/gnome-remote-desktop";
      };
      groups.gnome-remote-desktop = { };
    };
  };
}
