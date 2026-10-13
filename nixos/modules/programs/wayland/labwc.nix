{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.labwc;
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

  meta.maintainers = [ ];

  options.programs.labwc = {
    enable = lib.mkEnableOption "labwc";
    package = lib.mkPackageOption pkgs "labwc" { };
  };

  config = lib.mkIf cfg.enable (
    lib.mkMerge [
      {
        environment.systemPackages = [ cfg.package ];

        xdg.portal.config.wlroots.default = lib.mkDefault [
          "wlr"
          "gtk"
        ];

        # To make a labwc session available for certain DMs like SDDM
        services.displayManager.sessionPackages = [ cfg.package ];
      }
      (import ./wayland-session.nix { inherit lib pkgs; })
    ]
  );
}
