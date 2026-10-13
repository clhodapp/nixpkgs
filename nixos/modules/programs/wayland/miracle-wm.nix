{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.programs.wayland.miracle-wm;
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
    ../dconf.nix
    ../xwayland.nix
    # END generated imports
  ];

  options.programs.wayland.miracle-wm = {
    enable = lib.mkEnableOption ''
      miracle-wm, a tiling Mir based Wayland compositor. You can manually launch miracle-wm by
      executing "exec miracle-wm" on a TTY, or launch it from a display manager.
      Consult the USERGUIDE.md at <https://github.com/mattkae/miracle-wm> for information on
      how to use & configure it
    '';
  };

  config = lib.mkIf cfg.enable (
    lib.mkMerge [
      {
        environment = {
          systemPackages = [ pkgs.miracle-wm ];
        };

        # To make the miracle-wm session available if a display manager like SDDM is enabled:
        services.displayManager.sessionPackages = [ pkgs.miracle-wm ];
      }

      (import ./wayland-session.nix {
        inherit lib pkgs;
        # Hardcoded path in Mir, not really possible to disable
        enableXWayland = true;
        # No portal support yet: https://github.com/mattkae/miracle-wm/issues/164
        enableWlrPortal = false;
        enableGtkPortal = false;
      })
    ]
  );

  meta.maintainers = with lib.maintainers; [ OPNA2608 ];
}
