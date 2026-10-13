{
  lib,
  pkgs,
  config,
  ...
}:

let
  cfg = config.programs.wayvnc;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../security/pam.nix
    # END generated imports
  ];

  options.programs.wayvnc = {
    enable = lib.mkEnableOption "wayvnc, VNC server for wlroots based Wayland compositors";
    package = lib.mkPackageOption pkgs "wayvnc" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    # https://github.com/any1/wayvnc/blob/master/src/pam_auth.c
    security.pam.services.wayvnc = { };
  };

  meta.maintainers = with lib.maintainers; [ qusic ];
}
