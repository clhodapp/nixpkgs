{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.ente-auth;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/desktop-managers/plasma6.nix
    ../services/desktops/gnome/gnome-keyring.nix
    # END generated imports
  ];

  options.programs.ente-auth = {
    enable = lib.mkEnableOption "Ente Auth";
    package = lib.mkPackageOption pkgs "ente-auth" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.gnome.gnome-keyring.enable = lib.mkIf (
      !config.services.desktopManager.plasma6.enable
    ) true;
  };

  meta.maintainers = with lib.maintainers; [
    gepbird
    yiyu
  ];
}
