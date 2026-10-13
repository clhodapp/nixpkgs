# GNOME Sushi daemon.

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
    ../../system/dbus.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.gnome ];
  };

  ###### interface

  options = {

    services.gnome.sushi = {

      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to enable Sushi, a quick previewer for nautilus.
        '';
      };

    };

  };

  ###### implementation

  config = lib.mkIf config.services.gnome.sushi.enable {

    environment.systemPackages = [ pkgs.sushi ];

    services.dbus.packages = [ pkgs.sushi ];

  };

}
