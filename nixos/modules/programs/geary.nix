{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.programs.geary;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/desktops/gnome/gnome-keyring.nix
    ../services/desktops/gnome/gnome-online-accounts.nix
    ./dconf.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.gnome ];
  };

  options = {
    programs.geary.enable = lib.mkEnableOption "Geary, a Mail client for GNOME";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.geary ];
    programs.dconf.enable = true;
    services.gnome.gnome-keyring.enable = true;
    services.gnome.gnome-online-accounts.enable = true;
  };
}
