{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.xfconf;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/system/dbus.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.xfce ];
  };

  options = {
    programs.xfconf = {
      enable = lib.mkEnableOption "Xfconf, the Xfce configuration storage system";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      pkgs.xfconf
    ];

    services.dbus.packages = [
      pkgs.xfconf
    ];
  };
}
