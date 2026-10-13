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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
