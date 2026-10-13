# GNOME User Share daemon.

{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
