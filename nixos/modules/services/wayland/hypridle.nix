{
  lib,
  pkgs,
  config,
  ...
}:

let
  cfg = config.services.hypridle;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../programs/wayland/hyprland.nix
    ../../programs/wayland/hyprlock.nix
    ../../system/boot/systemd.nix
    ../../system/boot/systemd/user.nix
    # END generated imports
  ];

  options.services.hypridle = {
    enable = lib.mkEnableOption "hypridle, Hyprland's idle daemon";
    package = lib.mkPackageOption pkgs "hypridle" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    systemd = {
      packages = [ cfg.package ];
      user.services.hypridle.wantedBy = [ "graphical-session.target" ];
      user.services.hypridle.path = [
        config.programs.hyprland.package
        config.programs.hyprlock.package
        pkgs.procps
      ];
    };
  };

  meta.teams = [ lib.teams.hyprland ];
}
