{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.umbriel;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../config/xdg/portal.nix
    ../../config/xdg/portals/wlr.nix
    ../../security/pam.nix
    ../../security/polkit.nix
    ../../services/display-managers/default.nix
    ../../services/misc/graphical-desktop.nix
    ../../services/x11/desktop-managers/none.nix
    ../../system/boot/systemd.nix
    ../../system/boot/systemd/user.nix
    ../dconf.nix
    ../xwayland.nix
    # END generated imports
  ];

  options.programs.umbriel = {
    enable = lib.mkEnableOption "Umbriel, a Wayland compositor built on wlroots and SceneFX";
    package = lib.mkPackageOption pkgs "umbriel" { };
    portalPackage = lib.mkPackageOption pkgs "xdg-desktop-portal-umbriel" { };
  };

  config = lib.mkIf cfg.enable (
    lib.mkMerge [
      {
        environment.systemPackages = [ cfg.package ];
        services.displayManager.sessionPackages = [ cfg.package ];
        systemd.packages = [ cfg.package ];

        systemd.user.services.umbriel = {
          restartIfChanged = false;
          enableDefaultPath = false;
        };

        xdg.portal = {
          enable = lib.mkDefault true;
          extraPortals = [ cfg.portalPackage ];
          configPackages = [ cfg.portalPackage ];
        };
      }

      (import ./wayland-session.nix {
        inherit lib pkgs;
        enableXWayland = false;
        enableWlrPortal = false;
      })
    ]
  );

  meta.maintainers = with lib.maintainers; [
    samiser
    pyrox0
  ];
}
