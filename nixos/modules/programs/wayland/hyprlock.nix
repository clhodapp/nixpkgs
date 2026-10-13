{
  lib,
  pkgs,
  config,
  ...
}:

let
  cfg = config.programs.hyprlock;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../security/pam.nix
    ../../services/wayland/hypridle.nix
    # END generated imports
  ];

  options.programs.hyprlock = {
    enable = lib.mkEnableOption "hyprlock, Hyprland's GPU-accelerated screen locking utility";
    package = lib.mkPackageOption pkgs "hyprlock" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      cfg.package
    ];

    # Hyprlock needs Hypridle systemd service to be running to detect idle time
    services.hypridle.enable = true;

    # Hyprlock needs PAM access to authenticate, else it fallbacks to su
    security.pam.services.hyprlock = { };
  };

  meta.teams = [ lib.teams.hyprland ];
}
