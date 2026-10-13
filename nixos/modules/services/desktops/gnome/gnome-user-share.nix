# GNOME User Share daemon.

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
    ../../../system/boot/systemd.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.gnome ];
  };

  ###### interface

  options = {

    services.gnome.gnome-user-share = {

      enable = lib.mkEnableOption "GNOME User Share, a user-level file sharing service for GNOME";

    };

  };

  ###### implementation

  config = lib.mkIf config.services.gnome.gnome-user-share.enable {

    environment.systemPackages = [
      pkgs.gnome-user-share
    ];

    systemd.packages = [
      pkgs.gnome-user-share
    ];

  };

}
